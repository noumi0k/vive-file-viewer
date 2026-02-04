# vfv (Vive File Viewer)

**バイブコーディングのための超軽量ターミナルファイルビューワー**

AIとペアプロしてる最中に、ちょっとファイルを確認したいだけ。複雑な設定も重い機能もいらない。開いて、検索して、見て、コーディングに戻る。それだけ。

![Rust](https://img.shields.io/badge/rust-1.85%2B-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

## なぜ vfv？

- **設定不要のファジー検索** - [nucleo](https://github.com/helix-editor/nucleo)（Helixエディタと同じ）内蔵。fzfの設定不要。
- **4MBの単一バイナリ** - インストールして即実行。
- **Vimキーバインド** - 慣れた操作感。
- **シンタックスハイライト** - コードをカラーでプレビュー。
- **.gitignore対応** - ripgrepのignoreクレート使用。

### こんな人には向かない

ファイル管理（コピー、移動、削除）、画像プレビュー、プラグインが必要なら [yazi](https://github.com/sxyazi/yazi) を使ってください。

## インストール

```bash
# ソースから
git clone https://github.com/yourname/vive-file-viewer.git
cd vive-file-viewer
cargo install --path .

# またはReleasesからバイナリをダウンロード
```

## 使い方

```bash
vfv              # カレントディレクトリを開く
vfv ~/projects   # 指定ディレクトリを開く
```

## キーバインド

`?` でヘルプ画面を表示できます。

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
| `y` | パスをクリップボードにコピー |
| `f` + 文字 | その文字で始まるエントリにジャンプ |
| `;` | 次のマッチへジャンプ |
| `,` | 前のマッチへジャンプ |
| `/` | 全ファイル検索 |
| `D` | フォルダのみ検索 |
| `.` | 隠しファイル表示切替 |
| `r` | リロード |
| `?` | ヘルプ表示 |
| `q` | 終了 |

### ファイルプレビュー

| キー | 動作 |
|-----|--------|
| `j` / `↓` | 下にスクロール |
| `k` / `↑` | 上にスクロール |
| `Ctrl+d` | 半ページ下 |
| `Ctrl+u` | 半ページ上 |
| `Ctrl+f` / `PageDown` | 1ページ下 |
| `Ctrl+b` / `PageUp` | 1ページ上 |
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
| `j` / `k` / `Tab` | 結果を選択 |
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

## ライセンス

MIT
