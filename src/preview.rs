use std::fs;
use std::path::Path;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

pub struct PreviewContent {
    pub lines: Vec<PreviewLine>,
}

pub struct PreviewLine {
    pub line_number: usize,
    pub segments: Vec<(Style, String)>,
}

pub struct Previewer {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    theme_name: String,
    max_lines: usize,
}

impl Previewer {
    pub fn new(theme_name: &str, max_lines: usize) -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            theme_name: theme_name.to_string(),
            max_lines,
        }
    }

    pub fn preview(&self, path: &Path) -> PreviewContent {
        if !path.is_file() {
            return PreviewContent {
                lines: vec![PreviewLine {
                    line_number: 0,
                    segments: vec![(Style::default(), "[Directory]".to_string())],
                }],
            };
        }

        let content = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(e) => {
                return PreviewContent {
                    lines: vec![PreviewLine {
                        line_number: 0,
                        segments: vec![(Style::default(), format!("Error reading file: {}", e))],
                    }],
                };
            }
        };

        if is_binary(&content) {
            return PreviewContent {
                lines: vec![PreviewLine {
                    line_number: 0,
                    segments: vec![(Style::default(), "[Binary file]".to_string())],
                }],
            };
        }

        let text = String::from_utf8_lossy(&content);

        let syntax = self
            .syntax_set
            .find_syntax_for_file(path)
            .ok()
            .flatten()
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let theme = self
            .theme_set
            .themes
            .get(&self.theme_name)
            .unwrap_or_else(|| {
                self.theme_set
                    .themes
                    .values()
                    .next()
                    .expect("No themes available")
            });

        let mut highlighter = HighlightLines::new(syntax, theme);
        let mut lines = Vec::new();

        for (line_num, line) in LinesWithEndings::from(&text).enumerate() {
            if line_num >= self.max_lines {
                break;
            }

            let ranges = highlighter
                .highlight_line(line, &self.syntax_set)
                .unwrap_or_default();

            let segments: Vec<(Style, String)> = ranges
                .into_iter()
                .map(|(style, text)| (style, text.to_string()))
                .collect();

            lines.push(PreviewLine {
                line_number: line_num + 1,
                segments,
            });
        }

        PreviewContent { lines }
    }
}

fn is_binary(content: &[u8]) -> bool {
    let check_len = content.len().min(8000);
    let null_count = content[..check_len].iter().filter(|&&b| b == 0).count();
    null_count > check_len / 10
}
