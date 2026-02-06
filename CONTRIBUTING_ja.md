# vfv への貢献

vfv への貢献に興味を持っていただきありがとうございます！

## 開発環境のセットアップ

### 必要なもの

- Rust 1.85 以降
- Git

### ソースからビルド

```bash
git clone https://github.com/noumi0k/vive-file-viewer.git
cd vive-file-viewer
cargo build
```

### テストの実行

```bash
cargo test
```

### デバッグ実行

```bash
cargo run -- ~/some/directory
```

## プルリクエストの手順

1. リポジトリをフォーク
2. フィーチャーブランチを作成 (`git checkout -b feature/amazing-feature`)
3. 変更を加える
4. `cargo fmt` と `cargo clippy` を実行
5. `cargo test` でテストを実行
6. 説明的なメッセージでコミット
7. ブランチにプッシュ (`git push origin feature/amazing-feature`)
8. プルリクエストを作成

## コードスタイル

- Rust標準フォーマットに従う (`cargo fmt`)
- clippyの警告をすべて解消する (`cargo clippy`)
- 関数は小さく、焦点を絞る
- 自明でないロジックにはコメントを追加

## コミットメッセージ

- 命令形を使う（「Add feature」であって「Added feature」ではない）
- 1行目は72文字以内
- 該当する場合はissueを参照

## Issue報告

Issue報告時は以下を含めてください：

- vfvのバージョン (`vfv --version`)
- OSとバージョン
- 再現手順
- 期待される動作と実際の動作

## メンテナー向け

### リリース手順

1. `CHANGELOG.md` を新バージョンで更新
2. `Cargo.toml` のバージョンを更新
3. コミット: `git commit -am "Release vX.Y.Z"`
4. タグ付け: `git tag vX.Y.Z`
5. プッシュ: `git push && git push origin vX.Y.Z`

GitHub Actionsが自動的に：
- Linux/macOS/Windows用のバイナリをビルド
- バイナリ付きのGitHub Releaseを作成
- crates.ioに公開

## ライセンス

貢献することで、あなたの貢献がMITライセンスの下でライセンスされることに同意したものとみなされます。
