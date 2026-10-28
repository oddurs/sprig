//! Inline tokens (§4): what an item's line says besides its text.

use crate::diagnostic::Span;

/// A `key:value` token (§4.6). Every occurrence is kept, in order, so a tool
/// can point at the one it means; [`Tokens::field`] gives the value that counts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Field {
    /// Lower-cased: keys are case-insensitive.
    pub key: String,
    /// Without its quotes, if it had them.
    pub value: String,
    pub span: Span,
}

/// One `after:` value (§4.7), exactly as written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct After {
    pub value: String,
    pub span: Span,
}

/// A `[[file]]` or `[[file^anchor]]` link (§4.8).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Link {
    /// The file as written, with a leading `./` dropped and `.sprig` appended
    /// when absent. Still relative to the file that contains the link.
    pub path: String,
    pub anchor: Option<String>,
    /// The `[[...]]` itself.
    pub span: Span,
}

/// A token carrying an anchor (§4.5).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Anchor {
    pub name: String,
    pub span: Span,
}

/// Everything lifted from one line.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tokens {
    /// What is left, joined by single spaces, with link brackets dropped (§4.9).
    pub text: String,
    /// The words that make up the text, as written.
    pub words: Vec<String>,
    pub people: Vec<String>,
    pub tags: Vec<String>,
    /// 0 when unset, else 1 to 3.
    pub priority: u8,
    /// Every anchor written, in order. The last one names the item (Appendix
    /// B.3); more than one is reported.
    pub anchors: Vec<Anchor>,
    pub fields: Vec<Field>,
    pub after: Vec<After>,
    /// Links inside words. A graft uses the first; `after:` links are in
    /// [`Tokens::after`], not here.
    pub links: Vec<Link>,
    /// The first word containing `[[`, which is what a graft line grafts.
    pub(crate) graft_word: Option<usize>,
    /// For each link, the index of the word it came from.
    pub(crate) link_words: Vec<usize>,
}

impl Tokens {
    /// The value that counts for `key`: the last one written (§4.6).
    pub fn field(&self, key: &str) -> Option<&Field> {
        self.fields.iter().rev().find(|f| f.key == key)
    }

    /// Fields with the value that counts for each key, in order of first use.
    pub fn effective_fields(&self) -> Vec<&Field> {
        let mut out: Vec<&Field> = Vec::new();
        for f in &self.fields {
            if !out.iter().any(|o| o.key == f.key) {
                out.push(self.field(&f.key).expect("the key was just seen"));
            }
        }
        out
    }

    pub fn anchor(&self) -> Option<&Anchor> {
        self.anchors.last()
    }

    /// The link a graft line grafts: the first link of its first word that
    /// contains `[[`. A word with `[[` but no well-formed link means the
    /// graft has none, as in the reference parser.
    pub fn graft_link(&self) -> Option<&Link> {
        let word = self.graft_word?;
        let i = self.link_words.iter().position(|&w| w == word)?;
        self.links.get(i)
    }
}

/// Splits `text`, which starts at byte `base` of its file, and lifts its
/// tokens.
pub(crate) fn lift(text: &str, base: usize) -> Tokens {
    let mut t = Tokens::default();
    for (start, tok) in split(text) {
        let span = Span::new(base + start, base + start + tok.len());
        if let Some(name) = tok.strip_prefix('@').filter(|n| is_name(n, ".-_")) {
            t.people.push(name.to_string());
        } else if let Some(name) = tok.strip_prefix('#').filter(|n| is_name(n, "-_")) {
            t.tags.push(name.to_string());
        } else if matches!(tok, "!" | "!!" | "!!!") {
            t.priority = t.priority.max(tok.len() as u8);
        } else if let Some(name) = tok.strip_prefix('^').filter(|n| is_name(n, "-_")) {
            t.anchors.push(Anchor {
                name: name.to_string(),
                span,
            });
        } else if let Some((key, value)) = field(tok) {
            if key == "after" {
                t.after.push(After { value, span });
            } else {
                t.fields.push(Field { key, value, span });
            }
        } else {
            let index = t.words.len();
            if t.graft_word.is_none() && tok.contains("[[") {
                t.graft_word = Some(index);
            }
            for (at, link) in links(tok) {
                let span = Span::new(span.start + at, span.start + at + link.span.end);
                t.links.push(Link { span, ..link });
                t.link_words.push(index);
            }
            t.words.push(tok.to_string());
        }
    }
    t.text = t
        .words
        .iter()
        .map(|w| plain(w))
        .collect::<Vec<_>>()
        .join(" ");
    t
}

