use crate::app::AppState;

impl AppState {
    pub fn is_ascii_only(input: &str) -> bool {
        input.is_ascii()
    }

    pub fn is_ascii_lowercase_only(input: &str) -> bool {
        input.chars().all(|c| c.is_ascii_lowercase())
    }
}
