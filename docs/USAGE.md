# Usage Guide

## Quick Start

```bash
# Install
cargo install --path .

# Browse current directory
vfv

# Browse specific directory
vfv ~/dev
```

## TUI Navigation

### File Browser

- `j`/`k` or arrow keys - Move up/down
- `Enter` or `l` - Enter directory / Preview file
- `h` or `Backspace` - Go to parent directory
- `g`/`G` - Jump to first/last item
- `f` + char - Jump to entry starting with char
- `;`/`,` - Next/previous jump match

### File Preview

The right pane shows the selected file with:
- Line numbers
- Syntax highlighting (auto-detected)
- Scroll: `j`/`k`, `Ctrl+d`/`Ctrl+u`, `PageUp`/`PageDown`

### Search

1. Press `/` to enter search mode
2. Type query with options: `main -d -e -b ~/dev`
3. Press `Enter` to execute

Options:
- `-d` - Directories only
- `-e` - Exact match
- `-b <path>` - Search base directory

## CLI Search

```bash
# Fuzzy search
vfv find "config" ~/dev

# Find directory and cd
cd $(vfv find "project" ~/dev -d -1 -q)

# AI-friendly output
vfv find "main" ~/dev -q -j -c -t 5
```

## Configuration

Config location:
- macOS: `~/Library/Application Support/vive-file-viewer/config.toml`
- Linux: `~/.config/vive-file-viewer/config.toml`

```toml
editor = "vim"
editor_args = []
show_hidden = false
preview_max_lines = 1000
theme = "base16-ocean.dark"
```

### Available Themes

- `base16-ocean.dark` (default)
- `base16-eighties.dark`
- `base16-mocha.dark`
- `Solarized (dark)`
- `Solarized (light)`
