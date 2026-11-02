//! One place that decides how output looks: widths, colour, and the
//! machine-readable form.
//!
//! Commands build rows of cells and this module prints them, as an aligned
//! table for a person or, with `--plain`, as tab-separated fields with a ref
//! first and no colour, whose column order is a promise (cairn 0073), like
//! git's `--porcelain`. Colour follows ripgrep and the NO_COLOR convention:
//! `--color` wins, then a non-empty `NO_COLOR` turns it off, and otherwise it
//! is on only for a terminal. It only ever repeats what a mark already says.

use std::io::IsTerminal;

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::args::ColorChoice;

/// A style a cell may carry. Marks take the site's palette, one hue each, as
/// 256-colour codes; everything else is plain or dimmed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    Plain,
    Dim,
    Bold,
    /// A mark's character, coloured by its meaning.
    Mark(char),
    Error,
    Warning,
}

impl Style {
    fn sgr(self) -> Option<&'static str> {
        Some(match self {
            Style::Plain => return None,
            Style::Dim => "2",
            Style::Bold => "1",
            Style::Error => "1;31",
            Style::Warning => "1;33",
            // tokens.mjs: doing 70°, done 150°, later 255°, question 300°,
            // answer 350°, group 195°, graft 38°; to do is the text colour
            // and dropped the muted one.
            Style::Mark('~') => "38;5;178",
            Style::Mark('x') => "38;5;71",
            Style::Mark('>') => "38;5;68",
            Style::Mark('?') => "38;5;133",
            Style::Mark('=') => "38;5;168",
            Style::Mark('#') => "38;5;37",
            Style::Mark('+') => "38;5;172",
            Style::Mark('/') => "2",
            Style::Mark(_) => return None,
        })
    }
}

/// How this run prints.
#[derive(Clone, Copy, Debug)]
pub struct Out {
    pub color: bool,
    /// Columns to fit in; `None` means never truncate.
    pub width: Option<usize>,
    pub plain: bool,
}

impl Out {
    pub fn new(choice: ColorChoice, plain: bool) -> Out {
        let tty = std::io::stdout().is_terminal();
        let no_color = std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty());
        let color = !plain
            && match choice {
                ColorChoice::Always => true,
                ColorChoice::Never => false,
                ColorChoice::Auto => tty && !no_color,
            };
        // COLUMNS first, so a test or a pipe can ask for a width; then the
        // terminal's own; and no limit when there is neither.
        let width = std::env::var("COLUMNS")
            .ok()
            .and_then(|c| c.parse::<usize>().ok())
            .filter(|&w| w > 0)
            .or_else(|| {
                tty.then(|| terminal_size::terminal_size().map(|(w, _)| usize::from(w.0)))
                    .flatten()
            });
        Out {
            color,
            width: if plain { None } else { width },
            plain,
        }
    }

    pub fn paint(&self, text: &str, style: Style) -> String {
        match (self.color, style.sgr()) {
            (true, Some(code)) if !text.is_empty() => format!("\x1b[{code}m{text}\x1b[0m"),
            _ => text.to_string(),
        }
    }
}

/// One cell: text and how it looks.
#[derive(Clone, Debug)]
pub struct Cell {
    pub text: String,
    pub style: Style,
}

impl Cell {
    pub fn new(text: impl Into<String>, style: Style) -> Cell {
        Cell {
            text: text.into(),
            style,
        }
    }

    pub fn plain(text: impl Into<String>) -> Cell {
        Cell::new(text, Style::Plain)
    }
}

/// A row: its ref, for `--plain`, and its cells, each a list of pieces that
/// print side by side.
#[derive(Clone, Debug)]
pub struct Row {
    pub key: String,
    pub cells: Vec<Vec<Cell>>,
}

/// Rows, with the columns that give way, in order and down to a minimum,
/// when the table is wider than the terminal.
pub struct Table {
    pub rows: Vec<Row>,
    pub shrink: Vec<(usize, usize)>,
    /// Columns cut from the left, like paths, whose end says the most.
    pub keep_end: Vec<usize>,
}

pub fn width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

/// Cuts `s` to at most `max` columns, ending in an ellipsis when cut, and
/// never in the middle of a character.
pub fn clip(s: &str, max: usize) -> String {
    if width(s) <= max {
        return s.to_string();
    }
    if max == 0 {
        return String::new();
    }
    let mut out = String::new();
    let mut used = 0;
    for c in s.chars() {
        let w = c.width().unwrap_or(0);
        if used + w > max - 1 {
            break;
        }
        out.push(c);
        used += w;
    }
    out.push('…');
    out
}

