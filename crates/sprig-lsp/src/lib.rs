//! The Sprig language server.
//!
//! Built the way rust-analyzer is, on `lsp-server` and `lsp-types`: a plain
//! loop over messages on one thread, with no async runtime, since every answer
//! is a function of the files and the date and resolving a whole workspace
//! takes milliseconds. Each change marks the workspace stale; it is resolved
//! again once the queue of waiting messages is empty, so a burst of keystrokes
//! costs one resolution, and diagnostics are republished for every file whose
//! problems changed, including files nobody edited (cairn 0049).
//!
//! The server reads files, which the core never does: the folder the editor
//! opened is the workspace root, and an open document's text wins over what
//! is on disk. Like the CLI, it reads nothing outside the root, and it takes
//! today's date from its caller rather than the clock.

use std::collections::BTreeMap;
use std::error::Error as StdError;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::str::FromStr;

use lsp_server::{Connection, ErrorCode, Message, Notification, Request, RequestId, Response};
use lsp_types::notification::Notification as _;
use lsp_types::request::Request as _;
use lsp_types::{self as lsp, Uri};
use serde_json::json;
use sprig_core::{Date, Resolution, Workspace};

mod features;
mod pos;

use pos::{Encoding, Lines};

pub type Error = Box<dyn StdError + Send + Sync>;

/// Why no message was waiting.
enum TryRecvError {
    Empty,
    Disconnected,
}

/// Serves on stdin and stdout until the client says exit.
pub fn stdio(today: &dyn Fn() -> Date) -> Result<(), Error> {
    let (conn, io) = Connection::stdio();
    run(&conn, today)?;
    // The writer thread ends when the connection's sender is dropped.
    drop(conn);
    io.join()?;
    Ok(())
}

/// Serves on one connection: the initialize handshake, then requests and
/// notifications until shutdown.
pub fn run(conn: &Connection, today: &dyn Fn() -> Date) -> Result<(), Error> {
    let (id, params) = conn.initialize_start()?;
    let init: lsp::InitializeParams = serde_json::from_value(params)?;
    let utf8 = init
        .capabilities
        .general
        .as_ref()
        .and_then(|g| g.position_encodings.as_ref())
        .is_some_and(|e| e.contains(&lsp::PositionEncodingKind::UTF8));
    let encoding = if utf8 {
        Encoding::Utf8
    } else {
        Encoding::Utf16
    };
    let capabilities = lsp::ServerCapabilities {
        position_encoding: Some(if utf8 {
            lsp::PositionEncodingKind::UTF8
        } else {
            lsp::PositionEncodingKind::UTF16
        }),
        // Plans are small and line-based, so whole-text sync costs little
        // and cannot drift out of step with the client.
        text_document_sync: Some(lsp::TextDocumentSyncCapability::Kind(
            lsp::TextDocumentSyncKind::FULL,
        )),
        completion_provider: Some(lsp::CompletionOptions {
            trigger_characters: Some(["@", "^", "[", ":"].map(String::from).to_vec()),
            ..Default::default()
        }),
        definition_provider: Some(lsp::OneOf::Left(true)),
        references_provider: Some(lsp::OneOf::Left(true)),
        hover_provider: Some(lsp::HoverProviderCapability::Simple(true)),
        code_action_provider: Some(lsp::CodeActionProviderCapability::Simple(true)),
        document_formatting_provider: Some(lsp::OneOf::Left(true)),
        ..Default::default()
    };
    conn.initialize_finish(
        id,
        json!({
            "capabilities": capabilities,
            "serverInfo": {"name": "sprig", "version": env!("CARGO_PKG_VERSION")},
        }),
    )?;
    let watch = init
        .capabilities
        .workspace
        .as_ref()
        .and_then(|w| w.did_change_watched_files)
        .and_then(|d| d.dynamic_registration)
        .unwrap_or(false);
    let mut server = Server {
        conn,
        today,
        encoding,
        watch,
        root: None,
        disk: BTreeMap::new(),
        open: BTreeMap::new(),
        uris: BTreeMap::new(),
        published: BTreeMap::new(),
        snapshot: None,
        stale: false,
    };
    // initialize_finish has already waited for `initialized`, after which a
    // server may ask the client for things.
    server.register_watcher()?;
    if let Some(root) = root_of(&init) {
        server.set_root(root);
    }
    server.serve()
}

#[allow(deprecated)] // root_uri, for clients that send no workspace folders
fn root_of(init: &lsp::InitializeParams) -> Option<PathBuf> {
    let uri = init
        .workspace_folders
        .as_ref()
        .and_then(|f| f.first())
        .map(|f| &f.uri)
        .or(init.root_uri.as_ref())?;
    path_of(uri)
}

