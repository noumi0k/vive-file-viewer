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
        InputMode::SearchResult => draw_search_results(frame, app, area),
        InputMode::Normal => draw_file_list(frame, app, area),
    }
}

fn draw_search_input(frame: &mut Frame, app: &App, area: Rect) {
    let title = if app.search_dirs_only {
        "Search Folders (Enter to search)"
    } else {
        "Search All (Enter to search)"
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::Yellow));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let hint = if app.search_dirs_only {
        "Type folder name and press Enter..."
    } else {
        "Type file or folder name and press Enter..."
    };

    let text = Paragraph::new(hint)
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

    let title = "Files".to_string();

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
    let title = app
        .browser
        .selected_entry()
        .map(|e| e.name.clone())
        .unwrap_or_else(|| "Preview".to_string());

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::Cyan));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let visible_height = inner_area.height as usize;
    app.set_preview_height(visible_height);

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

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let content = match app.input_mode {
        InputMode::SearchInput => {
            "Enter:search  Esc:cancel".to_string()
        }
        InputMode::SearchResult => {
            "j/k:select  Enter:open  /:re-search  Esc:cancel".to_string()
        }
        InputMode::Normal => {
            if let Some(ref msg) = app.status_message {
                msg.clone()
            } else {
                let is_file = app.browser.selected_entry().map(|e| !e.is_dir).unwrap_or(false);
                if is_file {
                    "q:quit  j/k:move  Enter:open  h:back  e:editor  /:search  D:folders  r:reload".to_string()
                } else {
                    "q:quit  j/k:move  Enter:open  h:back  /:search  D:folders  r:reload".to_string()
                }
            }
        }
        InputMode::Preview => {
            "j/k:scroll  g/G:top/bottom  e:editor  h/q:back".to_string()
        }
    };

    let style = match app.input_mode {
        InputMode::SearchInput | InputMode::SearchResult => Style::default().fg(Color::Yellow),
        InputMode::Preview => Style::default().fg(Color::Cyan),
        InputMode::Normal => Style::default().fg(Color::DarkGray),
    };

    let footer = Paragraph::new(content).style(style);
    frame.render_widget(footer, area);
}
