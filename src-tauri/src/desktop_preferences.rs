use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use atomic_write_file::AtomicWriteFile;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    System,
    Light,
    Dark,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct DesktopPreferences {
    pub theme: ThemeMode,
    pub silent_start: bool,
    pub minimize_to_tray: bool,
}

impl Default for DesktopPreferences {
    fn default() -> Self {
        Self {
            theme: ThemeMode::System,
            silent_start: true,
            minimize_to_tray: true,
        }
    }
}

pub trait DesktopPreferencesStore: Send + Sync {
    fn load(&self) -> Result<DesktopPreferences, String>;
    fn save(&self, preferences: &DesktopPreferences) -> Result<(), String>;
}

#[derive(Clone, Debug)]
pub struct FileDesktopPreferencesStore {
    path: PathBuf,
}

impl FileDesktopPreferencesStore {
    pub fn at(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    fn ensure_parent_dir(&self) -> Result<(), String> {
        let parent = self
            .path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent).map_err(store_error)
    }
}

impl DesktopPreferencesStore for FileDesktopPreferencesStore {
    fn load(&self) -> Result<DesktopPreferences, String> {
        match fs::read_to_string(&self.path) {
            Ok(contents) => serde_json::from_str(&contents)
                .map_err(|_| "desktop preferences are not valid JSON".to_owned()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                Ok(DesktopPreferences::default())
            }
            Err(error) => Err(store_error(error)),
        }
    }

    fn save(&self, preferences: &DesktopPreferences) -> Result<(), String> {
        self.ensure_parent_dir()?;
        let contents = serde_json::to_vec(preferences)
            .map_err(|_| "desktop preferences cannot be serialized".to_owned())?;
        let mut file = AtomicWriteFile::open(&self.path).map_err(store_error)?;
        file.write_all(&contents).map_err(store_error)?;
        file.sync_all().map_err(store_error)?;
        file.commit().map_err(store_error)
    }
}

fn store_error(error: std::io::Error) -> String {
    format!("desktop preferences file operation failed: {error}")
}
