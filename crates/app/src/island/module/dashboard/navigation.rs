use ui::components::InputState;

pub(crate) struct Navigation {
    pub selected_ssid: Option<String>,
    pub password_input: InputState,
    pub error: Option<String>,
}

impl Default for Navigation {
    fn default() -> Self {
        Self {
            selected_ssid: None,
            password_input: InputState::new("").with_masked(true).with_focused(true),
            error: None,
        }
    }
}

impl Navigation {
    pub fn reset(&mut self) {
        self.selected_ssid = None;
        self.password_input.clear();
        self.error = None;
    }
}
