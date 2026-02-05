use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub path: PathBuf,
    pub display_path: String,
    pub score: u32,
    pub is_dir: bool,
}

pub struct FileSearcher {
    matcher: Matcher,
}

impl FileSearcher {
    pub fn new() -> Self {
        Self {
            matcher: Matcher::new(Config::DEFAULT),
        }
    }

    pub fn search(&mut self, base_dir: &Path, query: &str, max_results: usize, dir_only: bool) -> Vec<SearchResult> {
        if query.is_empty() {
            return Vec::new();
        }

        let pattern = Pattern::new(
            query,
            CaseMatching::Smart,
            Normalization::Smart,
            AtomKind::Fuzzy,
        );

        let mut results: Vec<SearchResult> = Vec::new();

        let walker = WalkBuilder::new(base_dir)
            .hidden(false)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .max_depth(Some(10))
            .build();

        for entry in walker.flatten() {
            let path = entry.path();
            let is_dir = path.is_dir();

            // ディレクトリのみモードの場合、ファイルをスキップ
            if dir_only && !is_dir {
                continue;
            }

            // ファイル/ディレクトリ名を取得
            let file_name = match path.file_name() {
                Some(name) => name.to_string_lossy().to_string(),
                None => continue,
            };

            // ベースディレクトリからの相対パスを取得（表示用）
            let display_path = path
                .strip_prefix(base_dir)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            if display_path.is_empty() {
                continue;
            }

            // クエリに/が含まれていればパス全体、なければファイル名のみでマッチ
            let target = if query.contains('/') { &display_path } else { &file_name };
            let mut buf = Vec::new();
            let haystack = Utf32Str::new(target, &mut buf);

            if let Some(score) = pattern.score(haystack, &mut self.matcher) {
                results.push(SearchResult {
                    path: path.to_path_buf(),
                    display_path,
                    score,
                    is_dir,
                });
            }
        }

        // スコアで降順ソート
        results.sort_by(|a, b| b.score.cmp(&a.score));
        results.truncate(max_results);
        results
    }
}