/// What a piece of a line is, for an editor to colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Piece {
    Person,
    Tag,
    Priority,
    Anchor,
    /// A field's key and its colon.
    Key,
    /// A field's value: plain, a link or an anchor.
    Value,
    Link,
}

impl Piece {
    pub fn as_str(self) -> &'static str {
        match self {
            Piece::Person => "person",
            Piece::Tag => "tag",
            Piece::Priority => "priority",
            Piece::Anchor => "anchor",
            Piece::Key => "key",
            Piece::Value => "value",
            Piece::Link => "link",
        }
    }
}

/// The coloured pieces of `text`, which starts at byte `base`. Words between
/// them are plain and not listed. In a note (§2.6) only people and links are
/// shown as such, and a person may carry trailing punctuation, since notes
/// are prose.
pub fn pieces(text: &str, base: usize, note: bool) -> Vec<(Piece, Span)> {
    let mut out = Vec::new();
    let add_links = |tok: &str, at: usize, out: &mut Vec<(Piece, Span)>| {
        for (i, l) in links(tok) {
            out.push((Piece::Link, Span::new(at + i, at + i + l.span.end)));
        }
    };
    for (start, tok) in split(text) {
        let at = base + start;
        let span = Span::new(at, at + tok.len());
        if note {
            let bare = tok.trim_end_matches(['.', ',', ';', ':', '!', '?', ')']);
            if bare.strip_prefix('@').is_some_and(|n| is_name(n, ".-_")) {
                out.push((Piece::Person, span));
            } else {
                add_links(tok, at, &mut out);
            }
            continue;
        }
        if tok.strip_prefix('@').is_some_and(|n| is_name(n, ".-_")) {
            out.push((Piece::Person, span));
        } else if tok.strip_prefix('#').is_some_and(|n| is_name(n, "-_")) {
            out.push((Piece::Tag, span));
        } else if matches!(tok, "!" | "!!" | "!!!") {
            out.push((Piece::Priority, span));
        } else if tok.strip_prefix('^').is_some_and(|n| is_name(n, "-_")) {
            out.push((Piece::Anchor, span));
        } else if field(tok).is_some() {
            let colon = tok.find(':').expect("a field has a colon") + 1;
            out.push((Piece::Key, Span::new(at, at + colon)));
            let value = &tok[colon..];
            let kind = if value.starts_with("[[") {
                Piece::Link
            } else if value.starts_with('^') {
                Piece::Anchor
            } else {
                Piece::Value
            };
            out.push((kind, Span::new(at + colon, span.end)));
        } else {
            add_links(tok, at, &mut out);
        }
    }
    out
}

/// Whitespace-separated tokens, where a double-quoted run keeps its spaces.
///
/// The same split as the reference parser's `[^\s"]*"[^"]*"\S*|\S+`: a token
/// that reaches a quote before any whitespace extends to the closing quote and
/// then to the next whitespace; an unterminated quote is an ordinary word.
pub(crate) fn split(text: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let mut i = 0;
    let len = text.len();
    while i < len {
        let c = text[i..].chars().next().expect("i is a char boundary");
        if c.is_whitespace() {
            i += c.len_utf8();
            continue;
        }
        let start = i;
        let end = quoted_end(text, start).unwrap_or_else(|| word_end(text, start));
        out.push((start, &text[start..end]));
        i = end;
    }
    out
}

fn word_end(text: &str, from: usize) -> usize {
    text[from..]
        .char_indices()
        .find(|(_, c)| c.is_whitespace())
        .map_or(text.len(), |(j, _)| from + j)
}

fn quoted_end(text: &str, from: usize) -> Option<usize> {
    let rest = &text[from..];
    let open = rest.find(|c: char| c == '"' || c.is_whitespace())?;
    if !rest[open..].starts_with('"') {
        return None;
    }
    let close = rest[open + 1..].find('"')? + open + 1;
    Some(word_end(text, from + close + 1))
}

/// A name after `@`, `#` or `^`: letters, digits and the given punctuation.
fn is_name(s: &str, punct: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || punct.contains(c))
}