fn path_of(uri: &Uri) -> Option<PathBuf> {
    url::Url::parse(uri.as_str()).ok()?.to_file_path().ok()
}

/// Every file, resolved for one day.
pub(crate) struct Snapshot {
    pub files: BTreeMap<String, String>,
    pub res: Resolution,
    pub today: Date,
}

struct Server<'a> {
    conn: &'a Connection,
    today: &'a dyn Fn() -> Date,
    encoding: Encoding,
    /// The client can watch files for us.
    watch: bool,
    root: Option<PathBuf>,
    /// Files as they are on disk, by workspace path.
    disk: BTreeMap<String, String>,
    /// Open documents' text, which wins over the disk's.
    open: BTreeMap<String, String>,
    /// The URI each open document was opened with, echoed back exactly.
    uris: BTreeMap<String, Uri>,
    /// What was last published for each file, so only changes are sent.
    published: BTreeMap<String, Vec<lsp::Diagnostic>>,
    snapshot: Option<Snapshot>,
    /// Something changed since diagnostics were last published.
    stale: bool,
}

impl Server<'_> {
    fn serve(&mut self) -> Result<(), Error> {
        loop {
            // Publish only once the queue is empty, so a burst of changes is
            // resolved once.
            let msg = if self.stale {
                match try_recv(self.conn) {
                    Ok(m) => m,
                    Err(TryRecvError::Empty) => {
                        self.publish()?;
                        continue;
                    }
                    Err(TryRecvError::Disconnected) => return Ok(()),
                }
            } else {
                match self.conn.receiver.recv() {
                    Ok(m) => m,
                    Err(_) => return Ok(()),
                }
            };
            match msg {
                Message::Request(req) => {
                    if self.conn.handle_shutdown(&req)? {
                        return Ok(());
                    }
                    let resp = self.request(req);
                    self.conn.sender.send(Message::Response(resp))?;
                }
                Message::Notification(n) => self.notification(n)?,
                Message::Response(_) => {}
            }
        }
    }

    fn set_root(&mut self, root: PathBuf) {
        let mut found = Vec::new();
        walk(&root, &root, &mut found);
        let canonical = root.canonicalize().unwrap_or_else(|_| root.clone());
        for (key, path) in found {
            if let Some(src) = read(&path, &canonical) {
                self.disk.insert(key, src);
            }
        }
        self.root = Some(root);
        self.changed();
    }

    fn changed(&mut self) {
        self.snapshot = None;
        self.stale = true;
    }

    /// The workspace path of a document, when it is a `.sprig` file under
    /// the root.
    fn key(&self, uri: &Uri) -> Option<String> {
        key_under(self.root.as_deref()?, &path_of(uri)?)
    }

    fn uri(&self, key: &str) -> Option<Uri> {
        if let Some(u) = self.uris.get(key) {
            return Some(u.clone());
        }
        let path = self.root.as_ref()?.join(key);
        let url = url::Url::from_file_path(path).ok()?;
        Uri::from_str(url.as_str()).ok()
    }

    fn snapshot(&mut self) -> &Snapshot {
        if self.snapshot.is_none() {
            let mut files = self.disk.clone();
            files.extend(self.open.iter().map(|(k, v)| (k.clone(), v.clone())));
            let mut ws = Workspace::new();
            for (k, v) in &files {
                ws.insert(k.clone(), v.clone());
            }
            let today = (self.today)();
            let res = ws.resolve(today);
            self.snapshot = Some(Snapshot { files, res, today });
        }
        self.snapshot.as_ref().expect("just built")
    }

    fn publish(&mut self) -> Result<(), Error> {
        self.stale = false;
        let enc = self.encoding;
        let snap = self.snapshot();
        let mut now: BTreeMap<String, Vec<lsp::Diagnostic>> = BTreeMap::new();
        for d in &snap.res.diagnostics {
            let Some(src) = snap.files.get(&d.file) else {
                continue;
            };
            now.entry(d.file.clone())
                .or_default()
                .push(features::diagnostic(d, &Lines::new(src), enc));
        }
        let keys: Vec<String> = now.keys().chain(self.published.keys()).cloned().collect();
        for key in keys {
            let diags = now.remove(&key).unwrap_or_default();
            if self.published.get(&key) == Some(&diags) {
                continue;
            }
            let Some(uri) = self.uri(&key) else { continue };
            let params = lsp::PublishDiagnosticsParams {
                uri,
                diagnostics: diags.clone(),
                version: None,
            };
            self.conn
                .sender
                .send(Message::Notification(Notification::new(
                    lsp::notification::PublishDiagnostics::METHOD.into(),
                    params,
                )))?;
            if diags.is_empty() {
                self.published.remove(&key);
            } else {
                self.published.insert(key, diags);
            }
        }
        Ok(())
    }

    fn notification(&mut self, n: Notification) -> Result<(), Error> {
        use lsp::notification as N;
        match n.method.as_str() {
            N::DidOpenTextDocument::METHOD => {
                let p: lsp::DidOpenTextDocumentParams = serde_json::from_value(n.params)?;
                if self.root.is_none()
                    && let Some(dir) = path_of(&p.text_document.uri)
                        .and_then(|f| f.parent().map(Path::to_path_buf))
                {
                    // A file opened without a folder: its own directory is
                    // the workspace, as for `sprig check FILE`.
                    self.set_root(dir);
                }
                if let Some(key) = self.key(&p.text_document.uri) {
                    self.uris.insert(key.clone(), p.text_document.uri);
                    self.open.insert(key, p.text_document.text);
                    self.changed();
                }
            }
            N::DidChangeTextDocument::METHOD => {
                let p: lsp::DidChangeTextDocumentParams = serde_json::from_value(n.params)?;
                if let Some(key) = self.key(&p.text_document.uri)
                    && let Some(change) = p.content_changes.into_iter().last()
                {
                    self.open.insert(key, change.text);
                    self.changed();
                }
            }
            N::DidCloseTextDocument::METHOD => {
                let p: lsp::DidCloseTextDocumentParams = serde_json::from_value(n.params)?;
                if let Some(key) = self.key(&p.text_document.uri) {
                    self.open.remove(&key);
                    self.reload(&key);
                    self.changed();
                }
            }
            N::DidChangeWatchedFiles::METHOD => {
                let p: lsp::DidChangeWatchedFilesParams = serde_json::from_value(n.params)?;
                for c in p.changes {
                    if let Some(key) = self.key(&c.uri) {
                        self.reload(&key);
                    }
                }
                self.changed();
            }
            _ => {}
        }
        Ok(())
    }

    /// Reads one file from disk again, or forgets it when it is gone.
    fn reload(&mut self, key: &str) {
        let Some(root) = &self.root else { return };
        let canonical = root.canonicalize().unwrap_or_else(|_| root.clone());
        match read(&root.join(key), &canonical) {
            Some(src) => self.disk.insert(key.to_string(), src),
            None => self.disk.remove(key),
        };
    }

    /// Asks the client to report changes to plans on disk, so that a file
    /// changed by `git pull` or `sprig tick` is seen.
    fn register_watcher(&self) -> Result<(), Error> {
        if !self.watch {
            return Ok(());
        }
        let options = lsp::DidChangeWatchedFilesRegistrationOptions {
            watchers: vec![lsp::FileSystemWatcher {
                glob_pattern: lsp::GlobPattern::String("**/*.sprig".into()),
                kind: None,
            }],
        };
        let params = lsp::RegistrationParams {
            registrations: vec![lsp::Registration {
                id: "sprig-watch".into(),
                method: lsp::notification::DidChangeWatchedFiles::METHOD.into(),
                register_options: Some(serde_json::to_value(options)?),
            }],
        };
        self.conn.sender.send(Message::Request(Request::new(
            RequestId::from("sprig-watch".to_string()),
            lsp::request::RegisterCapability::METHOD.into(),
            params,
        )))?;
        Ok(())
    }

    fn request(&mut self, req: Request) -> Response {
        let id = req.id.clone();
        if !ANSWERS.contains(&req.method.as_str()) {
            return Response::new_err(
                id,
                ErrorCode::MethodNotFound as i32,
                format!("sprig does not answer {}", req.method),
            );
        }
        match self.answer(req) {
            Ok(Some(v)) => Response::new_ok(id, v),
            Ok(None) => Response::new_ok(id, serde_json::Value::Null),
            Err(e) => Response::new_err(id, ErrorCode::InvalidParams as i32, e.to_string()),
        }
    }

    fn answer(&mut self, req: Request) -> Result<Option<serde_json::Value>, Error> {
        use lsp::request as R;
        let enc = self.encoding;
        let value = match req.method.as_str() {
            R::Completion::METHOD => {
                let p: lsp::CompletionParams = serde_json::from_value(req.params)?;
                let pos = p.text_document_position;
                let Some(doc) = self.doc(&pos.text_document.uri) else {
                    return Ok(None);
                };
                let snap = self.snapshot();
                serde_json::to_value(features::completion(snap, &doc.0, enc, pos.position))?
            }
            R::GotoDefinition::METHOD => {
                let p: lsp::GotoDefinitionParams = serde_json::from_value(req.params)?;
                let pos = p.text_document_position_params;
                let Some(doc) = self.doc(&pos.text_document.uri) else {
                    return Ok(None);
                };
                let snap = self.snapshot();
                let found = features::definition(snap, &doc.0, enc, pos.position);
                let locs: Vec<lsp::Location> = found
                    .into_iter()
                    .filter_map(|(k, r)| Some(lsp::Location::new(self.uri(&k)?, r)))
                    .collect();
                serde_json::to_value(locs)?
            }
            R::References::METHOD => {
                let p: lsp::ReferenceParams = serde_json::from_value(req.params)?;
                let pos = p.text_document_position;
                let Some(doc) = self.doc(&pos.text_document.uri) else {
                    return Ok(None);
                };
                let snap = self.snapshot();
                let found = features::references(
                    snap,
                    &doc.0,
                    enc,
                    pos.position,
                    p.context.include_declaration,
                );
                let locs: Vec<lsp::Location> = found
                    .into_iter()
                    .filter_map(|(k, r)| Some(lsp::Location::new(self.uri(&k)?, r)))
                    .collect();
                serde_json::to_value(locs)?
            }
            R::HoverRequest::METHOD => {
                let p: lsp::HoverParams = serde_json::from_value(req.params)?;
                let pos = p.text_document_position_params;
                let Some(doc) = self.doc(&pos.text_document.uri) else {
                    return Ok(None);
                };
                let snap = self.snapshot();
                serde_json::to_value(features::hover(snap, &doc.0, enc, pos.position))?
            }
            R::CodeActionRequest::METHOD => {
                let p: lsp::CodeActionParams = serde_json::from_value(req.params)?;
                let Some(doc) = self.doc(&p.text_document.uri) else {
                    return Ok(None);
                };
                let snap = self.snapshot();
                serde_json::to_value(features::actions(snap, &doc.0, &doc.1, enc, p.range))?
            }
            R::Formatting::METHOD => {
                let p: lsp::DocumentFormattingParams = serde_json::from_value(req.params)?;
                let Some(doc) = self.doc(&p.text_document.uri) else {
                    return Ok(None);
                };
                let snap = self.snapshot();
                serde_json::to_value(features::format(snap, &doc.0, enc))?
            }
            _ => unreachable!("request() only passes methods in ANSWERS"),
        };
        Ok(Some(value))
    }

    /// The workspace path and URI of a document the server knows.
    fn doc(&mut self, uri: &Uri) -> Option<(String, Uri)> {
        let key = self.key(uri)?;
        self.snapshot()
            .files
            .contains_key(&key)
            .then(|| (key, uri.clone()))
    }
}

