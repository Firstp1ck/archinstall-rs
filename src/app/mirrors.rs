use std::process::Command;

use super::{AppState, CustomRepo, PopupKind, RepoSignOption, RepoSignature, Screen};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

#[derive(Debug)]
pub enum MirrorsLoadError {
    ListCountries,
}

impl std::fmt::Display for MirrorsLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MirrorsLoadError::ListCountries => write!(f, "failed to list countries via reflector"),
        }
    }
}

impl std::error::Error for MirrorsLoadError {}

impl AppState {
    pub fn load_mirrors_options(&mut self) -> Result<(), MirrorsLoadError> {
        if self.mirrors_loaded {
            return Ok(());
        }
        if let Ok(output) = Command::new("reflector").arg("--list-countries").output()
            && output.status.success()
        {
            let text = String::from_utf8_lossy(&output.stdout);
            let mut items: Vec<String> = text
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            items.sort();
            self.mirrors_regions_options = items;
        }

        // Fallback small list when reflector is unavailable (Windows/dry-run)
        if (cfg!(windows) || self.dry_run) && self.mirrors_regions_options.is_empty() {
            self.mirrors_regions_options = vec![
                "United States          US   186".into(),
                "Germany                DE   120".into(),
                "France                 FR    80".into(),
                "United Kingdom         GB    75".into(),
            ];
        }

        self.mirrors_loaded = true;
        Ok(())
    }

    pub fn open_mirrors_regions_popup(&mut self) {
        if self.current_screen() != Screen::MirrorsRepos || self.focus != super::Focus::Content {
            return;
        }
        let _ = self.load_mirrors_options();
        self.popup_kind = Some(PopupKind::MirrorsRegions);
        self.popup_items = self.mirrors_regions_options.clone();
        self.popup_search_query.clear();
        self.popup_in_search = false;
        self.popup_visible_indices = (0..self.popup_items.len()).collect();
        self.popup_selected_visible = 0;
        self.popup_open = true;
    }

    pub fn open_optional_repos_popup(&mut self) {
        if self.current_screen() != Screen::MirrorsRepos || self.focus != super::Focus::Content {
            return;
        }
        self.popup_kind = Some(PopupKind::OptionalRepos);
        self.popup_items = self.optional_repos_options.clone();
        self.popup_search_query.clear();
        self.popup_in_search = false;
        self.popup_visible_indices = (0..self.popup_items.len()).collect();
        self.popup_selected_visible = 0;
        self.popup_open = true;
    }

    pub fn open_mirrors_custom_server_input(&mut self) {
        if self.current_screen() != Screen::MirrorsRepos || self.focus != super::Focus::Content {
            return;
        }
        self.popup_kind = Some(PopupKind::MirrorsCustomServerInput);
        self.custom_input_buffer.clear();
        self.popup_open = true;
    }

    pub fn open_mirrors_custom_repo_flow(&mut self) {
        if self.current_screen() != Screen::MirrorsRepos || self.focus != super::Focus::Content {
            return;
        }
        self.draft_repo_name.clear();
        self.draft_repo_url.clear();
        self.draft_repo_sig_index = 2;
        self.custom_input_buffer.clear();
        self.popup_kind = Some(PopupKind::MirrorsCustomRepoName);
        self.popup_open = true;
    }

    pub fn open_mirrors_custom_repo_url(&mut self) {
        self.custom_input_buffer.clear();
        self.popup_kind = Some(PopupKind::MirrorsCustomRepoUrl);
    }

    pub fn open_mirrors_custom_repo_sig(&mut self) {
        self.popup_kind = Some(PopupKind::MirrorsCustomRepoSig);
        self.popup_items = vec!["Never".into(), "Optional".into(), "Required".into()];
        self.popup_visible_indices = (0..self.popup_items.len()).collect();
        self.popup_selected_visible = self
            .draft_repo_sig_index
            .min(self.popup_items.len().saturating_sub(1));
        self.popup_in_search = false;
        self.popup_search_query.clear();
    }

    pub fn open_mirrors_custom_repo_signopt(&mut self) {
        self.popup_kind = Some(PopupKind::MirrorsCustomRepoSignOpt);
        self.popup_items = vec!["TrustedOnly".into(), "TrustedAll".into()];
        self.popup_visible_indices = (0..self.popup_items.len()).collect();
        self.popup_selected_visible = self
            .draft_repo_signopt_index
            .min(self.popup_items.len().saturating_sub(1));
        self.popup_in_search = false;
        self.popup_search_query.clear();
    }

    pub fn finalize_custom_repo(&mut self) {
        let sig = match self.draft_repo_sig_index {
            0 => RepoSignature::Never,
            1 => RepoSignature::Optional,
            _ => RepoSignature::Required,
        };
        let sign_option = if self.draft_repo_sig_index == 0 {
            None
        } else {
            Some(match self.draft_repo_signopt_index {
                0 => RepoSignOption::TrustedOnly,
                _ => RepoSignOption::TrustedAll,
            })
        };
        self.custom_repos.push(CustomRepo {
            name: self.draft_repo_name.clone(),
            url: self.draft_repo_url.clone(),
            signature: sig,
            sign_option,
        });
        self.close_popup();
    }
}

pub fn draw_mirrors_repos(frame: &mut ratatui::Frame, app: &mut AppState, area: Rect) {
    let title = Span::styled(
        "Mirrors and Repositories",
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    );

    let options = vec![
        ("Select Regions", app.mirrors_focus_index == 0),
        ("Add custom servers", app.mirrors_focus_index == 1),
        ("Optional repositories", app.mirrors_focus_index == 2),
        ("Add custom repository", app.mirrors_focus_index == 3),
    ];

    let mut lines: Vec<Line> = vec![Line::from(title), Line::from("")];
    for (label, is_focused_line) in options {
        let is_active_line = is_focused_line && matches!(app.focus, super::Focus::Content);
        let bullet = if is_focused_line { "▶" } else { " " };
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
        let mut text = label.to_string();
        if label == "Optional repositories" {
            let mut selected: Vec<String> = app
                .optional_repos_selected
                .iter()
                .filter_map(|&i| app.optional_repos_options.get(i).cloned())
                .collect();
            selected.sort();
            if !selected.is_empty() {
                text.push_str(&format!("  ({})", selected.join(", ")));
            }
            if app.aur_selected {
                let helper = match app.aur_helper_index {
                    Some(0) => "yay",
                    Some(1) => "paru",
                    _ => "choose helper",
                };
                text.push_str(&format!(" — AUR: {helper}"));
            }
        }
        let line = Line::from(vec![
            Span::styled(format!("{bullet} "), bullet_style),
            Span::styled(text, label_style),
        ]);
        lines.push(line);
    }

    let continue_style =
        if app.mirrors_focus_index == 4 && matches!(app.focus, super::Focus::Content) {
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
                    super::Focus::Content => " Desicion Menu (focused) ",
                    _ => " Desicion Menu ",
                }),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(content, area);
}
