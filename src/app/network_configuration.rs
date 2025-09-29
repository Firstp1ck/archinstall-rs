use super::{AppState, Focus, PopupKind};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

impl AppState {
    #[allow(dead_code)]
    pub fn init_network_configuration(&mut self) {
        // nothing to init for now
    }

    pub fn current_network_label(&self) -> &'static str {
        match self.network_mode_index {
            0 => "Copy ISO network configuration",
            1 => "Manual configuration",
            _ => "Use NetworkManager",
        }
    }
}

pub fn draw_network_configuration(frame: &mut ratatui::Frame, app: &mut AppState, area: Rect) {
    let title = Span::styled(
        "Network Configuration",
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    );

    let mut lines: Vec<Line> = vec![Line::from(title), Line::from("")];

    let choices = [
        "Copy ISO network configuration (not implemented yet)",
        "Manual configuration",
        "Use NetworkManager (necessary for KDE and GNOME)",
        "Add interface", // visible regardless; only enabled when Manual is selected
    ];

    for (idx, label) in choices.iter().enumerate() {
        let is_focused_line = app.network_focus_index == idx;
        let is_active_line = is_focused_line && matches!(app.focus, Focus::Content);
        let is_selected = idx <= 2 && app.network_mode_index == idx;
        let bullet = if is_focused_line { "▶" } else { " " };
        let marker = if idx <= 2 {
            if is_selected { "[x]" } else { "[ ]" }
        } else if app.network_mode_index == 1 {
            "[+]"
        } else {
            "[ ]"
        };
        let bullet_style = if is_active_line {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let label_style = if is_active_line {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let mut text = format!("{marker} {label}");
        if idx == 3 {
            text.push_str("  (Manual mode only)");
        }
        let line = Line::from(vec![
            Span::styled(format!("{bullet} "), bullet_style),
            Span::styled(text, label_style),
        ]);
        lines.push(line);
    }

    // Continue index is 4 now
    let continue_style = if app.network_focus_index == 4 && matches!(app.focus, Focus::Content) {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("[ Continue ]", continue_style)));

    let content = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(match app.focus {
                    Focus::Content => " Desicion Menu (focused) ",
                    _ => " Desicion Menu ",
                }),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(content, area);

    // Info box lines for configured interfaces are rendered in sections.rs
}

impl AppState {
    pub fn open_network_interfaces_popup(&mut self) {
        // Read interfaces via `ip -o link` and filter out lo
        let mut items: Vec<String> = Vec::new();
        if let Ok(out) = std::process::Command::new("ip")
            .args(["-o", "link"])
            .output()
            && out.status.success()
        {
            let text = String::from_utf8_lossy(&out.stdout);
            for line in text.lines() {
                // format: "1: lo: ..." or "2: eth0: ..."
                if let Some(colon) = line.find(": ") {
                    let rest = &line[colon + 2..];
                    if let Some(name_end) = rest.find(":") {
                        let name = &rest[..name_end];
                        if name != "lo" {
                            items.push(name.to_string());
                        }
                    }
                }
            }
        }
        items.sort();
        items.dedup();
        if items.is_empty() {
            self.open_info_popup("No interfaces found".into());
            return;
        }
        self.popup_kind = Some(PopupKind::NetworkInterfaces);
        self.popup_open = true;
        self.popup_items = items;
        self.popup_visible_indices = (0..self.popup_items.len()).collect();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
    }

    pub fn open_network_mode_popup(&mut self) {
        self.popup_kind = Some(PopupKind::NetworkMode);
        self.popup_open = true;
        self.popup_items = vec!["DHCP (auto detect)".into(), "Static IP".into()];
        self.popup_visible_indices = (0..self.popup_items.len()).collect();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
    }

    pub fn open_network_ip_input(&mut self) {
        self.popup_kind = Some(PopupKind::NetworkIP);
        self.custom_input_buffer.clear();
        self.popup_open = true;
        self.popup_items.clear();
        self.popup_visible_indices.clear();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
    }

    pub fn open_network_gateway_input(&mut self) {
        self.popup_kind = Some(PopupKind::NetworkGateway);
        self.custom_input_buffer.clear();
        self.popup_open = true;
        self.popup_items.clear();
        self.popup_visible_indices.clear();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
    }

    pub fn open_network_dns_input(&mut self) {
        self.popup_kind = Some(PopupKind::NetworkDNS);
        self.custom_input_buffer.clear();
        self.popup_open = true;
        self.popup_items.clear();
        self.popup_visible_indices.clear();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
    }
}
