#[derive(Default)]
pub(crate) struct Navigation {
    pub selected_ssid: Option<String>,
    pub password: String,
    pub error: Option<String>,
}
impl Navigation {
    pub fn reset(&mut self) {
        self.selected_ssid = None;
        self.password.clear();
        self.error = None;
    }
}
