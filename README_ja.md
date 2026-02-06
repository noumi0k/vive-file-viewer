# vfv (Vive File Viewer)

**バイブコーディングのための超軽量ターミナルファイルビューワー**

バイブコーディングをしてる最中に、ちょっとファイルを確認したいだけ。複雑な設定も重い機能もいらない。開いて、検索して、見て、コーディングに戻る。それだけ。

[![CI](https://github.com/noumi0k/vive-file-viewer/actions/workflows/ci.yml/badge.svg)](https://github.com/noumi0k/vive-file-viewer/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/vfv)](https://crates.io/crates/vfv)
![Rust](https://img.shields.io/badge/rust-1.85%2B-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

https://github.com/user-attachments/assets/e3ec4515-ac0d-453f-851f-ca449b19e877

## なぜ vfv？

- **設定不要のファジー検索** - [nucleo](https://github.com/helix-editor/nucleo)（Helixエディタと同じ）内蔵。fzfの設定不要。
- **3MBの単一バイナリ** - インストールして即実行。
- **Vimキーバインド** - 慣れた操作感。
- **シンタックスハイライト** - コードをカラーでプレビュー。
- **.gitignore対応** - ripgrepのignoreクレート使用。

### こんな人には向かない

ファイル管理（コピー、移動、削除）、画像プレビュー、プラグインが必要なら [yazi](https://github.com/sxyazi/yazi) を使ってください。

## インストール

```bash
cargo install vfv
```

または [Releases](https://github.com/noumi0k/vive-file-viewer/releases) からビルド済みバイナリをダウンロード。

<details>
<summary>ソースからビルド</summary>

```bash
git clone https://github.com/noumi0k/vive-file-viewer.git
cd vive-file-viewer
cargo install --path .
```
</details>

## セットアップ

セットアップコマンドで全て自動設定：

```bash
vfv init
```

以下を自動で行います：
- 設定ファイル作成
- シェル補完インストール（zsh/bash/fish）
- manページインストール
- シェルのrcファイル更新

対応シェル：**zsh**、**bash**、**fish**

既存ファイルを上書きするには `--force` を使用。

### 設定ファイル

場所：
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

## 使い方

```bash
vfv              # カレントディレクトリを開く（TUI）
vfv ~/projects   # 指定ディレクトリを開く（TUI）
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
| `/` | 検索（オプション付き） |
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

`/`で検索を開始。CLIと同じオプションが使えます：

```
main.rs           # ファジー検索
config -e         # 完全一致
src/main -d       # ディレクトリのみ、パスマッチ
telemo -d -e      # ディレクトリのみ＋完全一致
main -b ~/dev     # 指定ディレクトリを起点に検索
```

| キー | 動作 |
|-----|--------|
| (入力) | クエリとオプションを入力 |
| `Enter` | 検索実行 |
| `Esc` | キャンセル |

### 検索結果

| キー | 動作 |
|-----|--------|
| `j` / `k` / `Tab` | 結果を選択 |
| `Enter` | 選択を開く |
| `/` | 再検索 |
| `Esc` | キャンセル |

## CLI検索

コマンドラインから直接ファイル検索。AIアシスタントやシェルスクリプトから呼び出す想定。

```bash
vfv find <query> [path]    # ファジー検索
```

### オプション

| オプション | 説明 |
|-----------|------|
| `-d, --dir` | ディレクトリのみ検索 |
| `-e, --exact` | 完全一致（ファジーなし） |
| `-n, --limit <N>` | 最大件数（デフォルト: 20） |
| `-1, --first` | 最上位1件のみ出力 |
| `-j, --json` | JSON形式で出力 |
| `-c, --compact` | コンパクトJSON（1行） |
| `-t, --timeout <秒>` | タイムアウト秒数（デフォルト: 0 = 無制限） |
| `-q, --quiet` | スピナー非表示（スクリプト/AI用） |

### パスマッチ

クエリに`/`を含む場合、パス全体でマッチ：

```bash
vfv find "src/main" ~/dev    # src/*/main* にマッチ
vfv find "main" ~/dev        # ファイル名のみでマッチ
```

### 使用例

```bash
# 基本検索
vfv find "config" ~/dev

# ディレクトリを検索してcd
cd $(vfv find "project" ~/dev -d -1 -q)

# パス検索: どこかの dev ディレクトリ配下の telemo
vfv find "dev/telemo" ~ -d

# 完全一致
vfv find "config" ~/dev -e

# AI向け: quiet、コンパクトJSON、タイムアウト付き
vfv find "main" ~/dev -q -j -c -t 5
```

### 終了コード

| コード | 意味 |
|-------|------|
| 0 | 結果あり |
| 1 | 結果なし |
| 124 | タイムアウト |

## ライセンス

MIT
