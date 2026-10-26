use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NotesData {
    #[serde(default)]
    pub notes: HashMap<String, String>,
}

static GLOBAL_NOTES: OnceLock<CalendarNotesService> = OnceLock::new();

#[derive(Clone)]
pub struct CalendarNotesService {
    data: Arc<RwLock<NotesData>>,
}

impl Default for CalendarNotesService {
    fn default() -> Self {
        Self::new()
    }
}

impl CalendarNotesService {
    pub fn new() -> Self {
        let loaded = Self::load();
        Self {
            data: Arc::new(RwLock::new(loaded)),
        }
    }

    pub fn global() -> &'static CalendarNotesService {
        GLOBAL_NOTES.get_or_init(CalendarNotesService::new)
    }

    pub fn get_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("capsule")
            .join("calendar_notes.json")
    }

    fn load() -> NotesData {
        let path = Self::get_path();
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(data) = serde_json::from_str::<NotesData>(&content) {
                return data;
            }
        }
        NotesData::default()
    }

    fn persist(&self) {
        let path = Self::get_path();
        let json_str = {
            if let Ok(guard) = self.data.read() {
                serde_json::to_string_pretty(&*guard).ok()
            } else {
                None
            }
        };

        if let Some(json) = json_str {
            crate::spawn_tokio(async move {
                if let Some(parent) = path.parent() {
                    let _ = tokio::fs::create_dir_all(parent).await;
                }
                let _ = tokio::fs::write(path, json).await;
            });
        }
    }

    pub fn get_note(&self, date: &str) -> Option<String> {
        let guard = self.data.read().ok()?;
        guard.notes.get(date).cloned()
    }

    pub fn has_note(&self, date: &str) -> bool {
        let guard = match self.data.read() {
            Ok(g) => g,
            Err(_) => return false,
        };
        guard
            .notes
            .get(date)
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false)
    }

    pub fn set_note(&self, date: String, content: String) {
        let should_persist = if let Ok(mut guard) = self.data.write() {
            if content.trim().is_empty() {
                guard.notes.remove(&date).is_some()
            } else {
                guard.notes.insert(date, content);
                true
            }
        } else {
            false
        };

        if should_persist {
            self.persist();
        }
    }

    pub fn delete_note(&self, date: &str) {
        let removed = if let Ok(mut guard) = self.data.write() {
            guard.notes.remove(date).is_some()
        } else {
            false
        };

        if removed {
            self.persist();
        }
    }

    pub fn all_notes(&self) -> HashMap<String, String> {
        self.data.read().map(|g| g.notes.clone()).unwrap_or_default()
    }

    pub fn all_dates_with_notes(&self) -> std::collections::HashSet<String> {
        let mut set = std::collections::HashSet::new();
        if let Ok(guard) = self.data.read() {
            for (date, content) in &guard.notes {
                if !content.trim().is_empty() {
                    set.insert(date.clone());
                }
            }
        }
        set
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notes_crud() {
        let service = CalendarNotesService {
            data: Arc::new(RwLock::new(NotesData::default())),
        };

        assert!(!service.has_note("2026-10-26"));
        assert_eq!(service.get_note("2026-10-26"), None);

        service.set_note("2026-10-26".into(), "Meeting at 10:00".into());
        assert!(service.has_note("2026-10-26"));
        assert_eq!(
            service.get_note("2026-10-26"),
            Some("Meeting at 10:00".into())
        );

        service.set_note("2026-10-26".into(), "   ".into());
        assert!(!service.has_note("2026-10-26"));

        service.set_note("2026-10-27".into(), "Doctor".into());
        assert!(service.has_note("2026-10-27"));
        service.delete_note("2026-10-27");
        assert!(!service.has_note("2026-10-27"));
    }
}
