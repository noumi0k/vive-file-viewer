# vfv (Vive File Viewer)

[日本語](README_ja.md)

**The ultra-lightweight terminal file viewer for vibe coding.**

When you're in the flow with AI pair programming, you just want to quickly browse and check files—no complex setup, no heavy features. Just open, search, view, and get back to coding.

![Rust](https://img.shields.io/badge/rust-1.85%2B-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

## Why vfv?

- **Zero config fuzzy search** - Built-in [nucleo](https://github.com/helix-editor/nucleo) (same as Helix editor). No fzf setup needed.
- **4MB single binary** - Install and run. That's it.
- **Vim keybindings** - Navigate like you're used to.
- **Syntax highlighting** - Preview code with colors.
- **.gitignore aware** - Powered by ripgrep's ignore crate.

### Not for you if...

You need file management (copy, move, delete), image preview, or plugin ecosystem. Use [yazi](https://github.com/sxyazi/yazi) instead.

## Install

```bash
# From source
git clone https://github.com/yourname/vive-file-viewer.git
cd vive-file-viewer
cargo install --path .

# Or download binary from Releases
```

## Usage

```bash
vfv              # Browse current directory
vfv ~/projects   # Browse specific directory
```

## Keybindings

Press `?` to show help screen.

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
| `y` | Copy path to clipboard |
| `f` + char | Jump to entry starting with char |
| `;` | Jump to next match |
| `,` | Jump to previous match |
| `/` | Search all files |
| `D` | Search folders only |
| `.` | Toggle hidden files |
| `r` | Reload |
| `?` | Show help |
| `q` | Quit |

### File Preview

| Key | Action |
|-----|--------|
| `j` / `↓` | Scroll down |
| `k` / `↑` | Scroll up |
| `Ctrl+d` | Half page down |
| `Ctrl+u` | Half page up |
| `Ctrl+f` / `PageDown` | Page down |
| `Ctrl+b` / `PageUp` | Page up |
| `g` | Go to top |
| `G` | Go to bottom |
| `e` | Open in editor |
| `h` / `q` | Back to file browser |

### Search

| Key | Action |
|-----|--------|
| (type) | Enter search query |
| `Enter` | Execute search |
| `Esc` | Cancel |

### Search Results

| Key | Action |
|-----|--------|
| `j` / `k` / `Tab` | Select result |
| `Enter` | Open selected |
| `/` | New search |
| `Esc` | Cancel |

## Configuration

Config file location:
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

## License

MIT