fn field(tok: &str) -> Option<(String, String)> {
    let (key, value) = tok.split_once(':')?;
    let mut chars = key.chars();
    let first = chars.next()?;
    if !first.is_alphabetic() || !chars.all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return None;
    }
    if value.is_empty() || value.starts_with("//") {
        return None;
    }
    let value = match value.strip_prefix('"').and_then(|v| v.strip_suffix('"')) {
        Some(inner) if value.len() >= 2 => inner,
        _ => value,
    };
    Some((key.to_lowercase(), value.to_string()))
}

/// Every well-formed link in a word, with its offset there. The span in each
/// returned link is relative to that offset: `0..len`.
pub fn links(word: &str) -> Vec<(usize, Link)> {
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(found) = word[from..].find("[[") {
        let at = from + found;
        match link_at(&word[at..]) {
            Some((len, link)) => {
                out.push((
                    at,
                    Link {
                        span: Span::new(0, len),
                        ..link
                    },
                ));
                from = at + len;
            }
            None => from = at + 1,
        }
    }
    out
}

/// `[[path]]` or `[[path^anchor]]` at the start of `s`, and its length.
fn link_at(s: &str) -> Option<(usize, Link)> {
    let inner = s.strip_prefix("[[")?;
    let path_len = inner.find([']', '^']).unwrap_or(inner.len());
    if path_len == 0 {
        return None;
    }
    let path = &inner[..path_len];
    let rest = &inner[path_len..];
    let (anchor, rest) = match rest.strip_prefix('^') {
        Some(after) => {
            let n = after
                .find(|c: char| !(c.is_alphanumeric() || c == '_' || c == '-'))
                .unwrap_or(after.len());
            if n == 0 {
                return None;
            }
            (Some(after[..n].to_string()), &after[n..])
        }
        None => (None, rest),
    };
    rest.strip_prefix("]]")?;
    let len = s.len() - rest.len() + 2;
    Some((
        len,
        Link {
            path: file_key(path),
            anchor,
            span: Span::new(0, len),
        },
    ))
}

/// A link's file as the workspace names it, before directories are applied.
pub(crate) fn file_key(path: &str) -> String {
    let path = path.trim();
    let path = path.strip_prefix("./").unwrap_or(path);
    if path.ends_with(".sprig") {
        path.to_string()
    } else {
        format!("{path}.sprig")
    }
}

