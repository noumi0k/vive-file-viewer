use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
}

impl FileEntry {
    pub fn new(path: PathBuf) -> Option<Self> {
        let metadata = fs::metadata(&path).ok()?;
        let name = path.file_name()?.to_string_lossy().to_string();

        Some(Self {
            name,
            path,
            is_dir: metadata.is_dir(),
        })
    }
}

#[derive(Debug)]
pub struct FileBrowser {
    pub current_dir: PathBuf,
    pub entries: Vec<FileEntry>,
    pub selected_index: usize,
    pub show_hidden: bool,
}

impl FileBrowser {
    pub fn new(path: &Path, show_hidden: bool) -> Self {
        let current_dir = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let mut browser = Self {
            current_dir,
            entries: Vec::new(),
            selected_index: 0,
            show_hidden,
        };
        browser.refresh();
        browser
    }

    pub fn refresh(&mut self) {
        self.entries.clear();

        if let Ok(read_dir) = fs::read_dir(&self.current_dir) {
            for entry in read_dir.flatten() {
                if let Some(file_entry) = FileEntry::new(entry.path())
                    && (self.show_hidden || !file_entry.name.starts_with('.'))
                {
                    self.entries.push(file_entry);
                }
            }
        }

        self.entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });

        if self.selected_index >= self.entries.len() {
            self.selected_index = self.entries.len().saturating_sub(1);
        }
    }

    pub fn move_up(&mut self) {
        if self.entries.is_empty() {
            return;
        }
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else {
            self.selected_index = self.entries.len() - 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.entries.is_empty() {
            return;
        }
        if self.selected_index < self.entries.len() - 1 {
            self.selected_index += 1;
        } else {
            self.selected_index = 0;
        }
    }

    pub fn go_to_top(&mut self) {
        self.selected_index = 0;
    }

    pub fn go_to_bottom(&mut self) {
        self.selected_index = self.entries.len().saturating_sub(1);
    }

    pub fn selected_entry(&self) -> Option<&FileEntry> {
        self.entries.get(self.selected_index)
    }

    pub fn enter_directory(&mut self) -> bool {
        if let Some(entry) = self.selected_entry()
            && entry.is_dir
        {
            self.current_dir = entry.path.clone();
            self.selected_index = 0;
            self.refresh();
            return true;
        }
        false
    }

    pub fn go_parent(&mut self) -> bool {
        if let Some(parent) = self.current_dir.parent() {
            let old_dir_name = self
                .current_dir
                .file_name()
                .map(|n| n.to_string_lossy().to_string());
            self.current_dir = parent.to_path_buf();
            self.selected_index = 0;
            self.refresh();

            if let Some(old_name) = old_dir_name
                && let Some(idx) = self.entries.iter().position(|e| e.name == old_name)
            {
                self.selected_index = idx;
            }
            return true;
        }
        false
    }

    pub fn toggle_hidden(&mut self) {
        self.show_hidden = !self.show_hidden;
        self.refresh();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::TempDir;

    fn setup_test_dir() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // Create directories
        fs::create_dir(base.join("alpha_dir")).unwrap();
        fs::create_dir(base.join("beta_dir")).unwrap();
        fs::create_dir(base.join(".hidden_dir")).unwrap();

        // Create files
        File::create(base.join("file_a.txt")).unwrap();
        File::create(base.join("file_b.rs")).unwrap();
        File::create(base.join(".hidden_file")).unwrap();

        // Create nested structure
        fs::create_dir(base.join("alpha_dir/nested")).unwrap();
        File::create(base.join("alpha_dir/nested/deep.txt")).unwrap();

        temp_dir
    }

    #[test]
    fn test_new_browser() {
        let temp_dir = setup_test_dir();
        let browser = FileBrowser::new(temp_dir.path(), false);

        assert!(!browser.entries.is_empty());
        assert_eq!(browser.selected_index, 0);
        assert!(!browser.show_hidden);
    }

    #[test]
    fn test_directories_sorted_first() {
        let temp_dir = setup_test_dir();
        let browser = FileBrowser::new(temp_dir.path(), false);

        // First entries should be directories
        let dirs: Vec<_> = browser.entries.iter().take_while(|e| e.is_dir).collect();
        assert!(!dirs.is_empty());

        // After directories come files
        let files: Vec<_> = browser.entries.iter().skip_while(|e| e.is_dir).collect();
        assert!(files.iter().all(|e| !e.is_dir));
    }

    #[test]
    fn test_hidden_files_filtered() {
        let temp_dir = setup_test_dir();
        let browser = FileBrowser::new(temp_dir.path(), false);

        assert!(!browser.entries.iter().any(|e| e.name.starts_with('.')));
    }

    #[test]
    fn test_hidden_files_shown() {
        let temp_dir = setup_test_dir();
        let browser = FileBrowser::new(temp_dir.path(), true);

        assert!(browser.entries.iter().any(|e| e.name.starts_with('.')));
    }

    #[test]
    fn test_move_up_down() {
        let temp_dir = setup_test_dir();
        let mut browser = FileBrowser::new(temp_dir.path(), false);

        assert_eq!(browser.selected_index, 0);

        browser.move_down();
        assert_eq!(browser.selected_index, 1);

        browser.move_up();
        assert_eq!(browser.selected_index, 0);

        // Wrap around
        browser.move_up();
        assert_eq!(browser.selected_index, browser.entries.len() - 1);

        browser.move_down();
        assert_eq!(browser.selected_index, 0);
    }

    #[test]
    fn test_go_to_top_bottom() {
        let temp_dir = setup_test_dir();
        let mut browser = FileBrowser::new(temp_dir.path(), false);

        browser.go_to_bottom();
        assert_eq!(browser.selected_index, browser.entries.len() - 1);

        browser.go_to_top();
        assert_eq!(browser.selected_index, 0);
    }

    #[test]
    fn test_enter_directory() {
        let temp_dir = setup_test_dir();
        let mut browser = FileBrowser::new(temp_dir.path(), false);

        // Find alpha_dir and select it
        let alpha_idx = browser
            .entries
            .iter()
            .position(|e| e.name == "alpha_dir")
            .unwrap();
        browser.selected_index = alpha_idx;

        let old_dir = browser.current_dir.clone();
        assert!(browser.enter_directory());
        assert_ne!(browser.current_dir, old_dir);
        assert!(browser.current_dir.ends_with("alpha_dir"));
    }

    #[test]
    fn test_go_parent() {
        let temp_dir = setup_test_dir();
        let mut browser = FileBrowser::new(&temp_dir.path().join("alpha_dir"), false);

        let old_dir = browser.current_dir.clone();
        assert!(browser.go_parent());
        assert_ne!(browser.current_dir, old_dir);
    }

    #[test]
    fn test_toggle_hidden() {
        let temp_dir = setup_test_dir();
        let mut browser = FileBrowser::new(temp_dir.path(), false);

        let count_without_hidden = browser.entries.len();
        browser.toggle_hidden();
        let count_with_hidden = browser.entries.len();

        assert!(count_with_hidden > count_without_hidden);
    }

    #[test]
    fn test_selected_entry() {
        let temp_dir = setup_test_dir();
        let browser = FileBrowser::new(temp_dir.path(), false);

        let entry = browser.selected_entry();
        assert!(entry.is_some());
    }
}
