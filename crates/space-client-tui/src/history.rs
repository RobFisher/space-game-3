use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

pub const DEFAULT_MAX_HISTORY: usize = 1_000;

#[derive(Debug, Clone)]
pub enum CommandHistoryStore {
    Disabled,
    Path { path: PathBuf, max_entries: usize },
}

impl CommandHistoryStore {
    pub fn default_path() -> Self {
        Self::Path {
            path: default_history_path(),
            max_entries: DEFAULT_MAX_HISTORY,
        }
    }

    pub fn path(path: impl Into<PathBuf>) -> Self {
        Self::Path {
            path: path.into(),
            max_entries: DEFAULT_MAX_HISTORY,
        }
    }

    pub fn disabled() -> Self {
        Self::Disabled
    }

    pub fn load(&self) -> io::Result<Vec<String>> {
        let Self::Path {
            path, max_entries, ..
        } = self
        else {
            return Ok(Vec::new());
        };

        match fs::read_to_string(path) {
            Ok(contents) => Ok(bound_history(
                contents
                    .lines()
                    .map(str::trim)
                    .filter(|line| !line.is_empty())
                    .map(ToOwned::to_owned)
                    .collect(),
                *max_entries,
            )),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
            Err(err) => Err(err),
        }
    }

    pub fn save(&self, history: &[String]) -> io::Result<()> {
        let Self::Path {
            path, max_entries, ..
        } = self
        else {
            return Ok(());
        };

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let history = bound_history(history.to_vec(), *max_entries);
        fs::write(path, history.join("\n"))
    }
}

fn default_history_path() -> PathBuf {
    if let Some(data_home) = env::var_os("XDG_DATA_HOME") {
        return Path::new(&data_home)
            .join("space-game")
            .join("client-tui")
            .join("history");
    }
    if let Some(home) = env::var_os("HOME") {
        return Path::new(&home)
            .join(".local")
            .join("share")
            .join("space-game")
            .join("client-tui")
            .join("history");
    }
    PathBuf::from("space-game-client-tui-history")
}

fn bound_history(mut history: Vec<String>, max_entries: usize) -> Vec<String> {
    if history.len() > max_entries {
        let start = history.len() - max_entries;
        history.drain(0..start);
    }
    history
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    fn temp_history_path(name: &str) -> PathBuf {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        env::temp_dir().join(format!("space-game-{name}-{id}.history"))
    }

    #[test]
    fn saves_and_loads_history_from_injected_path() {
        let path = temp_history_path("round-trip");
        let store = CommandHistoryStore::path(&path);

        store
            .save(&["objects".to_string(), "status".to_string()])
            .unwrap();

        assert_eq!(
            store.load().unwrap(),
            vec!["objects".to_string(), "status".to_string()]
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn bounds_loaded_history_to_newest_entries() {
        let path = temp_history_path("bounded");
        let store = CommandHistoryStore::Path {
            path: path.clone(),
            max_entries: 2,
        };
        fs::write(&path, "objects\nstatus\ntime\n").unwrap();

        assert_eq!(
            store.load().unwrap(),
            vec!["status".to_string(), "time".to_string()]
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn disabled_store_does_not_touch_files() {
        let store = CommandHistoryStore::disabled();

        assert!(store.load().unwrap().is_empty());
        store.save(&["objects".to_string()]).unwrap();
    }
}
