use crate::new_capsule::widgets::dashboard::DashboardView;

pub(crate) struct Navigation {
    pub view: DashboardView,
    pub selected_ssid: Option<String>,
    pub password: String,
    pub error: Option<String>,
}
impl Default for Navigation {
    fn default() -> Self {
        Self {
            view: DashboardView::Home,
            selected_ssid: None,
            password: String::new(),
            error: None,
        }
    }
}
impl Navigation {
    pub fn open(&mut self, view: DashboardView) {
        self.view = view;
        self.selected_ssid = None;
        self.password.clear();
        self.error = None;
    }
    pub fn escape(&mut self) -> bool {
        if self.view == DashboardView::Home {
            true
        } else {
            self.open(DashboardView::Home);
            false
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn details_clear_password_and_escape_returns_home_before_closing() {
        let mut navigation = Navigation::default();
        navigation.open(DashboardView::Wifi);
        navigation.selected_ssid = Some("private-network".into());
        navigation.password = "secret".into();
        navigation.error = Some("failed".into());
        assert!(!navigation.escape());
        assert_eq!(navigation.view, DashboardView::Home);
        assert!(navigation.password.is_empty());
        assert!(navigation.selected_ssid.is_none());
        assert!(navigation.error.is_none());
        assert!(navigation.escape());
    }
    #[test]
    fn changing_details_and_reopening_reset_transient_state() {
        let mut navigation = Navigation::default();
        navigation.open(DashboardView::Wifi);
        navigation.password = "secret".into();
        navigation.open(DashboardView::Bluetooth);
        assert!(navigation.password.is_empty());
        navigation.open(DashboardView::Home);
        assert!(navigation.escape());
    }
}
