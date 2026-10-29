//! `sprig tui`: the tree, what's next and the agenda on one screen.
//!
//! The pure core is in [`app`]: state, keys and the edits they ask for. This
//! module is the shell around it, the only part that touches the terminal or
//! the disk: it turns key presses into [`app::Key`]s, performs the
//! [`app::Action`]s that come back, and reads the files again when they
//! change underneath it.
//!
//! A write re-reads its file and is refused if the line it is aimed at no
//! longer says what the TUI showed, so an editor or an agent saving the same
//! file never has its change overwritten (spec/tools.md T7).

pub mod app;
pub mod ui;

use std::fs;
use std::io::{self, IsTerminal};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::mpsc;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use sprig_core::Date;

use crate::Result;
use crate::args::TuiArgs;
use crate::load::{self, Loaded};
use crate::verbs::run_editor;
use crate::write::save;
use app::{Action, App, Key};

fn read(path: &Path) -> Result<Loaded> {
    let mut groups = load::load(std::slice::from_ref(&path.to_path_buf()))?;
    Ok(groups.pop().expect("one path gives one workspace"))
}

/// What the loop does after an action.
enum Flow {
    Go,
    Quit,
    Edit(PathBuf, usize),
}

/// Performs an action: the only place the TUI writes a file.
fn perform(app: &mut App, path: &Path, root: &Path, action: Action, today: Date) -> Result<Flow> {
    match action {
        Action::None => {}
        Action::Quit => return Ok(Flow::Quit),
        Action::Reload => {
            app.reload(read(path)?.workspace);
            app.flash = Some("read the files again".into());
        }
        Action::Edit { file, line } => return Ok(Flow::Edit(root.join(file), line)),
        Action::Write(w) => {
            let target = root.join(&w.file);
            let disk = fs::read_to_string(&target)
                .map_err(|e| format!("cannot read {}: {e}", target.display()))?;
            match app::apply(&disk, &w, today) {
                Ok((new, line)) => {
                    save(&target, &new).map_err(|e| crate::write::write_error(&target, e))?;
                    app.reload(read(path)?.workspace);
                    app.written(&w, disk, new, line);
                }
                Err(why) => {
                    // Show what the file says now, so the next try is aimed right.
                    app.reload(read(path)?.workspace);
                    app.failed(why);
                }
            }
        }
    }
    Ok(Flow::Go)
}

/// `--keys`: characters as typed, and `<Esc>`, `<Enter>`, `<BS>`, `<Up>`,
/// `<Down>`, `<Left>`, `<Right>`, `<Home>`, `<End>` and `<lt>` for the rest.
pub fn parse_keys(spec: &str) -> std::result::Result<Vec<Key>, String> {
    let mut out = Vec::new();
    let mut chars = spec.chars();
    while let Some(c) = chars.next() {
        if c != '<' {
            out.push(Key::Char(c));
            continue;
        }
        let name: String = chars.by_ref().take_while(|&c| c != '>').collect();
        out.push(match name.as_str() {
            "Esc" => Key::Esc,
            "Enter" | "CR" => Key::Enter,
            "BS" => Key::Backspace,
            "Up" => Key::Up,
            "Down" => Key::Down,
            "Left" => Key::Left,
            "Right" => Key::Right,
            "Home" => Key::Home,
            "End" => Key::End,
            "lt" => Key::Char('<'),
            other => return Err(format!("--keys: <{other}> is not a key")),
        });
    }
    Ok(out)
}

fn size(spec: &str) -> std::result::Result<(u16, u16), String> {
    let bad = || format!("--screenshot takes WIDTHxHEIGHT, such as 100x30, not {spec}");
    let (w, h) = spec.split_once('x').ok_or_else(bad)?;
    let (w, h): (u16, u16) = (w.parse().map_err(|_| bad())?, h.parse().map_err(|_| bad())?);
    if w < 20 || h < 5 {
        return Err(format!("--screenshot needs at least 20x5, not {spec}"));
    }
    Ok((w, h))
}

