mod app;
mod config;
mod editor;
mod file_browser;
mod preview;
mod search;
mod ui;

use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use clap::{CommandFactory, Parser, Subcommand};
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use indicatif::{ProgressBar, ProgressStyle};
use ratatui::{Terminal, backend::CrosstermBackend};

use app::{App, InputMode};
use config::Config;
use search::{FileSearcher, SearchResult};

#[derive(Parser)]
#[command(name = "vfv")]
#[command(about = "A fast terminal file viewer with fuzzy search")]
#[command(version)]
struct Cli {
    /// Directory to open (for TUI mode)
    #[arg(value_name = "PATH")]
    path: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Fuzzy search files and directories
    Find {
        /// Search query
        query: String,

        /// Base directory to search in
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,

        /// Output as JSON
        #[arg(short = 'j', long = "json")]
        json: bool,

        /// Search directories only
        #[arg(short = 'd', long = "dir")]
        dir_only: bool,

        /// Maximum number of results
        #[arg(short = 'n', long = "limit", default_value = "20")]
        limit: usize,

        /// Output only the top result (shortcut for -n 1)
        #[arg(short = '1', long = "first")]
        first: bool,

        /// Timeout in seconds (0 = no timeout)
        #[arg(short = 't', long = "timeout", default_value = "0")]
        timeout: u64,

        /// Quiet mode (no spinner)
        #[arg(short = 'q', long = "quiet")]
        quiet: bool,

        /// Compact JSON output (single line)
        #[arg(short = 'c', long = "compact")]
        compact: bool,

        /// Exact match (no fuzzy matching)
        #[arg(short = 'e', long = "exact")]
        exact: bool,
    },

    /// Initialize config, shell completions, and man page
    Init {
        /// Overwrite existing files
        #[arg(short, long)]
        force: bool,
    },

    /// Generate man page
    #[command(name = "man")]
    ManPage,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Find {
            query,
            path,
            json,
            dir_only,
            limit,
            first,
            timeout,
            quiet,
            compact,
            exact,
        }) => run_find(
            query, path, json, dir_only, limit, first, timeout, quiet, compact, exact,
        ),
        Some(Commands::Init { force }) => run_init(force),
        Some(Commands::ManPage) => {
            run_man_page();
            Ok(())
        }
        None => {
            let start_path = cli.path.unwrap_or(std::env::current_dir()?);
            run_tui(&start_path)
        }
    }
}

/// Maximum allowed query length to prevent memory exhaustion
const MAX_QUERY_LENGTH: usize = 1000;

