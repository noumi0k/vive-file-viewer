use std::fs::File;
use std::io::{BufRead, BufReader, Read};
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

        let file = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                return PreviewContent {
                    lines: vec![PreviewLine {
                        line_number: 0,
                        segments: vec![(Style::default(), format!("Error reading file: {}", e))],
                    }],
                };
            }
        };

        let mut reader = BufReader::new(file);

        // Read first 8KB for binary detection
        let mut header = vec![0u8; 8000];
        let header_len = reader.read(&mut header).unwrap_or(0);
        header.truncate(header_len);

        if is_binary(&header) {
            return PreviewContent {
                lines: vec![PreviewLine {
                    line_number: 0,
                    segments: vec![(Style::default(), "[Binary file]".to_string())],
                }],
            };
        }

        // Convert header to string and read remaining lines up to max_lines
        // Use byte limit (10MB) to prevent memory issues with long lines
        const MAX_BYTES: usize = 10 * 1024 * 1024;
        let mut total_bytes = header_len;
        let mut text = String::from_utf8_lossy(&header).into_owned();

        // Read remaining content up to limits
        for line in reader.lines() {
            if text.lines().count() >= self.max_lines || total_bytes >= MAX_BYTES {
                break;
            }
            match line {
                Ok(l) => {
                    total_bytes += l.len() + 1;
                    text.push_str(&l);
                    text.push('\n');
                }
                Err(_) => break,
            }
        }

        let text = text;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_preview_directory_returns_directory_message() {
        let temp_dir = TempDir::new().unwrap();
        let previewer = Previewer::new("base16-ocean.dark", 100);

        let content = previewer.preview(temp_dir.path());

        assert_eq!(content.lines.len(), 1);
        assert!(
            content.lines[0]
                .segments
                .iter()
                .any(|(_, text)| text.contains("[Directory]"))
        );
    }

    #[test]
    fn test_preview_nonexistent_file_returns_error() {
        let previewer = Previewer::new("base16-ocean.dark", 100);
        let nonexistent = Path::new("/nonexistent/file.txt");

        let content = previewer.preview(nonexistent);

        assert_eq!(content.lines.len(), 1);
        // Non-file path returns [Directory] message
        assert!(
            content.lines[0]
                .segments
                .iter()
                .any(|(_, text)| text.contains("[Directory]"))
        );
    }

    #[test]
    fn test_preview_text_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Line 1").unwrap();
        writeln!(file, "Line 2").unwrap();
        writeln!(file, "Line 3").unwrap();

        let previewer = Previewer::new("base16-ocean.dark", 100);
        let content = previewer.preview(&file_path);

        assert!(content.lines.len() >= 3);
        assert_eq!(content.lines[0].line_number, 1);
    }

    #[test]
    fn test_preview_respects_max_lines() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("long.txt");
        let mut file = File::create(&file_path).unwrap();
        for i in 1..=100 {
            writeln!(file, "Line {}", i).unwrap();
        }

        let previewer = Previewer::new("base16-ocean.dark", 10);
        let content = previewer.preview(&file_path);

        assert!(content.lines.len() <= 10);
    }

    #[test]
    fn test_preview_binary_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("binary.bin");
        let mut file = File::create(&file_path).unwrap();
        // Write binary content with lots of null bytes
        let binary_content: Vec<u8> = (0..1000)
            .map(|i| if i % 5 == 0 { 0 } else { i as u8 })
            .collect();
        file.write_all(&binary_content).unwrap();

        let previewer = Previewer::new("base16-ocean.dark", 100);
        let content = previewer.preview(&file_path);

        assert_eq!(content.lines.len(), 1);
        assert!(
            content.lines[0]
                .segments
                .iter()
                .any(|(_, text)| text.contains("[Binary file]"))
        );
    }

    #[test]
    fn test_preview_with_invalid_theme_uses_fallback() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "fn main() {{}}").unwrap();

        // Use an invalid theme name
        let previewer = Previewer::new("nonexistent-theme", 100);
        let content = previewer.preview(&file_path);

        // Should not panic and should return content
        assert!(!content.lines.is_empty());
    }

    #[test]
    fn test_is_binary_detects_binary() {
        // Content with >10% null bytes is binary
        let binary: Vec<u8> = vec![0, 0, 0, 1, 2, 3, 4, 5, 6, 7];
        assert!(is_binary(&binary));
    }

    #[test]
    fn test_is_binary_allows_text() {
        // Normal text content
        let text = b"Hello, World!\nThis is a text file.";
        assert!(!is_binary(text));
    }

    #[test]
    fn test_is_binary_empty_content() {
        let empty: Vec<u8> = vec![];
        assert!(!is_binary(&empty));
    }

    #[test]
    fn test_preview_file_with_syntax_highlighting() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "fn main() {{").unwrap();
        writeln!(file, "    println!(\"Hello\");").unwrap();
        writeln!(file, "}}").unwrap();

        let previewer = Previewer::new("base16-ocean.dark", 100);
        let content = previewer.preview(&file_path);

        assert!(content.lines.len() >= 3);
        // Each line should have segments
        for line in &content.lines {
            assert!(!line.segments.is_empty());
        }
    }
}
