use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};

/// Maximum directory depth for file search
const MAX_SEARCH_DEPTH: usize = 10;
/// Score assigned to exact matches
const EXACT_MATCH_SCORE: u32 = 1000;

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

    pub fn search(
        &mut self,
        base_dir: &Path,
        query: &str,
        max_results: usize,
        dir_only: bool,
        exact: bool,
    ) -> Vec<SearchResult> {
        if query.is_empty() {
            return Vec::new();
        }

        let is_path_query = query.contains('/');
        let query_lower = query.to_lowercase();

        // クエリの最後のセグメントを取得（パスクエリ用）
        let query_last_segment = if is_path_query {
            query.rsplit('/').next().unwrap_or(query)
        } else {
            query
        };
        let query_last_segment_lower = query_last_segment.to_lowercase();

        // ファジーマッチ用パターン（exactモードでは使わない）
        let pattern = if !exact {
            Some(Pattern::new(
                query,
                CaseMatching::Smart,
                Normalization::Smart,
                AtomKind::Fuzzy,
            ))
        } else {
            None
        };

        let mut results: Vec<SearchResult> = Vec::new();

        let walker = WalkBuilder::new(base_dir)
            .hidden(false)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .max_depth(Some(MAX_SEARCH_DEPTH))
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

            let file_name_lower = file_name.to_lowercase();

            if exact {
                // 完全一致モード：ファイル名がクエリと完全一致（大文字小文字無視）
                let matches = if is_path_query {
                    // パスクエリの場合：パスにクエリが含まれ、かつファイル名が最後のセグメントと完全一致
                    let display_path_lower = display_path.to_lowercase();
                    display_path_lower.contains(&query_lower)
                        && file_name_lower == query_last_segment_lower
                } else {
                    // 通常：ファイル名がクエリと完全一致
                    file_name_lower == query_lower
                };

                if matches {
                    results.push(SearchResult {
                        path: path.to_path_buf(),
                        display_path,
                        score: EXACT_MATCH_SCORE,
                        is_dir,
                    });
                }
            } else {
                // ファジーマッチモード
                let target = if is_path_query {
                    &display_path
                } else {
                    &file_name
                };
                let mut buf = Vec::new();
                let haystack = Utf32Str::new(target, &mut buf);

                if let Some(ref pat) = pattern
                    && let Some(score) = pat.score(haystack, &mut self.matcher)
                {
                    // パスクエリの場合、ファイル名がクエリの最後のセグメントを含まないものは除外
                    if is_path_query && !file_name_lower.contains(&query_last_segment_lower) {
                        continue;
                    }

                    results.push(SearchResult {
                        path: path.to_path_buf(),
                        display_path,
                        score,
                        is_dir,
                    });
                }
            }
        }

        // スコアで降順ソート
        results.sort_by(|a, b| b.score.cmp(&a.score));
        results.truncate(max_results);
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::TempDir;

    fn setup_test_dir() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // Create test structure
        fs::create_dir_all(base.join("src")).unwrap();
        fs::create_dir_all(base.join("tests")).unwrap();
        fs::create_dir_all(base.join("docs/api")).unwrap();

        File::create(base.join("src/main.rs")).unwrap();
        File::create(base.join("src/lib.rs")).unwrap();
        File::create(base.join("src/config.rs")).unwrap();
        File::create(base.join("tests/test_main.rs")).unwrap();
        File::create(base.join("docs/api/readme.md")).unwrap();
        File::create(base.join("README.md")).unwrap();

        temp_dir
    }

    #[test]
    fn test_empty_query_returns_empty() {
        let temp_dir = setup_test_dir();
        let mut searcher = FileSearcher::new();
        let results = searcher.search(temp_dir.path(), "", 10, false, false);
        assert!(results.is_empty());
    }

    #[test]
    fn test_fuzzy_search_finds_files() {
        let temp_dir = setup_test_dir();
        let mut searcher = FileSearcher::new();
        let results = searcher.search(temp_dir.path(), "main", 10, false, false);
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.display_path.contains("main")));
    }

    #[test]
    fn test_exact_match() {
        let temp_dir = setup_test_dir();
        let mut searcher = FileSearcher::new();
        let results = searcher.search(temp_dir.path(), "main.rs", 10, false, true);
        assert!(!results.is_empty());
        assert!(
            results
                .iter()
                .all(|r| r.path.file_name().unwrap() == "main.rs")
        );
    }

    #[test]
    fn test_dir_only_mode() {
        let temp_dir = setup_test_dir();
        let mut searcher = FileSearcher::new();
        let results = searcher.search(temp_dir.path(), "src", 10, true, false);
        assert!(results.iter().all(|r| r.is_dir));
    }

    #[test]
    fn test_path_query() {
        let temp_dir = setup_test_dir();
        let mut searcher = FileSearcher::new();
        let results = searcher.search(temp_dir.path(), "src/main", 10, false, false);
        assert!(!results.is_empty());
        assert!(
            results
                .iter()
                .any(|r| r.display_path.contains("src") && r.display_path.contains("main"))
        );
    }

    #[test]
    fn test_max_results_limit() {
        let temp_dir = setup_test_dir();
        let mut searcher = FileSearcher::new();
        let results = searcher.search(temp_dir.path(), "r", 2, false, false);
        assert!(results.len() <= 2);
    }

    #[test]
    fn test_results_sorted_by_score() {
        let temp_dir = setup_test_dir();
        let mut searcher = FileSearcher::new();
        let results = searcher.search(temp_dir.path(), "main", 10, false, false);
        for i in 1..results.len() {
            assert!(results[i - 1].score >= results[i].score);
        }
    }

    #[test]
    fn test_exact_match_uses_constant_score() {
        let temp_dir = setup_test_dir();
        let mut searcher = FileSearcher::new();
        let results = searcher.search(temp_dir.path(), "main.rs", 10, false, true);
        assert!(!results.is_empty());
        // All exact matches should have EXACT_MATCH_SCORE
        for result in &results {
            assert_eq!(result.score, EXACT_MATCH_SCORE);
        }
    }

    #[test]
    fn test_constants_have_expected_values() {
        assert_eq!(MAX_SEARCH_DEPTH, 10);
        assert_eq!(EXACT_MATCH_SCORE, 1000);
    }

    #[test]
    fn test_max_results_zero_returns_empty() {
        let temp_dir = setup_test_dir();
        let mut searcher = FileSearcher::new();
        let results = searcher.search(temp_dir.path(), "main", 0, false, false);
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_nonexistent_directory() {
        let mut searcher = FileSearcher::new();
        let results = searcher.search(Path::new("/nonexistent/path"), "test", 10, false, false);
        assert!(results.is_empty());
    }

    #[test]
    fn test_path_query_with_deep_nesting() {
        let temp_dir = setup_test_dir();
        let mut searcher = FileSearcher::new();
        // Search for nested path
        let results = searcher.search(temp_dir.path(), "docs/api", 10, true, false);
        assert!(results.iter().any(|r| r.display_path.contains("api")));
    }

    #[test]
    fn test_exact_match_no_match() {
        let temp_dir = setup_test_dir();
        let mut searcher = FileSearcher::new();
        let results = searcher.search(temp_dir.path(), "nonexistent.xyz", 10, false, true);
        assert!(results.is_empty());
    }

    #[test]
    fn test_fuzzy_search_partial_match() {
        let temp_dir = setup_test_dir();
        let mut searcher = FileSearcher::new();
        // Search with partial name
        let results = searcher.search(temp_dir.path(), "mai", 10, false, false);
        assert!(results.iter().any(|r| r.display_path.contains("main")));
    }
}
