use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::{AppState, RepoSignOption, RepoSignature};

pub(super) fn render(frame: &mut Frame, app: &mut AppState, area: Rect) {
    let mut info_lines = vec![Line::from(Span::styled(
        "Info",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ))];

    let regions_count = app.mirrors_regions_selected.len();
    let opt_selected: Vec<&str> = app
        .optional_repos_selected
        .iter()
        .filter_map(|&i| app.optional_repos_options.get(i).map(|s| s.as_str()))
        .collect();
    let servers_count = app.mirrors_custom_servers.len();
    let repos_count = app.custom_repos.len();
    info_lines.push(Line::from(format!("Regions selected: {}", regions_count)));
    if regions_count > 0 {
        for &idx in app.mirrors_regions_selected.iter().take(5) {
            if let Some(name) = app.mirrors_regions_options.get(idx) {
                info_lines.push(Line::from(format!("- {}", name)));
            }
        }
        if regions_count > 5 {
            info_lines.push(Line::from("…"));
        }
    }
    info_lines.push(Line::from(format!(
        "Optional repos: {}",
        if opt_selected.is_empty() {
            "none".into()
        } else {
            opt_selected.join(", ")
        }
    )));
    if servers_count > 0 {
        info_lines.push(Line::from(format!("Custom servers ({}):", servers_count)));
        for s in app.mirrors_custom_servers.iter().take(3) {
            info_lines.push(Line::from(format!("- {}", s)));
        }
        if servers_count > 3 {
            info_lines.push(Line::from("…"));
        }
    } else {
        info_lines.push(Line::from("Custom servers: none"));
    }
    if repos_count > 0 {
        info_lines.push(Line::from("Custom repos:"));
        for repo in app.custom_repos.iter().take(3) {
            let sig = match repo.signature {
                RepoSignature::Never => "Never",
                RepoSignature::Optional => "Optional",
                RepoSignature::Required => "Required",
            };
            let signopt = match repo.sign_option {
                Some(RepoSignOption::TrustedOnly) => "TrustedOnly",
                Some(RepoSignOption::TrustedAll) => "TrustedAll",
                None => "-",
            };
            info_lines.push(Line::from(format!(
                "{} | {} | {} | {}",
                repo.name, repo.url, sig, signopt
            )));
        }
        if repos_count > 3 {
            info_lines.push(Line::from("…"));
        }
    } else {
        info_lines.push(Line::from("Custom repos: none"));
    }

    let mut desc_lines = vec![Line::from(Span::styled(
        "Description",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ))];
    desc_lines.push(Line::from("An Arch Linux mirror is a server that copies official repositories, letting users download packages and updates efficiently. Repositories are organized collections of software for installation. Custom mirrors store user-managed package copies, often for local or organizational use, while custom repositories let users or groups provide their own curated package sets."));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(area);

    let description = Paragraph::new(desc_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Description "),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(description, chunks[0]);

    let info = Paragraph::new(info_lines)
        .block(Block::default().borders(Borders::ALL).title(" Info "))
        .wrap(Wrap { trim: true });
    frame.render_widget(info, chunks[1]);
}