pub fn tui(a: &TuiArgs, today: &dyn Fn() -> Date) -> Result<ExitCode> {
    let path = a.path.clone().unwrap_or_else(|| PathBuf::from("."));
    let g = read(&path)?;
    let root = g.root.clone();
    let day = today();
    let mut app = App::new(g.workspace, g.entries, day);

    if let Some(spec) = &a.screenshot {
        let (w, h) = match size(spec) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("sprig: {e}");
                return Ok(ExitCode::from(2));
            }
        };
        let keys = match a.keys.as_deref().map(parse_keys).transpose() {
            Ok(k) => k.unwrap_or_default(),
            Err(e) => {
                eprintln!("sprig: {e}");
                return Ok(ExitCode::from(2));
            }
        };
        // The keys run through the same core and the same writes as a
        // person's would; only the editor, which needs a person, is skipped.
        for k in keys {
            let action = app.key(k);
            match perform(&mut app, &path, &root, action, day)? {
                Flow::Quit => break,
                Flow::Edit(file, line) => {
                    app.flash = Some(format!("would open {}:{line}", file.display()));
                }
                Flow::Go => {}
            }
        }
        print!("{}", ui::render_to_string(&app, w, h));
        return Ok(ExitCode::SUCCESS);
    }

    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        eprintln!("sprig: tui needs a terminal; --screenshot WxH prints one frame instead");
        return Ok(ExitCode::from(2));
    }
    interactive(app, &path, &root, today)
}

fn key(e: event::KeyEvent) -> Option<Key> {
    if e.kind != KeyEventKind::Press {
        return None;
    }
    if e.modifiers.contains(KeyModifiers::CONTROL) && e.code == KeyCode::Char('c') {
        return Some(Key::Char('q'));
    }
    Some(match e.code {
        KeyCode::Char(c) => Key::Char(c),
        KeyCode::Enter => Key::Enter,
        KeyCode::Esc => Key::Esc,
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::Left => Key::Left,
        KeyCode::Right => Key::Right,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        _ => return None,
    })
}

/// Sends on `tx` whenever something under `root` changes. The folder is
/// watched, not the files, so a save that writes a new file and renames it
/// over the old one, as Vim's does, is seen like any other.
fn watch(root: &Path, tx: mpsc::Sender<()>) -> notify::Result<notify::RecommendedWatcher> {
    use notify::Watcher;
    let mut w = notify::recommended_watcher(move |e: notify::Result<notify::Event>| {
        if e.is_ok_and(|e| !e.kind.is_access()) {
            let _ = tx.send(());
        }
    })?;
    w.watch(root, notify::RecursiveMode::Recursive)?;
    Ok(w)
}

/// How often the loop looks for a key or a change on disk.
const TICK: Duration = Duration::from_millis(60);

