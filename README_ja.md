# vfv (Vive File Viewer)

ファジー検索とシンタックスハイライト機能を備えた高速ターミナルファイルビューワー。Rust製。

![Rust](https://img.shields.io/badge/rust-1.85%2B-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

## 特徴

- Vim風キーバインドで高速ディレクトリ移動
- コードプレビューのシンタックスハイライト
- [nucleo](https://github.com/helix-editor/nucleo) による高速ファジー検索
- .gitignore 対応の再帰検索
- お好みのエディタで開く（vim, nvim, code など）
- TOML で設定可能

## インストール

### ソースから

```bash
git clone https://github.com/yourname/vive-file-viewer.git
cd vive-file-viewer
cargo install --path .
```

### ビルド済みバイナリ

[Releases](https://github.com/yourname/vive-file-viewer/releases) からダウンロードして PATH の通った場所に配置。

## 使い方

```bash
vfv              # カレントディレクトリを開く
vfv ~/projects   # 指定ディレクトリを開く
```

## キーバインド

### ファイルブラウザ

| キー | 動作 |
|-----|--------|
| `j` / `↓` | 下に移動 |
| `k` / `↑` | 上に移動 |
| `Enter` / `l` | ファイルを開く / ディレクトリに入る |
| `h` / `Backspace` | 親ディレクトリへ |
| `g` | 先頭へ |
| `G` | 末尾へ |
| `e` | エディタで開く |
| `/` | 全ファイル検索 |
| `D` | フォルダのみ検索 |
| `.` | 隠しファイル表示切替 |
| `r` | リロード |
| `q` | 終了 |

### ファイルプレビュー

| キー | 動作 |
|-----|--------|
| `j` / `↓` | 下にスクロール |
| `k` / `↑` | 上にスクロール |
| `g` | 先頭へ |
| `G` | 末尾へ |
| `e` | エディタで開く |
| `h` / `q` | ファイルブラウザに戻る |

### 検索入力

| キー | 動作 |
|-----|--------|
| (入力) | 検索クエリを入力 |
| `Enter` | 検索実行 |
| `Esc` | キャンセル |

### 検索結果

| キー | 動作 |
|-----|--------|
| `j` / `k` | 結果を選択 |
| `Enter` | 選択を開く |
| `/` | 再検索 |
| `Esc` | キャンセル |

## 設定

設定ファイルの場所：
- **macOS**: `~/Library/Application Support/vive-file-viewer/config.toml`
- **Linux**: `~/.config/vive-file-viewer/config.toml`
- **Windows**: `%APPDATA%\vive-file-viewer\config.toml`

```toml
# エディタコマンド
editor = "vim"
editor_args = []

# デフォルトで隠しファイルを表示
show_hidden = false

# プレビューの最大行数
preview_max_lines = 1000

# シンタックスハイライトのテーマ
# 選択肢: "base16-ocean.dark", "base16-eighties.dark",
#         "base16-mocha.dark", "Solarized (dark)", "Solarized (light)"
theme = "base16-ocean.dark"
```

## 依存ライブラリ

- [ratatui](https://github.com/ratatui-org/ratatui) - ターミナルUIフレームワーク
- [syntect](https://github.com/trishume/syntect) - シンタックスハイライト
- [nucleo](https://github.com/helix-editor/nucleo) - ファジーマッチング（Helixエディタで使用）
- [ignore](https://github.com/BurntSushi/ripgrep/tree/master/crates/ignore) - .gitignore対応（ripgrepから）

## ライセンス

MIT License - 詳細は [LICENSE](LICENSE) を参照。