/// A word as plain text (§4.9): each link loses its brackets. Anything else
/// that looks like one keeps them, so a writer can see it is not a link.
fn plain(word: &str) -> String {
    let mut out = String::with_capacity(word.len());
    let mut at = 0;
    for (start, link) in links(word) {
        out.push_str(&word[at..start]);
        out.push_str(&word[start + 2..start + link.span.end - 2]);
        at = start + link.span.end;
    }
    out.push_str(&word[at..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(text: &str) -> Vec<&str> {
        split(text).into_iter().map(|(_, w)| w).collect()
    }

    #[test]
    fn quoted_values_keep_their_spaces_and_lose_their_quotes() {
        let t = lift(r#"Visit  where:"41 Mill Street"  @dana"#, 0);
        assert_eq!(t.text, "Visit");
        assert_eq!(t.field("where").unwrap().value, "41 Mill Street");
        assert_eq!(t.people, ["dana"]);
    }

    #[test]
    fn a_quote_mid_token_extends_it_to_the_closing_quote() {
        assert_eq!(words(r#"a"b c"d"e f" g"#), [r#"a"b c"d"e"#, r#"f""#, "g"]);
    }

    #[test]
    fn an_unterminated_quote_is_an_ordinary_word() {
        assert_eq!(words(r#"say "hello world"#), ["say", r#""hello"#, "world"]);
        let t = lift(r#"note:"open ended"#, 0);
        assert_eq!(t.field("note").unwrap().value, r#""open"#);
        assert_eq!(t.text, "ended");
    }

    #[test]
    fn a_url_stays_a_word() {
        let t = lift("Read https://example.org/quote now", 0);
        assert!(t.fields.is_empty());
        assert_eq!(t.text, "Read https://example.org/quote now");
    }

    #[test]
    fn people_tags_priorities_and_anchors_are_lifted() {
        let t = lift("Move the sofa  @me @jo.k  #home  !!  ! ^sofa", 0);
        assert_eq!(t.text, "Move the sofa");
        assert_eq!(t.people, ["me", "jo.k"]);
        assert_eq!(t.tags, ["home"]);
        assert_eq!(t.priority, 2);
        assert_eq!(t.anchor().unwrap().name, "sofa");
    }

    #[test]
    fn near_misses_are_words() {
        let t = lift("@jo, #a.b !!!! ^ ^a.b @ # 12:30 x: :y", 0);
        assert!(t.people.is_empty() && t.tags.is_empty() && t.anchors.is_empty());
        assert_eq!(t.priority, 0);
        assert!(t.fields.is_empty());
        assert_eq!(t.text, "@jo, #a.b !!!! ^ ^a.b @ # 12:30 x: :y");
    }

    #[test]
    fn names_may_use_any_letters() {
        let t = lift("Kaffi  @jón  #heimili  ^bolli", 0);
        assert_eq!(t.people, ["jón"]);
        assert_eq!(t.tags, ["heimili"]);
        assert_eq!(t.anchor().unwrap().name, "bolli");
    }

    #[test]
    fn keys_are_case_insensitive_and_the_last_value_wins() {
        let t = lift("Quote  Due:2026-10-02  budget:64k  due:2026-10-09", 0);
        assert_eq!(t.field("due").unwrap().value, "2026-10-09");
        let keys: Vec<_> = t.effective_fields().iter().map(|f| &f.key).collect();
        assert_eq!(keys, ["due", "budget"]);
    }

    #[test]
    fn after_values_are_collected_in_order() {
        let t = lift(
            "Ship  after:^tests  after:[[lease^signed]]  after:[[kitchen]]",
            0,
        );
        let after: Vec<_> = t.after.iter().map(|a| a.value.as_str()).collect();
        assert_eq!(after, ["^tests", "[[lease^signed]]", "[[kitchen]]"]);
        assert!(t.fields.is_empty());
    }

    #[test]
    fn links_lose_their_brackets_in_text_and_keep_their_place() {
        let t = lift(
            "Read the notes in [[kitchen]] and [[./rooms/hall.sprig^door]]",
            10,
        );
        assert_eq!(
            t.text,
            "Read the notes in kitchen and ./rooms/hall.sprig^door"
        );
        assert_eq!(t.links.len(), 2);
        assert_eq!(t.links[0].path, "kitchen.sprig");
        assert_eq!(t.links[0].anchor, None);
        assert_eq!(t.links[0].span, Span::new(28, 39));
        assert_eq!(t.links[1].path, "rooms/hall.sprig");
        assert_eq!(t.links[1].anchor.as_deref(), Some("door"));
    }

    #[test]
    fn a_graft_uses_the_first_word_with_brackets() {
        let t = lift("[[kitchen]]  @teo", 0);
        assert_eq!(t.graft_link().unwrap().path, "kitchen.sprig");
        let t = lift("see[[a^]] [[b]]", 0);
        assert!(t.graft_link().is_none());
        let t = lift("Kitchen  @teo", 0);
        assert!(t.graft_link().is_none());
    }

    #[test]
    fn malformed_links_are_not_links() {
        for w in ["[[]]", "[[^x]]", "[[a^]]", "[[a", "[a]]", "[[a^b c]]"] {
            assert!(links(w).is_empty(), "{w}");
        }
    }

    #[test]
    fn pieces_name_what_an_editor_colours() {
        let text = "Sign  @dana #legal !! ^signed due:fri after:[[lease^terms]] see [[kitchen]]";
        let got: Vec<(&str, &str)> = pieces(text, 0, false)
            .into_iter()
            .map(|(k, s)| (k.as_str(), &text[s.start..s.end]))
            .collect();
        assert_eq!(
            got,
            [
                ("person", "@dana"),
                ("tag", "#legal"),
                ("priority", "!!"),
                ("anchor", "^signed"),
                ("key", "due:"),
                ("value", "fri"),
                ("key", "after:"),
                ("link", "[[lease^terms]]"),
                ("link", "[[kitchen]]"),
            ]
        );
        let note = "Ask @teo, and #nottag see [[kitchen]].";
        let got: Vec<&str> = pieces(note, 0, true)
            .into_iter()
            .map(|(_, s)| &note[s.start..s.end])
            .collect();
        assert_eq!(got, ["@teo,", "[[kitchen]]"]);
    }

    #[test]
    fn token_spans_point_into_the_file() {
        let src = "- Sign  ^signed  due:2026-10-05";
        let t = lift(&src[2..], 2);
        let a = t.anchor().unwrap();
        assert_eq!(&src[a.span.start..a.span.end], "^signed");
        let f = t.field("due").unwrap();
        assert_eq!(&src[f.span.start..f.span.end], "due:2026-10-05");
    }
}
