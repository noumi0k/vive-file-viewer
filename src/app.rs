use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

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
    Searching,     // 検索実行中（スピナー表示）
    SearchResult,  // 検索結果選択中
    Preview,
    JumpInput,     // fキー後の1文字待ち
    Help,          // ヘルプ画面
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
    pub search_receiver: Option<Receiver<Vec<SearchResult>>>,
    pub spinner_frame: usize,
    // ジャンプ関連
    pub last_jump_char: Option<char>,
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
            search_receiver: None,
            spinner_frame: 0,
            last_jump_char: None,
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
        self.clear_jump();
        self.browser.move_up();
        self.list_state.select(Some(self.browser.selected_index));
        self.update_preview();
    }

    pub fn move_down(&mut self) {
        self.clear_jump();
        self.browser.move_down();
        self.list_state.select(Some(self.browser.selected_index));
        self.update_preview();
    }

    fn clear_jump(&mut self) {
        self.last_jump_char = None;
    }

    pub fn go_to_top(&mut self) {
        self.clear_jump();
        self.browser.go_to_top();
        self.list_state.select(Some(self.browser.selected_index));
        self.update_preview();
    }

    pub fn go_to_bottom(&mut self) {
        self.clear_jump();
        self.browser.go_to_bottom();
        self.list_state.select(Some(self.browser.selected_index));
        self.update_preview();
    }

    pub fn enter(&mut self) {
        self.clear_jump();
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
        self.clear_jump();
        if self.browser.go_parent() {
            self.list_state.select(Some(self.browser.selected_index));
            self.update_preview();
        }
    }

    pub fn toggle_hidden(&mut self) {
        self.clear_jump();
        self.browser.toggle_hidden();
        self.list_state.select(Some(self.browser.selected_index));
        self.update_preview();
    }

    pub fn reload(&mut self) {
        self.clear_jump();
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

    pub fn start_search(&mut self) {
        self.clear_jump();
        self.input_mode = InputMode::SearchInput;
        self.search_input.clear();
        self.search_results.clear();
        self.search_selected = 0;
        self.search_list_state.select(Some(0));
        self.search_dirs_only = false;
    }

    pub fn cancel_search(&mut self) {
        self.input_mode = InputMode::Normal;
        self.search_input.clear();
        self.search_results.clear();
        self.search_dirs_only = false;
    }

    /// 検索入力をパースしてクエリとオプションを分離
    /// 戻り値: (query, dirs_only, exact, base_path)
    fn parse_search_input(&self) -> (String, bool, bool, Option<PathBuf>) {
        let mut query_parts: Vec<&str> = Vec::new();
        let mut exact = false;
        let mut dirs_only = self.search_dirs_only; // Dキーで開始した場合のデフォルト
        let mut base_path: Option<PathBuf> = None;

        let parts: Vec<&str> = self.search_input.split_whitespace().collect();
        let mut i = 0;
        while i < parts.len() {
            match parts[i] {
                "-e" | "--exact" => exact = true,
                "-d" | "--dir" => dirs_only = true,
                "-b" | "--base" => {
                    if i + 1 < parts.len() {
                        i += 1;
                        let path_str = parts[i];
                        let expanded = if path_str.starts_with("~/") {
                            if let Ok(home) = std::env::var("HOME") {
                                PathBuf::from(home).join(&path_str[2..])
                            } else {
                                PathBuf::from(path_str)
                            }
                        } else if path_str == "~" {
                            if let Ok(home) = std::env::var("HOME") {
                                PathBuf::from(home)
                            } else {
                                PathBuf::from(path_str)
                            }
                        } else {
                            PathBuf::from(path_str)
                        };
                        base_path = Some(expanded);
                    }
                }
                _ => query_parts.push(parts[i]),
            }
            i += 1;
        }

        (query_parts.join(" "), dirs_only, exact, base_path)
    }

    /// 検索を実行（Enter で確定時）- バックグラウンドで実行開始
    pub fn execute_search(&mut self) {
        if self.search_input.is_empty() {
            self.cancel_search();
            return;
        }

        // 検索入力をパース
        let (query, dirs_only, exact, base_path) = self.parse_search_input();

        if query.is_empty() {
            self.cancel_search();
            return;
        }

        // 検索をバックグラウンドスレッドで実行
        let (tx, rx): (Sender<Vec<SearchResult>>, Receiver<Vec<SearchResult>>) = mpsc::channel();
        let base_dir = base_path.unwrap_or_else(|| self.browser.current_dir.clone());

        thread::spawn(move || {
            let mut searcher = FileSearcher::new();
            let results = searcher.search(&base_dir, &query, 100, dirs_only, exact);
            let _ = tx.send(results);
        });

        self.search_receiver = Some(rx);
        self.spinner_frame = 0;
        self.input_mode = InputMode::Searching;
    }

    /// 検索結果をポーリング（main loopから呼ばれる）
    pub fn poll_search(&mut self) -> bool {
        if let Some(ref rx) = self.search_receiver {
            match rx.try_recv() {
                Ok(results) => {
                    self.search_results = results;
                    self.search_selected = 0;
                    self.search_list_state.select(Some(0));
                    self.search_receiver = None;

                    if self.search_results.is_empty() {
                        self.status_message = Some("No results found".to_string());
                        self.input_mode = InputMode::Normal;
                    } else {
                        self.input_mode = InputMode::SearchResult;
                    }
                    return true;
                }
                Err(mpsc::TryRecvError::Empty) => {
                    // まだ検索中
                    self.spinner_frame = (self.spinner_frame + 1) % 10;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    // スレッドが終了（エラー）
                    self.search_receiver = None;
                    self.status_message = Some("Search failed".to_string());
                    self.input_mode = InputMode::Normal;
                    return true;
                }
            }
        }
        false
    }

    /// スピナー文字を取得
    pub fn spinner_char(&self) -> char {
        const SPINNER: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        SPINNER[self.spinner_frame % SPINNER.len()]
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
        if self.search_results.is_empty() {
            return;
        }
        if self.search_selected > 0 {
            self.search_selected -= 1;
        } else {
            self.search_selected = self.search_results.len() - 1;
        }
        self.search_list_state.select(Some(self.search_selected));
    }

    pub fn search_move_down(&mut self) {
        if self.search_results.is_empty() {
            return;
        }
        if self.search_selected < self.search_results.len() - 1 {
            self.search_selected += 1;
        } else {
            self.search_selected = 0;
        }
        self.search_list_state.select(Some(self.search_selected));
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

    pub fn copy_path(&mut self) {
        if let Some(entry) = self.browser.selected_entry() {
            let path_str = entry.path.to_string_lossy().to_string();

            #[cfg(target_os = "macos")]
            let result = std::process::Command::new("pbcopy")
                .stdin(std::process::Stdio::piped())
                .spawn()
                .and_then(|mut child| {
                    use std::io::Write;
                    if let Some(stdin) = child.stdin.as_mut() {
                        stdin.write_all(path_str.as_bytes())?;
                    }
                    child.wait()
                });

            #[cfg(target_os = "linux")]
            let result = std::process::Command::new("xclip")
                .args(["-selection", "clipboard"])
                .stdin(std::process::Stdio::piped())
                .spawn()
                .and_then(|mut child| {
                    use std::io::Write;
                    if let Some(stdin) = child.stdin.as_mut() {
                        stdin.write_all(path_str.as_bytes())?;
                    }
                    child.wait()
                });

            #[cfg(target_os = "windows")]
            let result = std::process::Command::new("cmd")
                .args(["/C", &format!("echo {} | clip", path_str)])
                .spawn()
                .and_then(|mut child| child.wait());

            match result {
                Ok(_) => {
                    self.status_message = Some(format!("Copied: {}", path_str));
                }
                Err(e) => {
                    self.status_message = Some(format!("Failed to copy: {}", e));
                }
            }
        }
    }

    pub fn start_jump(&mut self) {
        self.input_mode = InputMode::JumpInput;
    }

    pub fn execute_jump(&mut self, c: char) {
        self.last_jump_char = Some(c);
        self.jump_to_char(c, true);
        self.input_mode = InputMode::Normal;
    }

    pub fn jump_next(&mut self) {
        if let Some(c) = self.last_jump_char {
            self.jump_to_char(c, true);
        }
    }

    pub fn jump_prev(&mut self) {
        if let Some(c) = self.last_jump_char {
            self.jump_to_char(c, false);
        }
    }

    fn jump_to_char(&mut self, c: char, forward: bool) {
        let entries = &self.browser.entries;
        if entries.is_empty() {
            return;
        }

        let c_lower = c.to_lowercase().next().unwrap_or(c);
        let current = self.browser.selected_index;
        let len = entries.len();

        if forward {
            // 現在位置の次から検索、末尾まで行ったら先頭から
            for i in 1..=len {
                let idx = (current + i) % len;
                if entries[idx].name.to_lowercase().starts_with(c_lower) {
                    self.browser.selected_index = idx;
                    self.list_state.select(Some(idx));
                    self.update_preview();
                    return;
                }
            }
        } else {
            // 現在位置の前から検索、先頭まで行ったら末尾から
            for i in 1..=len {
                let idx = (current + len - i) % len;
                if entries[idx].name.to_lowercase().starts_with(c_lower) {
                    self.browser.selected_index = idx;
                    self.list_state.select(Some(idx));
                    self.update_preview();
                    return;
                }
            }
        }

        self.status_message = Some(format!("No match for '{}'", c));
    }

    pub fn cancel_jump(&mut self) {
        self.input_mode = InputMode::Normal;
    }

    pub fn show_help(&mut self) {
        self.input_mode = InputMode::Help;
    }

    pub fn close_help(&mut self) {
        self.input_mode = InputMode::Normal;
    }
}