#[allow(clippy::too_many_arguments)]
fn run_find(
    query: String,
    path: Option<PathBuf>,
    json: bool,
    dir_only: bool,
    limit: usize,
    first: bool,
    timeout: u64,
    quiet: bool,
    compact: bool,
    exact: bool,
) -> io::Result<()> {
    // Validate query length
    if query.len() > MAX_QUERY_LENGTH {
        eprintln!(
            "Query too long: {} characters (max: {})",
            query.len(),
            MAX_QUERY_LENGTH
        );
        std::process::exit(1);
    }

    let base_dir = path.unwrap_or(std::env::current_dir()?);
    let actual_limit = if first { 1 } else { limit };
    let timeout_duration = if timeout > 0 {
        Some(Duration::from_secs(timeout))
    } else {
        None
    };

    // スピナー表示（quiet/jsonモードでは非表示）
    let show_spinner = !quiet && !json;
    let spinner = if show_spinner {
        let pb = ProgressBar::new_spinner();
        if let Ok(style) = ProgressStyle::default_spinner().template("{spinner:.cyan} {msg}") {
            pb.set_style(style);
        }
        pb.set_message("Searching...");
        pb.enable_steady_tick(Duration::from_millis(80));
        Some(pb)
    } else {
        None
    };

    // 検索をバックグラウンドスレッドで実行
    let (tx, rx) = mpsc::channel::<Vec<SearchResult>>();
    let search_query = query.clone();
    let search_dir = base_dir.clone();

    thread::spawn(move || {
        let mut searcher = FileSearcher::new();
        let results = searcher.search(&search_dir, &search_query, actual_limit, dir_only, exact);
        let _ = tx.send(results);
    });

    // タイムアウト付きで結果を待つ
    let start = Instant::now();
    let results = loop {
        match rx.try_recv() {
            Ok(results) => break Some(results),
            Err(mpsc::TryRecvError::Empty) => {
                if let Some(timeout_dur) = timeout_duration
                    && start.elapsed() >= timeout_dur
                {
                    break None;
                }
                thread::sleep(Duration::from_millis(50));
            }
            Err(mpsc::TryRecvError::Disconnected) => break Some(Vec::new()),
        }
    };

    // スピナー終了
    if let Some(pb) = spinner {
        pb.finish_and_clear();
    }

    // 結果出力
    match results {
        Some(results) => {
            let is_empty = results.is_empty();

            if json {
                let json_results: Vec<serde_json::Value> = results
                    .iter()
                    .map(|r| {
                        serde_json::json!({
                            "path": r.path.to_string_lossy(),
                            "name": r.path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default(),
                            "is_dir": r.is_dir,
                            "score": r.score
                        })
                    })
                    .collect();

                let output = if compact {
                    serde_json::to_string(&json_results)
                } else {
                    serde_json::to_string_pretty(&json_results)
                };
                match output {
                    Ok(s) => println!("{}", s),
                    Err(e) => {
                        eprintln!("Failed to serialize JSON: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                for result in results {
                    println!("{}", result.path.display());
                }
            }

            // 結果が0件の場合は終了コード1
            if is_empty {
                std::process::exit(1);
            }
        }
        None => {
            if json {
                let error_json = serde_json::json!({
                    "error": "timeout",
                    "timeout_seconds": timeout
                });
                let output = if compact {
                    serde_json::to_string(&error_json)
                } else {
                    serde_json::to_string_pretty(&error_json)
                };
                match output {
                    Ok(s) => println!("{}", s),
                    Err(e) => eprintln!("Failed to serialize JSON: {}", e),
                }
            } else {
                eprintln!("Search timed out after {} seconds", timeout);
            }
            std::process::exit(124); // タイムアウトの終了コード
        }
    }

    Ok(())
}

fn run_tui(start_path: &Path) -> io::Result<()> {
    let config = Config::load();
    let mut app = App::new(start_path, config);

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

        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
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
                        app.start_search();
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
                            app.preview_scroll =
                                content.lines.len().saturating_sub(app.preview_height);
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
                InputMode::Searching => match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        app.cancel_search();
                    }
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.cancel_search();
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

        // 検索中の場合、結果をポーリング
        if app.input_mode == InputMode::Searching {
            app.poll_search();
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

/// Detect current shell from $SHELL environment variable
fn detect_shell() -> String {
    std::env::var("SHELL")
        .unwrap_or_default()
        .rsplit('/')
        .next()
        .unwrap_or("unknown")
        .to_string()
}

/// Initialize configuration, shell completions, and man page
fn run_init(force: bool) -> io::Result<()> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let shell = detect_shell();

    println!("Detected shell: {}", shell);
    println!();

    // 1. Config file (all shells)
    let config_path = Config::config_path();
    if !config_path.exists() || force {
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let default_config = r#"# vfv configuration file
# See https://github.com/noumi0k/vive-file-viewer for more information

# Editor command to use when pressing 'e'
editor = "vim"
editor_args = []

# Show hidden files by default
show_hidden = false

# Maximum lines to preview (for performance)
preview_max_lines = 1000

# Syntax highlighting theme
# Options: "base16-ocean.dark", "base16-eighties.dark",
#          "base16-mocha.dark", "Solarized (dark)", "Solarized (light)"
theme = "base16-ocean.dark"
"#;
        std::fs::write(&config_path, default_config)?;
        println!("Created: {}", config_path.display());
    } else {
        println!("Exists:  {} (use --force to overwrite)", config_path.display());
    }

    // 2. Man page (all shells)
    let man_dir = PathBuf::from(&home).join(".local/share/man/man1");
    let man_path = man_dir.join("vfv.1");
    if !man_path.exists() || force {
        std::fs::create_dir_all(&man_dir)?;
        let cmd = Cli::command();
        let man = clap_mangen::Man::new(cmd);
        let mut buffer = Vec::new();
        man.render(&mut buffer).expect("Failed to generate man page");
        std::fs::write(&man_path, buffer)?;
        println!("Created: {}", man_path.display());
    } else {
        println!("Exists:  {} (use --force to overwrite)", man_path.display());
    }

    // 3. Shell-specific setup
    match shell.as_str() {
        "zsh" => setup_zsh(&home, force)?,
        "bash" => setup_bash(&home, force)?,
        "fish" => setup_fish(&home, force)?,
        _ => {
            println!();
            println!("Shell '{}' is not supported for auto-setup.", shell);
            println!("Manual setup:");
            println!("  - Completions: Copy from https://github.com/noumi0k/vive-file-viewer/tree/main/completions");
            println!("  - Man page: Add to MANPATH: $HOME/.local/share/man");
        }
    }

    Ok(())
}

/// Setup for zsh
fn setup_zsh(home: &str, force: bool) -> io::Result<()> {
    // Install completion script
    let zfunc_dir = PathBuf::from(home).join(".zfunc");
    let completion_path = zfunc_dir.join("_vfv");
    if !completion_path.exists() || force {
        std::fs::create_dir_all(&zfunc_dir)?;
        let completion_script = include_str!("../completions/_vfv");
        std::fs::write(&completion_path, completion_script)?;
        println!("Created: {}", completion_path.display());
    } else {
        println!("Exists:  {} (use --force to overwrite)", completion_path.display());
    }

    // Update .zshrc
    let zshrc_path = PathBuf::from(home).join(".zshrc");
    if zshrc_path.exists() {
        let zshrc_content = std::fs::read_to_string(&zshrc_path)?;
        let mut updates = Vec::new();

        if !zshrc_content.contains(".zfunc") {
            updates.push("fpath=(~/.zfunc $fpath)");
        }
        if !zshrc_content.contains(".local/share/man") {
            updates.push("export MANPATH=\"$HOME/.local/share/man:$MANPATH\"");
        }

        if !updates.is_empty() {
            let lines: Vec<&str> = zshrc_content.lines().collect();
            let mut new_lines: Vec<String> = Vec::new();
            let mut inserted = false;

            for line in &lines {
                if !inserted && line.contains("compinit") {
                    new_lines.push("# vfv setup".to_string());
                    for update in &updates {
                        new_lines.push(update.to_string());
                    }
                    new_lines.push(String::new());
                    inserted = true;
                }
                new_lines.push(line.to_string());
            }

            if !inserted {
                new_lines.push(String::new());
                new_lines.push("# vfv setup".to_string());
                for update in &updates {
                    new_lines.push(update.to_string());
                }
            }

            std::fs::write(&zshrc_path, new_lines.join("\n") + "\n")?;
            println!("Updated: {}", zshrc_path.display());
        } else {
            println!("OK:      {} (already configured)", zshrc_path.display());
        }
    }

    println!();
    println!("Done! Restart your shell or run: source ~/.zshrc");

    Ok(())
}

/// Setup for bash
fn setup_bash(home: &str, force: bool) -> io::Result<()> {
    // Install completion script
    let bash_completion_dir = PathBuf::from(home).join(".local/share/bash-completion/completions");
    let completion_path = bash_completion_dir.join("vfv");
    if !completion_path.exists() || force {
        std::fs::create_dir_all(&bash_completion_dir)?;
        let completion_script = include_str!("../completions/vfv.bash");
        std::fs::write(&completion_path, completion_script)?;
        println!("Created: {}", completion_path.display());
    } else {
        println!("Exists:  {} (use --force to overwrite)", completion_path.display());
    }

    // Update .bashrc
    let bashrc_path = PathBuf::from(home).join(".bashrc");
    if bashrc_path.exists() {
        let bashrc_content = std::fs::read_to_string(&bashrc_path)?;
        let mut updates = Vec::new();

        if !bashrc_content.contains(".local/share/man") {
            updates.push("export MANPATH=\"$HOME/.local/share/man:$MANPATH\"");
        }
        if !bashrc_content.contains(".local/share/bash-completion") {
            updates.push("source ~/.local/share/bash-completion/completions/vfv 2>/dev/null");
        }

        if !updates.is_empty() {
            let mut new_content = bashrc_content.clone();
            if !new_content.ends_with('\n') {
                new_content.push('\n');
            }
            new_content.push_str("\n# vfv setup\n");
            for update in &updates {
                new_content.push_str(update);
                new_content.push('\n');
            }
            std::fs::write(&bashrc_path, new_content)?;
            println!("Updated: {}", bashrc_path.display());
        } else {
            println!("OK:      {} (already configured)", bashrc_path.display());
        }
    }

    println!();
    println!("Done! Restart your shell or run: source ~/.bashrc");

    Ok(())
}

/// Setup for fish
fn setup_fish(home: &str, force: bool) -> io::Result<()> {
    // Install completion script
    let fish_completion_dir = PathBuf::from(home).join(".config/fish/completions");
    let completion_path = fish_completion_dir.join("vfv.fish");
    if !completion_path.exists() || force {
        std::fs::create_dir_all(&fish_completion_dir)?;
        let completion_script = include_str!("../completions/vfv.fish");
        std::fs::write(&completion_path, completion_script)?;
        println!("Created: {}", completion_path.display());
    } else {
        println!("Exists:  {} (use --force to overwrite)", completion_path.display());
    }

    // Update config.fish for MANPATH
    let config_fish_path = PathBuf::from(home).join(".config/fish/config.fish");
    let config_fish_dir = PathBuf::from(home).join(".config/fish");
    std::fs::create_dir_all(&config_fish_dir)?;

    let config_content = if config_fish_path.exists() {
        std::fs::read_to_string(&config_fish_path)?
    } else {
        String::new()
    };

    if !config_content.contains(".local/share/man") {
        let mut new_content = config_content.clone();
        if !new_content.is_empty() && !new_content.ends_with('\n') {
            new_content.push('\n');
        }
        new_content.push_str("\n# vfv setup\n");
        new_content.push_str("set -gx MANPATH $HOME/.local/share/man $MANPATH\n");
        std::fs::write(&config_fish_path, new_content)?;
        println!("Updated: {}", config_fish_path.display());
    } else {
        println!("OK:      {} (already configured)", config_fish_path.display());
    }

    println!();
    println!("Done! Restart your shell.");

    Ok(())
}

/// Generate man page to stdout
fn run_man_page() {
    let cmd = Cli::command();
    let man = clap_mangen::Man::new(cmd);
    let mut buffer = Vec::new();
    man.render(&mut buffer).expect("Failed to generate man page");
    io::Write::write_all(&mut io::stdout(), &buffer).expect("Failed to write man page");
}
