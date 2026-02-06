use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, InputMode};

pub fn draw(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(frame.area());

    draw_header(frame, app, chunks[0]);
    draw_main(frame, app, chunks[1]);
    draw_footer(frame, app, chunks[2]);
}

fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let (content, style) = match app.input_mode {
        InputMode::SearchInput | InputMode::SearchResult => {
            let text = format!("/{}", app.search_input);
            (text, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        }
        InputMode::Searching => {
            let spinner = app.spinner_char();
            let text = format!("{} /{}", spinner, app.search_input);
            (text, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        }
        _ => {
            let path_str = app.browser.current_dir.to_string_lossy().to_string();
            (path_str, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        }
    };
    let header = Paragraph::new(content).style(style);
    frame.render_widget(header, area);
}

fn draw_main(frame: &mut Frame, app: &mut App, area: Rect) {
    match app.input_mode {
        InputMode::Preview => draw_preview(frame, app, area),
        InputMode::SearchInput => draw_search_input(frame, app, area),
        InputMode::Searching => draw_searching(frame, app, area),
        InputMode::SearchResult => draw_search_results(frame, app, area),
        InputMode::Help => draw_help(frame, area),
        InputMode::Normal | InputMode::JumpInput => draw_file_list(frame, app, area),
    }
}

fn draw_search_input(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Search (Enter to search)")
        .border_style(Style::default().fg(Color::Yellow));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let help_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Usage: ", Style::default().fg(Color::White)),
            Span::styled("<query> [options]", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Options:", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("    -d, --dir    ", Style::default().fg(Color::Yellow)),
            Span::styled("Directories only", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled("    -e, --exact  ", Style::default().fg(Color::Yellow)),
            Span::styled("Exact match (no fuzzy)", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled("    -b, --base   ", Style::default().fg(Color::Yellow)),
            Span::styled("Search base directory", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Examples:", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("    main.rs      ", Style::default().fg(Color::Cyan)),
            Span::styled("Fuzzy search for main.rs", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled("    src/main -d  ", Style::default().fg(Color::Cyan)),
            Span::styled("Directories containing 'main' under 'src'", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled("    config -e    ", Style::default().fg(Color::Cyan)),
            Span::styled("Exact match for 'config'", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled("    main -b ~/dev", Style::default().fg(Color::Cyan)),
            Span::styled("Search 'main' under ~/dev", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let paragraph = Paragraph::new(help_lines);
    frame.render_widget(paragraph, inner_area);
}

fn draw_searching(frame: &mut Frame, app: &App, area: Rect) {
    let spinner = app.spinner_char();
    let title = format!("{} Searching: {}", spinner, app.search_input);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::Yellow));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let mode = if app.search_dirs_only { "folders" } else { "files" };
    let text = Paragraph::new(format!("Searching {} in {}...", mode, app.base_dir.display()))
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(text, inner_area);
}

fn draw_search_results(frame: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .search_results
        .iter()
        .map(|result| {
            let icon = if result.is_dir { " " } else { " " };
            let name = format!("{}{}", icon, result.display_path);

            let style = if result.is_dir {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(name).style(style)
        })
        .collect();

    let mode = if app.search_dirs_only { "Folders" } else { "All" };
    let title = format!("{}: {} ({} results)", mode, app.search_input, app.search_results.len());

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_stateful_widget(list, area, &mut app.search_list_state);
}

fn draw_file_list(frame: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .browser
        .entries
        .iter()
        .map(|entry| {
            let icon = if entry.is_dir { " " } else { " " };
            let name = format!("{}{}", icon, entry.name);

            let style = if entry.is_dir {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(name).style(style)
        })
        .collect();

    let total = app.browser.entries.len();
    let title = if total > 0 {
        format!("Files [{}/{}]", app.browser.selected_index + 1, total)
    } else {
        "Files [empty]".to_string()
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_stateful_widget(list, area, &mut app.list_state);
}

fn draw_preview(frame: &mut Frame, app: &mut App, area: Rect) {
    let file_name = app
        .browser
        .selected_entry()
        .map(|e| e.name.clone())
        .unwrap_or_else(|| "Preview".to_string());

    // 一時的にinner_areaを計算するためのブロック
    let temp_block = Block::default().borders(Borders::ALL);
    let inner_area = temp_block.inner(area);
    let visible_height = inner_area.height as usize;
    app.set_preview_height(visible_height);

    // タイトルに位置情報を追加
    let title = if let Some(ref content) = app.preview_content {
        let total = content.lines.len();
        let current_line = app.preview_scroll + 1;
        let end_line = (app.preview_scroll + visible_height).min(total);
        format!("{} [{}-{}/{}]", file_name, current_line, end_line, total)
    } else {
        file_name
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::Cyan));

    frame.render_widget(block, area);

    if let Some(ref content) = app.preview_content {
        let start = app.preview_scroll;
        let end = (start + visible_height).min(content.lines.len());

        let lines: Vec<Line> = content.lines[start..end]
            .iter()
            .map(|preview_line| {
                let mut spans = vec![Span::styled(
                    format!("{:4} ", preview_line.line_number),
                    Style::default().fg(Color::DarkGray),
                )];

                for (style, text) in &preview_line.segments {
                    let fg = Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
                    spans.push(Span::styled(text.clone(), Style::default().fg(fg)));
                }

                Line::from(spans)
            })
            .collect();

        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
        frame.render_widget(paragraph, inner_area);
    } else if let Some(entry) = app.browser.selected_entry() {
        if entry.is_dir {
            let text = Paragraph::new("[Directory]")
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(text, inner_area);
        }
    }
}

fn draw_help(frame: &mut Frame, area: Rect) {
    let help_text = vec![
        "",
        "  vfv - Vive File Viewer",
        "",
        "  === File Browser ===",
        "  j/k, ↑/↓     Move up/down",
        "  Enter, l     Open file / Enter directory",
        "  h, Backspace Go to parent directory",
        "  g/G          Go to top/bottom",
        "  e            Open in editor",
        "  y            Copy path to clipboard",
        "  f + char     Jump to entry starting with char",
        "  ;            Jump to next match",
        "  ,            Jump to previous match",
        "  /            Search all files (fuzzy)",
        "  D            Search folders only",
        "  .            Toggle hidden files",
        "  r            Reload",
        "  ?            Show this help",
        "  q            Quit",
        "",
        "  === Preview ===",
        "  j/k          Scroll up/down",
        "  Ctrl+d/u     Half page down/up",
        "  Ctrl+f/b     Page down/up",
        "  g/G          Go to top/bottom",
        "  e            Open in editor",
        "  h/q          Back to browser",
        "",
        "  Press q or ? to close",
    ];

    let lines: Vec<Line> = help_text
        .iter()
        .map(|&s| Line::from(s))
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Help")
        .border_style(Style::default().fg(Color::Green));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .style(Style::default().fg(Color::White));

    frame.render_widget(paragraph, area);
}

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let content = match app.input_mode {
        InputMode::SearchInput => {
            "Enter:search  Esc:cancel".to_string()
        }
        InputMode::Searching => {
            "Searching...  Esc:cancel".to_string()
        }
        InputMode::SearchResult => {
            "j/k:select  Enter:open  /:re-search  Esc:cancel".to_string()
        }
        InputMode::JumpInput => {
            "Type a character to jump...".to_string()
        }
        InputMode::Normal => {
            if let Some(ref msg) = app.status_message {
                msg.clone()
            } else {
                let is_file = app.browser.selected_entry().map(|e| !e.is_dir).unwrap_or(false);
                let jump_hint = if let Some(c) = app.last_jump_char {
                    format!("  ;/,:next/prev '{}'", c)
                } else {
                    String::new()
                };
                if is_file {
                    format!("q:quit  j/k:move  f:jump{}  Enter:open  e:editor  /:search", jump_hint)
                } else {
                    format!("q:quit  j/k:move  f:jump{}  Enter:open  /:search", jump_hint)
                }
            }
        }
        InputMode::Preview => {
            "j/k:scroll  g/G:top/bottom  e:editor  h/q:back".to_string()
        }
        InputMode::Help => {
            "Press q or ? to close".to_string()
        }
    };

    let style = match app.input_mode {
        InputMode::SearchInput | InputMode::SearchResult | InputMode::Searching => Style::default().fg(Color::Yellow),
        InputMode::JumpInput | InputMode::Help => Style::default().fg(Color::Green),
        InputMode::Preview => Style::default().fg(Color::Cyan),
        InputMode::Normal => Style::default().fg(Color::DarkGray),
    };

    let footer = Paragraph::new(content).style(style);
    frame.render_widget(footer, area);
}
