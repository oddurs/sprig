//! Byte offsets, which the core speaks, and LSP positions, which count
//! characters in whatever unit the client and server agreed on.
//!
//! LSP's default unit is the UTF-16 code unit, a JavaScript inheritance;
//! clients that can count UTF-8 bytes say so, as rust-analyzer's do, and then
//! no conversion is needed past the start of the line.

use lsp_types::{Position, Range};
use sprig_core::Span;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Encoding {
    Utf8,
    Utf16,
}

/// Where each line of one text starts.
pub struct Lines<'a> {
    text: &'a str,
    starts: Vec<usize>,
}

impl<'a> Lines<'a> {
    pub fn new(text: &'a str) -> Lines<'a> {
        let mut starts = vec![0];
        starts.extend(text.match_indices('\n').map(|(i, _)| i + 1));
        Lines { text, starts }
    }

    /// The line's text from its start, without its line ending.
    fn line(&self, i: usize) -> &'a str {
        let start = self.starts[i];
        let end = self.starts.get(i + 1).map_or(self.text.len(), |&e| e - 1);
        self.text[start..end]
            .strip_suffix('\r')
            .unwrap_or(&self.text[start..end])
    }

    pub fn position(&self, offset: usize, enc: Encoding) -> Position {
        let offset = offset.min(self.text.len());
        let line = self.starts.partition_point(|&s| s <= offset) - 1;
        let before = &self.text[self.starts[line]..offset];
        let character = match enc {
            Encoding::Utf8 => before.len(),
            Encoding::Utf16 => before.encode_utf16().count(),
        };
        Position::new(line as u32, character as u32)
    }

    pub fn range(&self, span: Span, enc: Encoding) -> Range {
        Range::new(self.position(span.start, enc), self.position(span.end, enc))
    }

    /// The byte offset of a position, clamped to the end of its line, and to
    /// the end of the text for a line past the last: a client may ask about
    /// the place it is typing before the server has seen the keystroke.
    pub fn offset(&self, pos: Position, enc: Encoding) -> usize {
        let Some(&start) = self.starts.get(pos.line as usize) else {
            return self.text.len();
        };
        let line = self.line(pos.line as usize);
        let want = pos.character as usize;
        let mut units = 0;
        for (i, c) in line.char_indices() {
            if units >= want {
                return start + i;
            }
            units += match enc {
                Encoding::Utf8 => c.len_utf8(),
                Encoding::Utf16 => c.len_utf16(),
            };
        }
        start + line.len()
    }

    /// 1-based, as the core numbers lines.
    pub fn line_of(&self, offset: usize) -> usize {
        self.starts.partition_point(|&s| s <= offset)
    }

    /// The span of a 1-based line, without its line ending.
    pub fn line_span(&self, line: usize) -> Span {
        let start = self.starts[line - 1];
        Span::new(start, start + self.line(line - 1).len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf16_counts_surrogate_pairs_and_utf8_counts_bytes() {
        let text = "Þorrablót\r\n- 🍞 bread\n";
        let l = Lines::new(text);
        let at = text.find("bread").unwrap();
        assert_eq!(l.position(at, Encoding::Utf16), Position::new(1, 5));
        assert_eq!(l.position(at, Encoding::Utf8), Position::new(1, 7));
        assert_eq!(l.offset(Position::new(1, 5), Encoding::Utf16), at);
        assert_eq!(l.offset(Position::new(1, 7), Encoding::Utf8), at);
    }

    #[test]
    fn positions_past_the_end_are_clamped() {
        let text = "Plan\r\n- one\n";
        let l = Lines::new(text);
        assert_eq!(l.offset(Position::new(0, 99), Encoding::Utf16), 4);
        assert_eq!(l.offset(Position::new(9, 0), Encoding::Utf16), text.len());
        assert_eq!(l.line_span(2), Span::new(6, 11));
        assert_eq!(l.line_of(6), 2);
    }
}
