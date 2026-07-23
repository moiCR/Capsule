pub mod launcher;

pub use launcher::LauncherService;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Application {
    pub id: String,
    pub name: String,
    pub generic_name: Option<String>,
    pub comment: Option<String>,
    pub exec: String,
    pub icon_name: Option<String>,
    pub icon_path: Option<PathBuf>,
    pub keywords: Vec<String>,
    pub terminal: bool,
}

impl Application {
    pub fn launch(&self) -> anyhow::Result<()> {
        LauncherService::launch_app(self)
    }
}
