use super::AppState;

impl AppState {
    #[allow(dead_code)]
    pub fn init_abort(&mut self) {
        // TODO: Implement abort confirmation flow integration if additional cleanup is needed.
    }

    pub fn open_abort_confirm_popup(&mut self) {
        self.popup_kind = Some(super::PopupKind::AbortConfirm);
        self.popup_items = vec!["Yes".into(), "No".into()];
        self.popup_visible_indices = (0..self.popup_items.len()).collect();
        self.popup_selected_visible = 1; // default to No
        self.popup_in_search = false;
        self.popup_search_query.clear();
        self.popup_open = true;
    }
}
