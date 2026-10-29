//! Drawing the TUI: one function from [`App`] to a frame, used by the
//! terminal and by `--screenshot` alike, so a snapshot is what a person sees.
//!
//! Colour only repeats what a mark already says (spec/tools.md T4); with it
//! stripped, as in a screenshot, nothing is lost.

use ratatui::Frame;
use ratatui::backend::TestBackend;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use unicode_width::UnicodeWidthStr;

use super::app::{App, HELP, Mode, Row, View};
use crate::out::clip;

/// Below this the detail pane hides, so the list keeps the width it needs.
const WIDE: u16 = 80;

pub fn draw(f: &mut Frame, app: &App) {
    let [top, body, bottom] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .areas(f.area());
    header(f, top, app);
    if body.width >= WIDE {
        // A third of the screen, so the tree keeps most of a narrow one.
        let pane = (body.width / 3).clamp(26, 44);
        let [list, detail] =
            Layout::horizontal([Constraint::Min(1), Constraint::Length(pane)]).areas(body);
        rows(f, list, app);
        details(f, detail, app);
    } else {
        rows(f, body, app);
    }
    status(f, bottom, app);
    if app.mode == Mode::Help {
        help(f, body);
    }
}

fn header(f: &mut Frame, area: Rect, app: &App) {
    let mut spans = vec![Span::styled(
        " sprig ",
        Style::new().add_modifier(Modifier::BOLD),
    )];
    for (i, v) in View::ALL.iter().enumerate() {
        let style = if *v == app.view {
            Style::new().add_modifier(Modifier::REVERSED)
        } else {
            Style::new().add_modifier(Modifier::DIM)
        };
        spans.push(Span::raw(" "));
        spans.push(Span::styled(format!("{} {}", i + 1, v.name()), style));
    }
    if !app.filter.is_empty() {
        spans.push(Span::raw("   "));
        spans.push(Span::styled(
            format!("/{}", app.filter),
            Style::new().fg(Color::Cyan),
        ));
    }
    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn mark_style(c: char) -> Style {
    match c {
        '~' => Style::new().fg(Color::Yellow),
        'x' => Style::new().fg(Color::Green),
        '?' => Style::new().fg(Color::Magenta),
        '/' | '>' => Style::new().add_modifier(Modifier::DIM),
        '#' | '+' => Style::new().fg(Color::Blue),
        _ => Style::new(),
    }
}

fn rows(f: &mut Frame, area: Rect, app: &App) {
    let height = area.height as usize;
    if app.rows.is_empty() {
        let msg = if app.filter.is_empty() {
            "Nothing here"
        } else {
            "Nothing matches the filter"
        };
        f.render_widget(
            Paragraph::new(format!("  {msg}")).style(Style::new().add_modifier(Modifier::DIM)),
            area,
        );
        return;
    }
    // Keep the cursor on screen, a few rows from the bottom where possible.
    let offset = (app.cursor + 3)
        .saturating_sub(height)
        .min(app.rows.len().saturating_sub(height));
    let width = area.width as usize;
    let lines: Vec<Line> = app
        .rows
        .iter()
        .enumerate()
        .skip(offset)
        .take(height)
        .map(|(i, r)| row_line(r, width, i == app.cursor))
        .collect();
    f.render_widget(Paragraph::new(lines), area);
}

fn row_line(r: &Row, width: usize, selected: bool) -> Line<'static> {
    let dim = Style::new().add_modifier(Modifier::DIM);
    if r.heading && r.at.is_none() {
        let text = clip(&format!("  {}", r.label), width);
        let style = Style::new().add_modifier(Modifier::BOLD);
        return Line::from(Span::styled(
            text,
            if selected {
                style.add_modifier(Modifier::REVERSED)
            } else {
                style
            },
        ));
    }
    // The cursor is a character as well as a highlight, so a screenshot,
    // which keeps no styles, still shows where it is.
    let cursor = if selected { "›" } else { " " };
    let indent = "  ".repeat(r.depth);
    let fold = if r.folded { "▸" } else { " " };
    let lead = format!("{cursor}{indent}{fold}");
    let info = if r.info.is_empty() {
        String::new()
    } else {
        format!("  {}", r.info)
    };
    let room = width.saturating_sub(lead.width() + 2 + info.width() + 1);
    let label = clip(&r.label, room);
    let context = if r.context.is_empty() || label.width() + 3 >= room {
        String::new()
    } else {
        clip(&format!("  {}", r.context), room - label.width())
    };
    let used = lead.width() + 2 + label.width() + context.width() + info.width();
    let pad = " ".repeat(width.saturating_sub(used + 1));
    let base = if selected {
        Style::new().add_modifier(Modifier::REVERSED)
    } else {
        Style::new()
    };
    let label_style = if r.heading {
        base.add_modifier(Modifier::BOLD)
    } else if r.blocked {
        base.add_modifier(Modifier::DIM)
    } else {
        base
    };
    Line::from(vec![
        Span::styled(lead, base),
        Span::styled(r.glyph.to_string(), base.patch(mark_style(r.glyph))),
        Span::styled(" ", base),
        Span::styled(label, label_style),
        Span::styled(context, base.patch(dim)),
        Span::styled(pad, base),
        Span::styled(info, base.patch(dim)),
        Span::styled(" ", base),
    ])
}

