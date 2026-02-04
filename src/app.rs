use std::path::{Path, PathBuf};

use ratatui::widgets::ListState;

use crate::config::Config;
use crate::editor::Editor;
use crate::file_browser::FileBrowser;
use crate::preview::{PreviewContent, Previewer};
use crate::search::{FileSearcher, SearchResult};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMode {
    Normal,
    SearchInput,   // 検索文字入力中
    SearchResult,  // 検索結果選択中
    Preview,
}

pub struct App {
    pub browser: FileBrowser,
    pub previewer: Previewer,
    pub editor: Editor,
    pub config: Config,
    pub preview_content: Option<PreviewContent>,
    pub preview_scroll: usize,
    pub preview_height: usize,
    pub input_mode: InputMode,
    pub search_input: String,
    pub status_message: Option<String>,
    pub should_quit: bool,
    pub list_state: ListState,
    pub needs_redraw: bool,
    // 検索関連
    pub searcher: FileSearcher,
    pub search_results: Vec<SearchResult>,
    pub search_selected: usize,
    pub search_list_state: ListState,
    pub base_dir: PathBuf,
    pub search_dirs_only: bool,
}

impl App {
    pub fn new(start_path: &Path, config: Config) -> Self {
        let previewer = Previewer::new(&config.theme, config.preview_max_lines);
        let editor = Editor::new(&config);
        let browser = FileBrowser::new(start_path, config.show_hidden);
        let base_dir = start_path.canonicalize().unwrap_or_else(|_| start_path.to_path_buf());

        let mut list_state = ListState::default();
        list_state.select(Some(0));

        let mut search_list_state = ListState::default();
        search_list_state.select(Some(0));

        let mut app = Self {
            browser,
            previewer,
            editor,
            config,
            preview_content: None,
            preview_scroll: 0,
            preview_height: 20,
            input_mode: InputMode::Normal,
            search_input: String::new(),
            status_message: None,
            should_quit: false,
            list_state,
            needs_redraw: false,
            searcher: FileSearcher::new(),
            search_results: Vec::new(),
            search_selected: 0,
            search_list_state,
            base_dir,
            search_dirs_only: false,
        };

        app.update_preview();
        app
    }

    pub fn update_preview(&mut self) {
        self.preview_scroll = 0;
        if let Some(entry) = self.browser.selected_entry() {
            if !entry.is_dir {
                self.preview_content = Some(self.previewer.preview(&entry.path));
            } else {
                self.preview_content = None;
            }
        } else {
            self.preview_content = None;
        }
    }

    pub fn move_up(&mut self) {
        self.browser.move_up();
        self.list_state.select(Some(self.browser.selected_index));
        self.update_preview();
    }

    pub fn move_down(&mut self) {
        self.browser.move_down();
        self.list_state.select(Some(self.browser.selected_index));
        self.update_preview();
    }

    pub fn go_to_top(&mut self) {
        self.browser.go_to_top();
        self.list_state.select(Some(self.browser.selected_index));
        self.update_preview();
    }

    pub fn go_to_bottom(&mut self) {
        self.browser.go_to_bottom();
        self.list_state.select(Some(self.browser.selected_index));
        self.update_preview();
    }

    pub fn enter(&mut self) {
        if let Some(entry) = self.browser.selected_entry() {
            if entry.is_dir {
                if self.browser.enter_directory() {
                    self.list_state.select(Some(self.browser.selected_index));
                    self.update_preview();
                }
            } else {
                // ファイルの場合はプレビューモードに入る
                self.input_mode = InputMode::Preview;
            }
        }
    }

    pub fn exit_preview(&mut self) {
        self.input_mode = InputMode::Normal;
    }

    pub fn go_parent(&mut self) {
        if self.browser.go_parent() {
            self.list_state.select(Some(self.browser.selected_index));
            self.update_preview();
        }
    }

    pub fn toggle_hidden(&mut self) {
        self.browser.toggle_hidden();
        self.list_state.select(Some(self.browser.selected_index));
        self.update_preview();
    }

