use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};

use crate::app::AppState;
use crate::app::config::presets::ConfigPresetTableRow;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

const SEP: &str = " │ ";

fn spaces(n: usize) -> String {
    " ".repeat(n)
}

fn truncate_display(s: &str, max_w: usize) -> String {
    if max_w == 0 {
        return String::new();
    }
    if s.width() <= max_w {
        return s.to_string();
    }
    let mut out = String::new();
    let mut w = 0usize;
    for ch in s.chars() {
        let cw = ch.width().unwrap_or(0);
        if w + cw > max_w.saturating_sub(1) {
            break;
        }
        out.push(ch);
        w += cw;
    }
    out.push('…');
    out
}

fn pad_or_trunc(s: &str, target_w: usize) -> String {
    let t = truncate_display(s, target_w);
    let pad = target_w.saturating_sub(t.width());
    format!("{t}{}", spaces(pad))
}

fn push_chunk(lines: &mut Vec<String>, chunk: &str, max_w: usize) {
    let mut rest = chunk;
    while !rest.is_empty() {
        let mut out = String::new();
        let mut w = 0usize;
        let mut consumed = 0usize;
        for ch in rest.chars() {
            let c = ch.width().unwrap_or(0);
            if w + c > max_w && !out.is_empty() {
                break;
            }
            if w + c > max_w && out.is_empty() {
                out.push(ch);
                consumed += ch.len_utf8();
                break;
            }
            out.push(ch);
            w += c;
            consumed += ch.len_utf8();
        }
        lines.push(out);
        rest = rest.get(consumed..).unwrap_or("");
    }
}

fn wrap_additional(s: &str, max_w: usize) -> Vec<String> {
    if max_w == 0 {
        return vec![String::new()];
    }
    if s.is_empty() {
        return vec![String::new()];
    }
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut cur_w = 0usize;
    for word in s.split_whitespace() {
        let ww = word.width();
        if ww > max_w {
            if !current.is_empty() {
                lines.push(std::mem::take(&mut current));
                cur_w = 0;
            }
            push_chunk(&mut lines, word, max_w);
            continue;
        }
        let need = if cur_w == 0 { ww } else { 1 + ww };
        if cur_w > 0 && cur_w + need > max_w {
            lines.push(std::mem::take(&mut current));
            cur_w = 0;
        }
        if cur_w > 0 {
            current.push(' ');
            cur_w += 1;
        }
        current.push_str(word);
        cur_w += ww;
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn column_widths(area_w: u16) -> (usize, usize, usize, usize) {
    let inner = area_w.saturating_sub(4) as usize;
    let sep_w = SEP.width() * 3;
    let c = 14usize;
    let l = 16usize;
    let d = 18usize;
    let add = inner.saturating_sub(c + l + d + sep_w).max(10);
    (c, l, d, add)
}

fn row_lines(
    row: &ConfigPresetTableRow,
    cw: usize,
    lw: usize,
    dw: usize,
    aw: usize,
    base: Style,
) -> Vec<Line<'static>> {
    let add_lines = wrap_additional(&row.additional, aw);
    let c0 = pad_or_trunc(&row.country, cw);
    let l0 = pad_or_trunc(&row.language, lw);
    let d0 = pad_or_trunc(&row.desktop, dw);
    let mut out: Vec<Line<'static>> = Vec::new();
    let first_add = add_lines.first().cloned().unwrap_or_default();
    out.push(Line::from(vec![
        Span::styled(c0, base),
        Span::styled(SEP.to_string(), base),
        Span::styled(l0, base),
        Span::styled(SEP.to_string(), base),
        Span::styled(d0, base),
        Span::styled(SEP.to_string(), base),
        Span::styled(first_add, base),
    ]));
    for extra in add_lines.iter().skip(1) {
        out.push(Line::from(vec![
            Span::styled(spaces(cw), base),
            Span::styled(SEP.to_string(), base),
            Span::styled(spaces(lw), base),
            Span::styled(SEP.to_string(), base),
            Span::styled(spaces(dw), base),
            Span::styled(SEP.to_string(), base),
            Span::styled(extra.clone(), base),
        ]));
    }
    out
}

pub fn draw(frame: &mut ratatui::Frame, app: &AppState, area: Rect) {
    let t = crate::render::theme::catppuccin_mocha();
    let header_style = Style::default().fg(t.accent).add_modifier(Modifier::BOLD);
    let base = Style::default().fg(t.text);
    let (cw, lw, dw, aw) = column_widths(area.width);

    let hdr = Line::from(vec![
        Span::styled(pad_or_trunc("Country", cw), header_style),
        Span::styled(SEP.to_string(), header_style),
        Span::styled(pad_or_trunc("Language", lw), header_style),
        Span::styled(SEP.to_string(), header_style),
        Span::styled(pad_or_trunc("Desktop / WM", dw), header_style),
        Span::styled(SEP.to_string(), header_style),
        Span::styled(truncate_display("Additional information", aw), header_style),
    ]);

    let mut items: Vec<ListItem<'static>> = Vec::new();
    for &gi in app.popup_visible_indices.iter() {
        let Some(row) = app.config_popup_rows.get(gi) else {
            continue;
        };
        let lines = row_lines(row, cw, lw, dw, aw, base);
        items.push(ListItem::new(Text::from(lines)));
    }

    let mut state = ListState::default();
    state.select(Some(app.popup_selected_visible));
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Presets (Enter to load) "),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Min(1),
        ])
        .split(area);

    let header_widget = ratatui::widgets::Paragraph::new(hdr).style(base);
    frame.render_widget(header_widget, chunks[0]);
    frame.render_stateful_widget(list, chunks[1], &mut state);
}
