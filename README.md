# vfv (Vive File Viewer)

[日本語](README_ja.md)

A fast terminal file viewer with fuzzy search and syntax highlighting, built with Rust.

![Rust](https://img.shields.io/badge/rust-1.85%2B-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

## Features

- Fast directory navigation with vim-like keybindings
- Syntax highlighting for code preview
- Fuzzy file search powered by [nucleo](https://github.com/helix-editor/nucleo)
- Recursive search with .gitignore support
- Open files in your favorite editor (vim, nvim, code, etc.)
- Configurable via TOML

## Installation

### From source

```bash
git clone https://github.com/yourname/vive-file-viewer.git
cd vive-file-viewer
cargo install --path .
```

### Pre-built binary

Download from [Releases](https://github.com/yourname/vive-file-viewer/releases) and place in your PATH.

## Usage

```bash
vfv              # Browse current directory
vfv ~/projects   # Browse specific directory
```

## Key Bindings

### File Browser

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `Enter` / `l` | Open file / Enter directory |
| `h` / `Backspace` | Go to parent directory |
| `g` | Go to top |
| `G` | Go to bottom |
| `e` | Open in editor |
| `/` | Search all files |
| `D` | Search folders only |
| `.` | Toggle hidden files |
| `r` | Reload |
| `q` | Quit |

### File Preview

| Key | Action |
|-----|--------|
| `j` / `↓` | Scroll down |
| `k` / `↑` | Scroll up |
| `g` | Go to top |
| `G` | Go to bottom |
| `e` | Open in editor |
| `h` / `q` | Back to file browser |

### Search Mode

| Key | Action |
|-----|--------|
| (type) | Enter search query |
| `Enter` | Execute search |
| `Esc` | Cancel |

### Search Results

| Key | Action |
|-----|--------|
| `j` / `k` | Select result |
| `Enter` | Open selected |
| `/` | New search |
| `Esc` | Cancel |

## Configuration

Create a config file at:
- **macOS**: `~/Library/Application Support/vive-file-viewer/config.toml`
- **Linux**: `~/.config/vive-file-viewer/config.toml`
- **Windows**: `%APPDATA%\vive-file-viewer\config.toml`

```toml
# Editor command
editor = "vim"
editor_args = []

# Show hidden files by default
show_hidden = false

# Maximum lines to preview
preview_max_lines = 1000

# Syntax highlighting theme
# Options: "base16-ocean.dark", "base16-eighties.dark",
#          "base16-mocha.dark", "Solarized (dark)", "Solarized (light)"
theme = "base16-ocean.dark"
```

## Dependencies

- [ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- [syntect](https://github.com/trishume/syntect) - Syntax highlighting
- [nucleo](https://github.com/helix-editor/nucleo) - Fuzzy matching (used in Helix editor)
- [ignore](https://github.com/BurntSushi/ripgrep/tree/master/crates/ignore) - .gitignore support (from ripgrep)

## License

MIT License - see [LICENSE](LICENSE) for details.
