use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};

use crate::app::AppState;

pub fn draw(frame: &mut Frame, app: &mut AppState, area: Rect) {
    // Optional header for device list is not used here; this is a 2-col layout
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(28), Constraint::Min(1)])
        .split(area);

    let is_wm = |name: &str| -> bool {
        matches!(
            name,
            "Awesome" | "Bspwm" | "Enlightenment" | "Hyprland" | "Qtile" | "Sway" | "i3-wm"
        )
    };

    let mut de_visible_positions: Vec<usize> = Vec::new();
    let mut wm_visible_positions: Vec<usize> = Vec::new();
    for (vis_pos, &global_idx) in app.popup_visible_indices.iter().enumerate() {
        if let Some(name) = app.popup_items.get(global_idx) {
            if is_wm(name) {
                wm_visible_positions.push(vis_pos);
            } else {
                de_visible_positions.push(vis_pos);
            }
        }
    }

    let de_items: Vec<ListItem> = de_visible_positions
        .iter()
        .filter_map(|&vis_pos| app.popup_visible_indices.get(vis_pos).copied())
        .map(|global_idx| {
            let name = app.popup_items[global_idx].clone();
            let marker = if app.selected_desktop_envs.contains(&name) {
                "[x]"
            } else {
                "[ ]"
            };
            ListItem::new(format!("{} {}", marker, name))
        })
        .collect();
    let wm_items: Vec<ListItem> = wm_visible_positions
        .iter()
        .filter_map(|&vis_pos| app.popup_visible_indices.get(vis_pos).copied())
        .map(|global_idx| {
            let name = app.popup_items[global_idx].clone();
            let marker = if app.selected_desktop_envs.contains(&name) {
                "[x]"
            } else {
                "[ ]"
            };
            ListItem::new(format!("{} {}", marker, name))
        })
        .collect();

    let selected_vis = app.popup_selected_visible;
    let de_selected_local = de_visible_positions.iter().position(|&p| p == selected_vis);
    let wm_selected_local = wm_visible_positions.iter().position(|&p| p == selected_vis);

    let left_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(cols[0]);

    let mut de_state = ratatui::widgets::ListState::default();
    de_state.select(de_selected_local);
    let de_focused =
        !app.popup_packages_focus && !app.popup_login_focus && de_selected_local.is_some();
    let de_title = if de_focused {
        " Desktop Environments (focused) "
    } else {
        " Desktop Environments "
    };
    let de_highlight = if de_focused {
        Color::Yellow
    } else {
        Color::White
    };
    let de_list = List::new(de_items)
        .block(Block::default().borders(Borders::ALL).title(de_title))
        .highlight_style(
            Style::default()
                .fg(de_highlight)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    frame.render_stateful_widget(de_list, left_split[0], &mut de_state);

    let mut wm_state = ratatui::widgets::ListState::default();
    wm_state.select(wm_selected_local);
    let wm_focused =
        !app.popup_packages_focus && !app.popup_login_focus && wm_selected_local.is_some();
    let wm_title = if wm_focused {
        " Window Managers (focused) "
    } else {
        " Window Managers "
    };
    let wm_highlight = if wm_focused {
        Color::Yellow
    } else {
        Color::White
    };
    let wm_list = List::new(wm_items)
        .block(Block::default().borders(Borders::ALL).title(wm_title))
        .highlight_style(
            Style::default()
                .fg(wm_highlight)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    frame.render_stateful_widget(wm_list, left_split[1], &mut wm_state);

    let selected_label = if let Some(&global_idx) =
        app.popup_visible_indices.get(app.popup_selected_visible)
        && let Some(name) = app.popup_items.get(global_idx)
    {
        name.clone()
    } else {
        String::new()
    };
    let env_type = match selected_label.as_str() {
        "Awesome" | "Bspwm" | "Enlightenment" | "Hyprland" | "Qtile" | "Sway" | "i3-wm" => {
            "Window Manager"
        }
        _ => "Desktop Environment",
    };

    use crate::common::env_packages::get_env_packages;
    let packages = get_env_packages(selected_label.as_str());

    let selected_set = app
        .selected_env_packages
        .entry(selected_label.clone())
        .or_insert_with(|| packages.iter().map(|s| s.to_string()).collect());

    let pkg_items: Vec<ListItem> = packages
        .iter()
        .map(|name| {
            let marker = if selected_set.contains(*name) {
                "[x]"
            } else {
                "[ ]"
            };
            ListItem::new(format!("{} {}", marker, name))
        })
        .collect();

    let right_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(cols[1]);

    let pkg_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(right_cols[0]);

    let mut pkg_state = ratatui::widgets::ListState::default();
    let pkg_len = pkg_items.len();
    if pkg_len > 0 {
        if app.popup_packages_selected_index >= pkg_len {
            app.popup_packages_selected_index = pkg_len - 1;
        }
        pkg_state.select(Some(app.popup_packages_selected_index));
    } else {
        pkg_state.select(None);
    }
    let pkg_title = if app.popup_packages_focus {
        format!(" Installed packages — {} (focused) ", env_type)
    } else {
        format!(" Installed packages — {} ", env_type)
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
    frame.render_stateful_widget(pkg_list, pkg_cols[0], &mut pkg_state);

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
                ListItem::new(format!("{} {}", marker, name))
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
    frame.render_stateful_widget(drv_list, pkg_cols[1], &mut drv_state);

    let login_managers = [
        "none".to_string(),
        "gdm".to_string(),
        "lightdm-gtk-greeter".to_string(),
        "lightdm-slick-greeter".to_string(),
        "ly".to_string(),
        "sddm".to_string(),
    ];
    let default_lm: Option<&str> = match selected_label.as_str() {
        "GNOME" => Some("gdm"),
        "KDE Plasma" | "Hyprland" | "Cutefish" | "Lxqt" => Some("sddm"),
        "Budgie" => Some("lightdm-slick-greeter"),
        "Bspwm" | "Cinnamon" | "Deepin" | "Enlightenment" | "Mate" | "Qtile" | "Sway" | "Xfce4"
        | "i3-wm" => Some("lightdm-gtk-greeter"),
        _ => None,
    };
    let selected_lm = if app.login_manager_user_set {
        app.selected_login_manager.clone()
    } else {
        app.selected_login_manager
            .clone()
            .or_else(|| default_lm.map(|s| s.to_string()))
    };

    let lm_items: Vec<ListItem> = login_managers
        .iter()
        .map(|name| {
            let is_selected = match &selected_lm {
                Some(s) => s == name,
                None => name == "none",
            };
            let marker = if is_selected { "[x]" } else { "[ ]" };
            ListItem::new(format!("{} {}", marker, name))
        })
        .collect();
    let mut lm_state = ratatui::widgets::ListState::default();
    lm_state.select(Some(
        app.popup_login_selected_index
            .min(login_managers.len().saturating_sub(1)),
    ));
    let lm_title = if app.popup_login_focus {
        " Login Manager (focused) "
    } else {
        " Login Manager "
    };
    let lm_highlight = if app.popup_login_focus {
        Color::Yellow
    } else {
        Color::White
    };
    let lm_list = List::new(lm_items)
        .block(Block::default().borders(Borders::ALL).title(lm_title))
        .highlight_style(
            Style::default()
                .fg(lm_highlight)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    frame.render_stateful_widget(lm_list, right_cols[1], &mut lm_state);
}
