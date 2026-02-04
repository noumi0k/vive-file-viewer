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
                if let Some(file_entry) = FileEntry::new(entry.path()) {
                    if self.show_hidden || !file_entry.name.starts_with('.') {
                        self.entries.push(file_entry);
                    }
                }
            }
        }

        self.entries.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => Ordering::Less,
                (false, true) => Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });

        if self.selected_index >= self.entries.len() {
            self.selected_index = self.entries.len().saturating_sub(1);
        }
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected_index < self.entries.len().saturating_sub(1) {
            self.selected_index += 1;
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
        if let Some(entry) = self.selected_entry() {
            if entry.is_dir {
                self.current_dir = entry.path.clone();
                self.selected_index = 0;
                self.refresh();
                return true;
            }
        }
        false
    }

    pub fn go_parent(&mut self) -> bool {
        if let Some(parent) = self.current_dir.parent() {
            let old_dir_name = self.current_dir.file_name().map(|n| n.to_string_lossy().to_string());
            self.current_dir = parent.to_path_buf();
            self.selected_index = 0;
            self.refresh();

            if let Some(old_name) = old_dir_name {
                if let Some(idx) = self.entries.iter().position(|e| e.name == old_name) {
                    self.selected_index = idx;
                }
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
