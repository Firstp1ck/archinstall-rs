use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::ui::app;
use crate::ui::app::{AppState, Focus, Screen};

pub fn draw_content(frame: &mut Frame, app: &mut AppState, area: Rect) {
    match app.current_screen() {
        Screen::Locales => app::locales::draw_locales(frame, app, area),
        Screen::MirrorsRepos => app::mirrors::draw_mirrors_repos(frame, app, area),
        Screen::Disks => app::disks::draw_disks(frame, app, area),
        Screen::DiskEncryption => app::disk_encryption::draw_disk_encryption(frame, app, area),
        Screen::Bootloader => app::bootloader::draw_bootloader(frame, app, area),
        Screen::Hostname => app::hostname::draw_hostname(frame, app, area),
        Screen::RootPassword => app::root_password::draw_root_password(frame, app, area),
        Screen::UserAccount => app::user_account::draw_user_account(frame, app, area),
        Screen::ExperienceMode => app::experience_mode::draw_experience_mode(frame, app, area),
        Screen::SwapPartition => app::swap_partition::draw_swap_partition(frame, app, area),
        Screen::UnifiedKernelImages => {
            app::unified_kernel_images::draw_unified_kernel_images(frame, app, area)
        }
        Screen::Kernels => app::kernels::draw_kernels(frame, app, area),
        Screen::Audio => app::audio::draw_audio(frame, app, area),
        Screen::Timezone => app::timezone::draw_timezone(frame, app, area),
        Screen::AutomaticTimeSync => {
            app::automatic_time_sync::draw_automatic_time_sync(frame, app, area)
        }
        Screen::NetworkConfiguration => {
            app::network_configuration::draw_network_configuration(frame, app, area)
        }
        Screen::AdditionalPackages => {
            app::additional_packages::draw_additional_packages(frame, app, area)
        }
        Screen::SaveConfiguration => app::config::draw_configuration(frame, app, area),
        Screen::Install => app::install::draw_install(frame, app, area),
        _ => {
            let content_lines = vec![
                Line::from(Span::styled(
                    app.menu_entries[app.selected_index].label.to_string(),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(app.menu_entries[app.selected_index].content.clone()),
                Line::from(""),
                Line::from(Span::styled(
                    if app.current_screen() == Screen::Abort {
                        "[ Close/Exit ]"
                    } else {
                        "[ Continue ]"
                    },
                    match app.focus {
                        Focus::Content => Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                        _ => Style::default(),
                    },
                )),
            ];
            let content = Paragraph::new(content_lines)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Desicion Menu "),
                )
                .wrap(Wrap { trim: false });
            frame.render_widget(content, area);
        }
    }
}