/// The requests the server answers; anything else is MethodNotFound.
const ANSWERS: [&str; 6] = [
    lsp::request::Completion::METHOD,
    lsp::request::GotoDefinition::METHOD,
    lsp::request::References::METHOD,
    lsp::request::HoverRequest::METHOD,
    lsp::request::CodeActionRequest::METHOD,
    lsp::request::Formatting::METHOD,
];

fn try_recv(conn: &Connection) -> Result<Message, TryRecvError> {
    conn.receiver.try_recv().map_err(|e| {
        if e.is_empty() {
            TryRecvError::Empty
        } else {
            TryRecvError::Disconnected
        }
    })
}

/// The workspace path of `file` under `root`: `/`-separated, and only for a
/// `.sprig` file with no `..` in the way.
fn key_under(root: &Path, file: &Path) -> Option<String> {
    let rel = file.strip_prefix(root).ok()?;
    let parts: Option<Vec<&str>> = rel
        .components()
        .map(|c| match c {
            Component::Normal(s) => s.to_str(),
            _ => None,
        })
        .collect();
    let key = parts?.join("/");
    key.ends_with(".sprig").then_some(key)
}

/// Reads a file, or `None` if it is unreadable, not UTF-8, or resolves
/// outside the root: a symlink is untrusted input, as a graft path is.
fn read(path: &Path, root: &Path) -> Option<String> {
    let real = path.canonicalize().ok()?;
    if !real.starts_with(root) || !real.is_file() {
        return None;
    }
    String::from_utf8(fs::read(&real).ok()?).ok()
}

/// Every `.sprig` file under `dir`, skipping hidden directories and the
/// build output of the tools a plan usually sits beside, and not following
/// directory links.
fn walk(root: &Path, dir: &Path, out: &mut Vec<(String, PathBuf)>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = entries.flatten().collect();
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let path = e.path();
        let name = e.file_name();
        let name = name.to_string_lossy();
        let Ok(kind) = e.file_type() else { continue };
        if kind.is_dir() {
            if !name.starts_with('.') && name != "node_modules" && name != "target" {
                walk(root, &path, out);
            }
        } else if let Some(key) = key_under(root, &path) {
            out.push((key, path));
        }
    }
}
