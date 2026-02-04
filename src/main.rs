mod app;
mod config;
mod editor;
mod file_browser;
mod preview;
mod search;
mod ui;

use std::io;
use std::path::PathBuf;
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::{App, InputMode};
use config::Config;

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let start_path = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        std::env::current_dir()?
    };

    let config = Config::load();
    let mut app = App::new(&start_path, config);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    loop {
        // vim から戻ってきた場合は画面をクリアして再描画
        if app.needs_redraw {
            terminal.clear()?;
            app.needs_redraw = false;
        }

        terminal.draw(|f| ui::draw(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                app.status_message = None;

                match app.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => {
                            app.quit();
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            app.move_down();
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            app.move_up();
                        }
                        KeyCode::Char('l') | KeyCode::Enter | KeyCode::Right => {
                            app.enter();
                        }
                        KeyCode::Char('h') | KeyCode::Backspace | KeyCode::Left => {
                            app.go_parent();
                        }
                        KeyCode::Char('g') => {
                            app.go_to_top();
                        }
                        KeyCode::Char('G') => {
                            app.go_to_bottom();
                        }
                        KeyCode::Char('e') => {
                            app.open_in_editor();
                        }
                        KeyCode::Char('/') => {
                            app.start_search(false);
                        }
                        KeyCode::Char('D') => {
                            app.start_search(true);  // フォルダのみ検索
                        }
                        KeyCode::Char('.') => {
                            app.toggle_hidden();
                        }
                        KeyCode::Char('r') => {
                            app.reload();
                        }
                        KeyCode::Char('y') => {
                            app.copy_path();
                        }
                        KeyCode::Char('f') => {
                            app.start_jump();
                        }
                        KeyCode::Char(';') => {
                            app.jump_next();
                        }
                        KeyCode::Char(',') => {
                            app.jump_prev();
                        }
                        KeyCode::Char('?') => {
                            app.show_help();
                        }
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.quit();
                        }
                        _ => {}
                    },
                    InputMode::Help => match key.code {
                        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
                            app.close_help();
                        }
                        _ => {}
                    },
                    InputMode::JumpInput => match key.code {
                        KeyCode::Char(c) => {
                            app.execute_jump(c);
                        }
                        KeyCode::Esc => {
                            app.cancel_jump();
                        }
                        _ => {
                            app.cancel_jump();
                        }
                    },
                    InputMode::Preview => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('h') | KeyCode::Left => {
                            app.exit_preview();
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            app.scroll_preview_down(1);
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            app.scroll_preview_up(1);
                        }
                        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            let half = app.preview_height / 2;
                            app.scroll_preview_down(half.max(1));
                        }
                        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            let half = app.preview_height / 2;
                            app.scroll_preview_up(half.max(1));
                        }
                        KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.scroll_preview_down(app.preview_height.saturating_sub(2));
                        }
                        KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.scroll_preview_up(app.preview_height.saturating_sub(2));
                        }
                        KeyCode::PageUp => {
                            app.scroll_preview_up(app.preview_height.saturating_sub(2));
                        }
                        KeyCode::PageDown => {
                            app.scroll_preview_down(app.preview_height.saturating_sub(2));
                        }
                        KeyCode::Char('g') => {
                            app.preview_scroll = 0;
                        }
                        KeyCode::Char('G') => {
                            if let Some(ref content) = app.preview_content {
                                app.preview_scroll = content.lines.len().saturating_sub(app.preview_height);
                            }
                        }
                        KeyCode::Char('e') => {
                            app.open_in_editor();
                        }
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.quit();
                        }
                        _ => {}
                    },
                    InputMode::SearchInput => match key.code {
                        KeyCode::Enter => {
                            app.execute_search();
                        }
                        KeyCode::Esc => {
                            app.cancel_search();
                        }
                        KeyCode::Backspace => {
                            app.search_input_backspace();
                        }
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.cancel_search();
                        }
                        KeyCode::Char(c) => {
                            app.search_input_char(c);
                        }
                        _ => {}
                    },
                    InputMode::SearchResult => match key.code {
                        KeyCode::Enter => {
                            app.confirm_search_result();
                        }
                        KeyCode::Esc | KeyCode::Char('q') => {
                            app.cancel_search();
                        }
                        KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab => {
                            app.search_move_up();
                        }
                        KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => {
                            app.search_move_down();
                        }
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.cancel_search();
                        }
                        KeyCode::Char('/') => {
                            // 再検索（モードは維持）
                            app.search_input.clear();
                            app.input_mode = InputMode::SearchInput;
                        }
                        _ => {}
                    },
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
