# Tasks: v51.8.0 — ドキュメントサイト Performance 記事

Status: COMPLETE
Date: 2026-07-20

---

## T0 — 事前確認

- [x] `cargo test` 3130 passed, 0 failed を確認（ベース確認）
- [x] `cargo clippy -- -D warnings` クリーンであることを確認
- [x] `site/content/docs/runtime/` ディレクトリが**存在しない**ことを確認（新規作成対象）
- [x] `site/content/docs/tools/bench-regression.mdx` が**存在しない**ことを確認（新規作成対象）
- [x] `include_str!` パスの確認:
  - [x] `driver.rs` は `fav/src/driver.rs`
  - [x] `../../site/content/docs/runtime/parallel.mdx` が `favnir/site/content/docs/runtime/parallel.mdx` を指すことを確認
  - [x] `../../site/content/docs/tools/bench-regression.mdx` が `favnir/site/content/docs/tools/bench-regression.mdx` を指すことを確認

## T1 — `site/content/docs/runtime/parallel.mdx` 作成

- [x] `site/content/docs/runtime/` ディレクトリを新規作成
- [x] `site/content/docs/runtime/parallel.mdx` を新規作成:
  - [x] frontmatter（`title` / `description`）を含む
  - [x] `par` キーワードを含む（`par [A, B] |> Merge` 構文説明）
  - [x] `Merge` キーワードを含む（`Merge.ordered` / `Merge.any` 説明）
  - [x] `buffer_size` キーワードを含む（`fav.toml [stream] buffer_size` 設定説明）
  - [x] 実用的な Favnir コード例を含む

## T2 — `site/content/docs/tools/bench-regression.mdx` 作成

- [x] `site/content/docs/tools/bench-regression.mdx` を新規作成:
  - [x] frontmatter（`title` / `description`）を含む
  - [x] `--compare` キーワードを含む（`fav bench --compare <baseline.json>` 説明）
  - [x] `--fail-on-regression` キーワードを含む（CI 向けフラグ説明）
  - [x] CLI 出力例（`+50% WARN` / `-5% OK` 等）を含む
  - [x] `benchmarks/` ディレクトリ管理ポリシーの説明を含む

## T3 — `v51800_tests` 追加 + バージョン更新

- [x] `driver.rs` の `v51700_tests` 直前に `v51800_tests` モジュールを追加（2 件）:
  - [x] `docs_parallel_page_exists`:
    - [x] `include_str!("../../site/content/docs/runtime/parallel.mdx")` で読み込む
    - [x] `src.contains("par")` を assert
    - [x] `src.contains("Merge")` を assert
    - [x] `src.contains("buffer_size")` を assert
  - [x] `docs_bench_regression_page_exists`:
    - [x] `include_str!("../../site/content/docs/tools/bench-regression.mdx")` で読み込む
    - [x] `src.contains("--compare")` を assert
    - [x] `src.contains("--fail-on-regression")` を assert
- [x] `v51700_tests` から `cargo_toml_version_is_51_7_0` を削除
- [x] `fav/Cargo.toml` version → `"51.8.0"`
- [x] `cargo test` 3131 passed, 0 failed（3130 - 1 削除 + 2 新規 = 3131）
- [x] `cargo clippy -- -D warnings` クリーン

## T4 — 後処理

- [x] `CHANGELOG.md` に v51.8.0 エントリ追加
- [x] `versions/current.md` を v51.8.0（3131 tests）に更新
- [x] `roadmap-v51.1-v52.0.md` の v51.8.0 実績欄を更新
- [x] tasks.md を COMPLETE に更新（T0〜T4 全 `[x]`）
- [x] `roadmap-v51.1-v52.0.md` の v51.9.0 実績推定値（3131）が v51.8.0 完了後テスト数と一致することを確認

## code-review 対応（2026-07-20）

- [x] [MED] `parallel.mdx` から Rust 内部実装詳細（`std::thread::spawn` / `tokio::join_all` / `FuturesUnordered`）を除去
- [x] [MED] `parallel.mdx` バックプレッシャーのコード例を型整合性のある形に修正（`Ok(order)` → `Order.from_raw(raw)`、`raw` 未使用を解消）
- [x] [LOW] `parallel.mdx` `Arc<Mutex<T>>` の注意事項を `!Cache` エフェクト経由の記述に差し替え、OOM リスク警告を追加
- [x] [LOW] `bench-regression.mdx` `--fail-on-regression` は `--compare` と併用必須である旨を明記
- [x] `cargo test` 3131 passed, 0 failed を確認