    pub fn reload(&mut self) {
        self.browser.refresh();
        self.list_state.select(Some(self.browser.selected_index));
        self.update_preview();
        self.status_message = Some("Reloaded".to_string());
    }

    pub fn open_in_editor(&mut self) {
        if let Some(entry) = self.browser.selected_entry() {
            if !entry.is_dir {
                match self.editor.open(&entry.path) {
                    Ok(_) => {
                        self.needs_redraw = true;
                    }
                    Err(e) => {
                        self.status_message = Some(e);
                        self.needs_redraw = true;
                    }
                }
            }
        }
    }

    pub fn start_search(&mut self, dirs_only: bool) {
        self.input_mode = InputMode::SearchInput;
        self.search_input.clear();
        self.search_results.clear();
        self.search_selected = 0;
        self.search_list_state.select(Some(0));
        self.search_dirs_only = dirs_only;
    }

    pub fn cancel_search(&mut self) {
        self.input_mode = InputMode::Normal;
        self.search_input.clear();
        self.search_results.clear();
        self.search_dirs_only = false;
    }

    /// 検索を実行（Enter で確定時）
    pub fn execute_search(&mut self) {
        if self.search_input.is_empty() {
            self.cancel_search();
            return;
        }
        let mut results = self.searcher.search(&self.base_dir, &self.search_input, 100);

        // フォルダのみフィルタ
        if self.search_dirs_only {
            results.retain(|r| r.is_dir);
        }

        self.search_results = results;
        self.search_selected = 0;
        self.search_list_state.select(Some(0));

        if self.search_results.is_empty() {
            self.status_message = Some("No results found".to_string());
            self.input_mode = InputMode::Normal;
        } else {
            self.input_mode = InputMode::SearchResult;
        }
    }

    /// 検索結果から選択確定
    pub fn confirm_search_result(&mut self) {
        if let Some(result) = self.search_results.get(self.search_selected) {
            let path = result.path.clone();
            let is_dir = result.is_dir;

            self.input_mode = InputMode::Normal;
            self.search_input.clear();
            self.search_results.clear();

            if is_dir {
                self.browser = FileBrowser::new(&path, self.config.show_hidden);
                self.list_state.select(Some(0));
                self.update_preview();
            } else {
                if let Some(parent) = path.parent() {
                    self.browser = FileBrowser::new(parent, self.config.show_hidden);
                    if let Some(file_name) = path.file_name() {
                        let name = file_name.to_string_lossy().to_string();
                        if let Some(idx) = self.browser.entries.iter().position(|e| e.name == name) {
                            self.browser.selected_index = idx;
                            self.list_state.select(Some(idx));
                        }
                    }
                }
                self.update_preview();
                self.input_mode = InputMode::Preview;
            }
        } else {
            self.cancel_search();
        }
    }

    pub fn search_input_char(&mut self, c: char) {
        self.search_input.push(c);
    }

    pub fn search_input_backspace(&mut self) {
        self.search_input.pop();
    }

    pub fn search_move_up(&mut self) {
        if self.search_selected > 0 {
            self.search_selected -= 1;
            self.search_list_state.select(Some(self.search_selected));
        }
    }

    pub fn search_move_down(&mut self) {
        if self.search_selected < self.search_results.len().saturating_sub(1) {
            self.search_selected += 1;
            self.search_list_state.select(Some(self.search_selected));
        }
    }

    pub fn scroll_preview_up(&mut self, amount: usize) {
        self.preview_scroll = self.preview_scroll.saturating_sub(amount);
    }

    pub fn scroll_preview_down(&mut self, amount: usize) {
        if let Some(ref content) = self.preview_content {
            let max_scroll = content.lines.len().saturating_sub(self.preview_height);
            self.preview_scroll = (self.preview_scroll + amount).min(max_scroll);
        }
    }

    pub fn set_preview_height(&mut self, height: usize) {
        self.preview_height = height;
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