fn details(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::new()
        .borders(Borders::LEFT)
        .border_style(Style::new().add_modifier(Modifier::DIM));
    let Some(d) = app.detail() else {
        f.render_widget(block, area);
        return;
    };
    let dim = Style::new().add_modifier(Modifier::DIM);
    let mut lines = vec![
        Line::from(Span::styled(
            d.label,
            Style::new().add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(d.place, dim)),
        Line::raw(""),
    ];
    for (k, v) in d.lines {
        lines.push(Line::from(vec![
            Span::styled(format!("{k:<9}"), dim),
            Span::raw(v),
        ]));
    }
    if !d.waits.is_empty() {
        lines.push(Line::raw(""));
        lines.push(Line::from(Span::styled("waits on", dim)));
        for w in d.waits {
            lines.push(Line::raw(format!("  {w}")));
        }
    }
    if !d.notes.is_empty() {
        lines.push(Line::raw(""));
        for n in d.notes {
            lines.push(Line::raw(n));
            lines.push(Line::raw(""));
        }
    }
    f.render_widget(
        Paragraph::new(lines)
            .block(block.padding(ratatui::widgets::Padding::horizontal(1)))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn status(f: &mut Frame, area: Rect, app: &App) {
    let width = area.width as usize;
    let left = match &app.mode {
        Mode::Filter => format!(" /{}▏", app.filter),
        Mode::Add { child, text } => {
            format!(" add {}: {text}▏", if *child { "a child" } else { "after" })
        }
        _ => {
            let (done, total, ready, waiting) = app.counts();
            let mut s = format!(" {done}/{total} done · {ready} ready · {waiting} waiting");
            let errors = app.errors();
            if errors > 0 {
                s.push_str(&format!(
                    " · {errors} error{}",
                    if errors == 1 { "" } else { "s" }
                ));
            }
            s
        }
    };
    let right = match (&app.mode, &app.flash) {
        (Mode::Normal, Some(flash)) => flash.clone(),
        (Mode::Filter, _) => "Enter keeps · Esc clears".into(),
        (Mode::Add { .. }, _) => "Enter adds · Esc cancels".into(),
        _ => "? help".into(),
    };
    let left = clip(&left, width);
    let right = clip(&right, width.saturating_sub(left.width() + 3));
    let pad = " ".repeat(width.saturating_sub(left.width() + right.width() + 1));
    let style = Style::new().add_modifier(Modifier::REVERSED);
    f.render_widget(
        Paragraph::new(Line::from(format!("{left}{pad}{right} "))).style(style),
        area,
    );
}

fn help(f: &mut Frame, area: Rect) {
    let w = 64.min(area.width);
    let h = (HELP.len() as u16 + 4).min(area.height);
    let r = Rect::new(
        area.x + (area.width - w) / 2,
        area.y + (area.height - h) / 2,
        w,
        h,
    );
    let lines: Vec<Line> = HELP
        .iter()
        .map(|(k, what)| {
            Line::from(vec![
                Span::styled(
                    format!(" {k:<10}"),
                    Style::new().add_modifier(Modifier::BOLD),
                ),
                Span::raw(clip(what, (w as usize).saturating_sub(14))),
            ])
        })
        .collect();
    f.render_widget(Clear, r);
    f.render_widget(
        Paragraph::new(lines).block(
            Block::bordered()
                .title(" keys · any key closes ")
                .padding(ratatui::widgets::Padding::vertical(1)),
        ),
        r,
    );
}

/// One frame as text, with no terminal: the seam `--screenshot` and the
/// snapshot tests use.
pub fn render_to_string(app: &App, width: u16, height: u16) -> String {
    let mut terminal =
        ratatui::Terminal::new(TestBackend::new(width, height)).expect("a test backend draws");
    terminal
        .draw(|f| draw(f, app))
        .expect("a test backend draws");
    let buf = terminal.backend().buffer();
    let mut out = String::new();
    for y in 0..height {
        let mut line = String::new();
        for x in 0..width {
            line.push_str(buf[(x, y)].symbol());
        }
        out.push_str(line.trim_end());
        out.push('\n');
    }
    out
}
