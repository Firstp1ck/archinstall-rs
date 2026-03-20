use crate::app::{AppState, Focus, Screen};
use crate::core::storage::planner::StoragePlanner;

pub(crate) fn move_disks_up(app: &mut AppState) {
    if app.current_screen() == Screen::Disks && app.focus == Focus::Content {
        if app.disks_focus_index == 0 {
            app.disks_focus_index = 4;
        } else {
            app.disks_focus_index -= 1;
        }
    }
}
pub(crate) fn move_disks_down(app: &mut AppState) {
    if app.current_screen() == Screen::Disks && app.focus == Focus::Content {
        app.disks_focus_index = (app.disks_focus_index + 1) % 5;
    }
}
pub(crate) fn change_disks_value(app: &mut AppState, _next: bool) {
    if app.current_screen() == Screen::Disks && app.focus == Focus::Content {
        // Selecting a partitioning mode should happen explicitly on Enter.
        // Left/Right on this screen do not implicitly change the selected mode.
    }
}

pub(crate) fn handle_enter_disks(app: &mut AppState) {
    if app.disks_focus_index <= 2 {
        app.disks_mode_index = app.disks_focus_index;
        if app.disks_mode_index == 0 || app.disks_mode_index == 1 {
            app.open_disks_device_list();
        } else if app.disks_mode_index == 2 {
            match StoragePlanner::compile(app) {
                Ok(plan) => {
                    let mount_summary: Vec<String> = plan
                        .mounts
                        .iter()
                        .map(|m| {
                            if m.is_swap {
                                format!("  swap: {}", m.source)
                            } else {
                                format!("  {} -> {} ({})", m.source, m.target, m.fstype)
                            }
                        })
                        .collect();
                    app.open_info_popup(format!(
                        "Pre-mounted mode: detected {} mount(s):\n{}",
                        plan.mounts.len(),
                        mount_summary.join("\n")
                    ));
                }
                Err(errors) => {
                    let msg = errors
                        .iter()
                        .map(|e| e.message.as_str())
                        .collect::<Vec<_>>()
                        .join("\n- ");
                    app.open_info_popup(format!(
                        "Pre-mounted mode validation failed:\n- {msg}"
                    ));
                }
            }
        }
    } else if app.disks_focus_index == 3 {
        // Btrfs subvolume preset selector (only active for automatic mode)
        if app.disks_mode_index == 0 {
            app.open_btrfs_subvolume_preset_popup();
        }
    } else if app.disks_focus_index == 4 {
        // Validate storage plan before advancing from the Disks screen
        if (app.disks_mode_index == 1 || app.disks_mode_index == 2)
            && let Err(errors) = StoragePlanner::compile(app)
        {
            let msg = errors
                .iter()
                .map(|e| e.message.as_str())
                .collect::<Vec<_>>()
                .join("\n- ");
            app.open_info_popup(format!("Partition layout issues:\n- {msg}"));
            return;
        }
        super::common::advance(app);
    }
}