/// Cuts `s` from the left, keeping its end: for paths, whose nearest part
/// says the most.
pub fn clip_left(s: &str, max: usize) -> String {
    if width(s) <= max {
        return s.to_string();
    }
    let rev: String = s.chars().rev().collect();
    let cut: String = clip(&rev, max).chars().rev().collect();
    // clip put its ellipsis at the end of the reversed text: now the start.
    cut
}

fn cell_text(cell: &[Cell]) -> String {
    cell.iter().map(|c| c.text.as_str()).collect()
}

impl Out {
    /// Prints a table: aligned columns two spaces apart, fitted to the width,
    /// or with `--plain`, the key and every cell separated by tabs.
    pub fn table(&self, t: &Table) -> Vec<String> {
        if self.plain {
            return t
                .rows
                .iter()
                .map(|r| {
                    let mut fields = vec![r.key.clone()];
                    fields.extend(
                        r.cells
                            .iter()
                            .map(|c| cell_text(c).replace(['\t', '\n'], " ")),
                    );
                    fields.join("\t")
                })
                .collect();
        }
        let ncols = t.rows.iter().map(|r| r.cells.len()).max().unwrap_or(0);
        let mut widths: Vec<usize> = (0..ncols)
            .map(|i| {
                t.rows
                    .iter()
                    .filter_map(|r| r.cells.get(i))
                    .map(|c| width(&cell_text(c)))
                    .max()
                    .unwrap_or(0)
            })
            .collect();
        if let Some(limit) = self.width {
            let gaps = 2 * ncols.saturating_sub(1);
            for &(col, min) in &t.shrink {
                let total: usize = widths.iter().sum::<usize>() + gaps;
                if total > limit && col < ncols {
                    widths[col] = widths[col].saturating_sub(total - limit).max(min);
                }
            }
        }
        t.rows
            .iter()
            .map(|r| {
                let mut line = String::new();
                for (i, cell) in r.cells.iter().enumerate() {
                    let room = widths[i];
                    let full = cell_text(cell);
                    let mut used = 0;
                    if width(&full) > room {
                        // Cut the text, keeping the first piece's style.
                        let style = cell.first().map_or(Style::Plain, |c| c.style);
                        let cut = if t.keep_end.contains(&i) {
                            clip_left(&full, room)
                        } else {
                            clip(&full, room)
                        };
                        used = width(&cut);
                        line.push_str(&self.paint(&cut, style));
                    } else {
                        for c in cell {
                            line.push_str(&self.paint(&c.text, c.style));
                            used += width(&c.text);
                        }
                    }
                    if i + 1 < r.cells.len() {
                        line.push_str(&" ".repeat(room - used + 2));
                    }
                }
                line.trim_end().to_string()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clipping_counts_columns_and_never_splits_a_character() {
        assert_eq!(clip("Kiln & Crumb", 20), "Kiln & Crumb");
        assert_eq!(clip("Kiln & Crumb", 6), "Kiln …");
        assert_eq!(clip("日本語のテキスト", 7), "日本語…");
        assert_eq!(width(&clip("日本語のテキスト", 7)), 7);
        assert_eq!(clip_left("Opening week › Soft opening", 12), "…oft opening");
    }

    #[test]
    fn plain_rows_are_tab_separated_with_the_key_first() {
        let out = Out {
            color: true,
            width: Some(10),
            plain: true,
        };
        let t = Table {
            rows: vec![Row {
                key: "lease^signed".into(),
                cells: vec![
                    vec![Cell::new("-", Style::Mark('-'))],
                    vec![Cell::plain("Sign the lease")],
                ],
            }],
            shrink: vec![(1, 4)],
            keep_end: vec![],
        };
        assert_eq!(out.table(&t), ["lease^signed\t-\tSign the lease"]);
    }

    #[test]
    fn colour_repeats_the_mark_and_can_be_turned_off() {
        let on = Out {
            color: true,
            width: None,
            plain: false,
        };
        let off = Out {
            color: false,
            width: None,
            plain: false,
        };
        assert_eq!(on.paint("~", Style::Mark('~')), "\x1b[38;5;178m~\x1b[0m");
        assert_eq!(on.paint("-", Style::Mark('-')), "-");
        assert_eq!(off.paint("~", Style::Mark('~')), "~");
    }
}
