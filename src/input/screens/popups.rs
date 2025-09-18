use crate::ui::app::AppState;

// Extracted from screens.rs: popup vertical navigation
pub(crate) fn popup_move_up(app: &mut AppState) {
    if matches!(
        app.popup_kind,
        Some(crate::ui::app::PopupKind::DesktopEnvSelect)
    ) && !app.popup_visible_indices.is_empty()
    {
        let is_wm_name = |s: &str| -> bool {
            matches!(
                s,
                "Awesome" | "Bspwm" | "Enlightenment" | "Hyprland" | "Qtile" | "Sway" | "i3-wm"
            )
        };
        let mut de_positions: Vec<usize> = Vec::new();
        let mut wm_positions: Vec<usize> = Vec::new();
        for (vis_pos, &gi) in app.popup_visible_indices.iter().enumerate() {
            if let Some(name) = app.popup_items.get(gi) {
                if is_wm_name(name) {
                    wm_positions.push(vis_pos);
                } else {
                    de_positions.push(vis_pos);
                }
            }
        }
        let current_vis = app.popup_selected_visible;
        let current_is_wm = if let Some(&gi) = app.popup_visible_indices.get(current_vis)
            && let Some(name) = app.popup_items.get(gi)
        {
            is_wm_name(name)
        } else {
            false
        };
        if current_is_wm {
            if let Some(idx) = wm_positions.iter().position(|&p| p == current_vis) {
                if idx > 0 {
                    app.popup_selected_visible = wm_positions[idx - 1];
                } else if let Some(&last_de) = de_positions.last() {
                    app.popup_selected_visible = last_de;
                }
            }
        } else if let Some(idx) = de_positions.iter().position(|&p| p == current_vis) {
            if idx > 0 {
                app.popup_selected_visible = de_positions[idx - 1];
            } else if let Some(&last_wm) = wm_positions.last() {
                app.popup_selected_visible = last_wm;
            }
        }
        return;
    }
    if app.popup_selected_visible == 0 {
        if !app.popup_visible_indices.is_empty() {
            app.popup_selected_visible = app.popup_visible_indices.len() - 1;
        }
    } else {
        app.popup_selected_visible = app.popup_selected_visible.saturating_sub(1);
    }
}

pub(crate) fn popup_move_down(app: &mut AppState) {
    if matches!(
        app.popup_kind,
        Some(crate::ui::app::PopupKind::DesktopEnvSelect)
    ) && !app.popup_visible_indices.is_empty()
    {
        let is_wm_name = |s: &str| -> bool {
            matches!(
                s,
                "Awesome" | "Bspwm" | "Enlightenment" | "Hyprland" | "Qtile" | "Sway" | "i3-wm"
            )
        };
        let mut de_positions: Vec<usize> = Vec::new();
        let mut wm_positions: Vec<usize> = Vec::new();
        for (vis_pos, &gi) in app.popup_visible_indices.iter().enumerate() {
            if let Some(name) = app.popup_items.get(gi) {
                if is_wm_name(name) {
                    wm_positions.push(vis_pos);
                } else {
                    de_positions.push(vis_pos);
                }
            }
        }
        let current_vis = app.popup_selected_visible;
        let current_is_wm = if let Some(&gi) = app.popup_visible_indices.get(current_vis)
            && let Some(name) = app.popup_items.get(gi)
        {
            is_wm_name(name)
        } else {
            false
        };
        if current_is_wm {
            if let Some(idx) = wm_positions.iter().position(|&p| p == current_vis) {
                if idx + 1 < wm_positions.len() {
                    app.popup_selected_visible = wm_positions[idx + 1];
                } else if let Some(&first_de) = de_positions.first() {
                    app.popup_selected_visible = first_de;
                }
            }
        } else if let Some(idx) = de_positions.iter().position(|&p| p == current_vis) {
            if idx + 1 < de_positions.len() {
                app.popup_selected_visible = de_positions[idx + 1];
            } else if let Some(&first_wm) = wm_positions.first() {
                app.popup_selected_visible = first_wm;
            }
        }
        return;
    }
    if !app.popup_visible_indices.is_empty() {
        app.popup_selected_visible =
            (app.popup_selected_visible + 1) % app.popup_visible_indices.len();
    }
}