fn interactive(
    mut app: App,
    path: &Path,
    root: &Path,
    today: &dyn Fn() -> Date,
) -> Result<ExitCode> {
    // Editors save in bursts (write a temporary file, rename it into place),
    // so a reload waits for a tick with no events after the last one.
    let (tx, rx) = mpsc::channel();
    let watch_root = if root.is_dir() {
        root.to_path_buf()
    } else {
        PathBuf::from(".")
    };
    let _watcher = watch(&watch_root, tx);
    // Without a watcher, `r` still reads the files again.
    if let Err(e) = &_watcher {
        app.flash = Some(format!("not watching for changes ({e}); r reloads"));
    }

    let mut terminal = ratatui::init();
    let mut pending = false;
    let result = loop {
        app.today = today();
        if let Err(e) = terminal.draw(|f| ui::draw(f, &app)) {
            break Err(e.into());
        }
        let mut changed = false;
        while rx.try_recv().is_ok() {
            changed = true;
        }
        if pending && !changed {
            pending = false;
            match read(path) {
                Ok(g) => app.reload(g.workspace),
                Err(e) => app.failed(e.to_string()),
            }
        }
        pending |= changed;
        match event::poll(TICK) {
            Ok(false) => continue,
            Ok(true) => {}
            Err(e) => break Err(e.into()),
        }
        let Ok(Event::Key(k)) = event::read() else {
            continue;
        };
        let Some(k) = key(k) else { continue };
        let action = app.key(k);
        let day = app.today;
        match perform(&mut app, path, root, action, day) {
            Ok(Flow::Go) => {}
            Ok(Flow::Quit) => break Ok(ExitCode::SUCCESS),
            Ok(Flow::Edit(file, line)) => {
                ratatui::restore();
                let ran = run_editor(&file, line);
                terminal = ratatui::init();
                match ran {
                    Ok(_) => match read(path) {
                        Ok(g) => app.reload(g.workspace),
                        Err(e) => app.failed(e.to_string()),
                    },
                    Err(e) => app.failed(e.to_string()),
                }
            }
            Err(e) => app.failed(e.to_string()),
        }
    };
    ratatui::restore();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn folder(name: &str, text: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("sprig-tui-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("plan.sprig"), text).unwrap();
        dir
    }

    fn today() -> Date {
        Date::new(2026, 10, 1).unwrap()
    }

    fn open(dir: &Path) -> App {
        let g = read(dir).unwrap();
        App::new(g.workspace, g.entries, today())
    }

    fn to(a: &mut App, label: &str) {
        a.cursor = a.rows.iter().position(|r| r.label == label).unwrap();
    }

    #[test]
    fn a_change_saved_elsewhere_appears_and_the_cursor_stays_on_its_item() {
        let dir = folder("reload", "Plan\n\n- One\n- Two\n- Three\n");
        let mut app = open(&dir);
        to(&mut app, "Three");
        // An editor adds a line above and ticks Three.
        fs::write(
            dir.join("plan.sprig"),
            "Plan\n\n- Zero\n- One\n- Two\nx Three\n",
        )
        .unwrap();
        app.reload(read(&dir).unwrap().workspace);
        let row = &app.rows[app.cursor];
        assert_eq!((row.label.as_str(), row.glyph), ("Three", 'x'));
        assert!(app.rows.iter().any(|r| r.label == "Zero"));
    }

    #[test]
    fn a_write_raced_by_an_editor_is_refused_and_the_editors_change_kept() {
        let dir = folder("race", "Plan\n\n- One\n- Two\n");
        let mut app = open(&dir);
        to(&mut app, "Two");
        let action = app.key(Key::Char('x'));
        // Between the key and its write, an editor rewrites that line.
        let theirs = "Plan\n\n- One\n- Two, and three\n";
        fs::write(dir.join("plan.sprig"), theirs).unwrap();
        let root = dir.clone();
        perform(&mut app, &dir, &root, action, today()).unwrap();
        assert_eq!(fs::read_to_string(dir.join("plan.sprig")).unwrap(), theirs);
        assert_eq!(
            app.flash.as_deref(),
            Some("plan.sprig:4 changed on disk since it was read; nothing was written")
        );
        // The TUI now shows their text, so the next tick lands on it.
        assert!(app.rows.iter().any(|r| r.label == "Two, and three"));
        to(&mut app, "Two, and three");
        let action = app.key(Key::Char('x'));
        perform(&mut app, &dir, &root, action, today()).unwrap();
        assert_eq!(
            fs::read_to_string(dir.join("plan.sprig")).unwrap(),
            "Plan\n\n- One\nx Two, and three\n"
        );
    }

    #[test]
    fn a_change_elsewhere_in_the_file_does_not_stop_a_write() {
        let dir = folder("elsewhere", "Plan\n\n- One\n- Two\n");
        let mut app = open(&dir);
        to(&mut app, "Two");
        let action = app.key(Key::Char('x'));
        fs::write(dir.join("plan.sprig"), "Plan\n\nx One\n- Two\n").unwrap();
        let root = dir.clone();
        perform(&mut app, &dir, &root, action, today()).unwrap();
        assert_eq!(
            fs::read_to_string(dir.join("plan.sprig")).unwrap(),
            "Plan\n\nx One\nx Two\n"
        );
    }

    #[test]
    fn a_save_by_rename_is_seen_by_the_watcher() {
        let dir = folder("watch", "Plan\n\n- One\n");
        let (tx, rx) = mpsc::channel();
        let _w = watch(&dir, tx).unwrap();
        // What Vim does with `backupcopy=no`: write a new file, then rename
        // it over the old one.
        fs::write(dir.join(".plan.sprig.swp"), "Plan\n\nx One\n").unwrap();
        fs::rename(dir.join(".plan.sprig.swp"), dir.join("plan.sprig")).unwrap();
        assert!(
            rx.recv_timeout(Duration::from_secs(5)).is_ok(),
            "no event within 5s"
        );
        let mut app = open(&dir);
        app.reload(read(&dir).unwrap().workspace);
        assert!(app.rows.iter().any(|r| r.label == "One" && r.glyph == 'x'));
    }

    #[test]
    fn keys_parse_names_in_angle_brackets() {
        assert_eq!(
            parse_keys("jx<Esc><Enter><BS><lt>").unwrap(),
            [
                Key::Char('j'),
                Key::Char('x'),
                Key::Esc,
                Key::Enter,
                Key::Backspace,
                Key::Char('<')
            ]
        );
        assert!(parse_keys("<Nope>").is_err());
    }
}
