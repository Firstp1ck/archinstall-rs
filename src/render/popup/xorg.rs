use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};

use crate::app::AppState;

pub fn draw(frame: &mut Frame, app: &mut AppState, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let xorg_items: Vec<ListItem> = app
        .popup_visible_indices
        .iter()
        .filter_map(|&vis| app.popup_items.get(vis).cloned())
        .map(|name| {
            let marker = if app.selected_xorg_types.contains(&name) {
                "[x]"
            } else {
                "[ ]"
            };
            ListItem::new(format!("{marker} {name}"))
        })
        .collect();
    let mut xorg_state = ratatui::widgets::ListState::default();
    xorg_state.select(Some(app.popup_selected_visible));
    let xorg_focused = !app.popup_packages_focus;
    let xorg_title = if xorg_focused {
        " Xorg (focused) "
    } else {
        " Xorg "
    };
    let xorg_highlight = if xorg_focused {
        Color::Yellow
    } else {
        Color::White
    };
    let xorg_list = List::new(xorg_items)
        .block(Block::default().borders(Borders::ALL).title(xorg_title))
        .highlight_style(
            Style::default()
                .fg(xorg_highlight)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    frame.render_stateful_widget(xorg_list, cols[0], &mut xorg_state);

    let selected_label = if let Some(&global_idx) =
        app.popup_visible_indices.get(app.popup_selected_visible)
        && let Some(name) = app.popup_items.get(global_idx)
    {
        name.clone()
    } else {
        String::new()
    };
    let pkgs: Vec<&str> = match selected_label.as_str() {
        "Xorg" => vec!["xorg-server"],
        _ => Vec::new(),
    };
    if !selected_label.is_empty() {
        let set = app
            .selected_xorg_packages
            .entry(selected_label.clone())
            .or_insert_with(|| pkgs.iter().map(|s| s.to_string()).collect());
        let pkg_items: Vec<ListItem> = pkgs
            .iter()
            .map(|name| {
                let marker = if set.contains(*name) { "[x]" } else { "[ ]" };
                ListItem::new(format!("{marker} {name}"))
            })
            .collect();

        let right_split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(cols[1]);

        let mut pkg_state = ratatui::widgets::ListState::default();
        let mut idx = app.popup_packages_selected_index;
        if idx >= pkg_items.len() {
            idx = pkg_items.len().saturating_sub(1);
        }
        pkg_state.select(Some(idx));
        let pkg_title = if app.popup_packages_focus {
            " Installed packages (focused) "
        } else {
            " Installed packages "
        };
        let pkg_highlight = if app.popup_packages_focus {
            Color::Yellow
        } else {
            Color::White
        };
        let pkg_list = List::new(pkg_items)
            .block(Block::default().borders(Borders::ALL).title(pkg_title))
            .highlight_style(
                Style::default()
                    .fg(pkg_highlight)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");
        frame.render_stateful_widget(pkg_list, right_split[0], &mut pkg_state);

        let drivers_all: Vec<(&str, bool)> = vec![
            (" Open Source Drivers ", false),
            ("intel-media-driver", true),
            ("libva-intel-driver", true),
            ("mesa", true),
            ("vulkan-intel", true),
            ("vulkan-nouveau", true),
            ("vulkan-radeon", true),
            ("xf86-video-amdgpu", true),
            ("xf86-video-ati", true),
            ("xf86-video-nouveau", true),
            ("xorg-server", true),
            ("xorg-xinit", true),
            (" Nvidia Drivers ", false),
            ("dkms", true),
            ("libva-nvidia-driver", true),
            (" Choose one ", false),
            ("nvidia-open-dkms", true),
            ("nvidia-dkms", true),
        ];
        let driver_items: Vec<ListItem> = drivers_all
            .iter()
            .map(|(name, selectable)| {
                if *selectable {
                    let marker = if app.selected_graphic_drivers.contains(*name) {
                        "[x]"
                    } else {
                        "[ ]"
                    };
                    ListItem::new(format!("{marker} {name}"))
                } else {
                    let title_span = Span::styled(
                        name.trim(),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    );
                    ListItem::new(Line::from(title_span))
                }
            })
            .collect();
        let mut drv_state = ratatui::widgets::ListState::default();
        let drv_len = driver_items.len();
        if drv_len == 0 {
            drv_state.select(None);
        } else {
            if app.popup_drivers_selected_index >= drv_len {
                app.popup_drivers_selected_index = drv_len - 1;
            }
            drv_state.select(Some(app.popup_drivers_selected_index));
        }
        let drv_title = if app.popup_drivers_focus {
            " Graphic Drivers (focused) "
        } else {
            " Graphic Drivers "
        };
        let drv_highlight = if app.popup_drivers_focus {
            Color::Yellow
        } else {
            Color::White
        };
        let drv_list = List::new(driver_items)
            .block(Block::default().borders(Borders::ALL).title(drv_title))
            .highlight_style(
                Style::default()
                    .fg(drv_highlight)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");
        frame.render_stateful_widget(drv_list, right_split[1], &mut drv_state);
    }
}
