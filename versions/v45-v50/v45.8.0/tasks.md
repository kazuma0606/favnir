# Tasks: v45.8.0 — examples 更新 Phase 1

Status: COMPLETE
Date: 2026-07-16

---

## T0 — 事前確認

- [x] `cargo test` 2985 passed, 0 failed を確認

## T1 — `examples/pipeline/pipeline.fav` 更新

- [x] `pipeline.fav` を読み込んで内容確認
- [x] 末尾に `return` ガード節パターン関数（`validate_amount`）を追加
- [x] 数値リテラル `_`（`1_000_000.0`）を使って v45.7.0 の機能も示す
- [x] `main` 関数から `validate_amount` を呼び出して W017 lint を回避

## T2 — `driver.rs`: v458000_tests 追加

- [x] `v458000_tests` モジュール追加（`v457000_tests` の直後）
- [x] `#[cfg(not(target_arch = "wasm32"))]` を付与（walkdir の WASM 互換）
- [x] `examples_no_legacy_effect_syntax` テスト実装
  - [x] `examples/` ディレクトリを `WalkDir` で再帰スキャン
  - [x] レガシーファイル（`custom_effects.fav` / `effect_errors.fav`）をスキップ
  - [x] インラインコメント除去後に `-> ... !UpperCase` パターン検出
  - [x] `ends_with` を `rel == s || rel.ends_with(&format!("/{}", s))` で安全に照合
- [x] `regex_like_match` ヘルパー関数実装

## T3 — テスト＆完了

- [x] `cargo test` 2986 passed, 0 failed
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `fav/Cargo.toml` version → `45.8.0`
- [x] `CHANGELOG.md` に v45.8.0 エントリ追加
- [x] `versions/current.md` を v45.8.0（2986 tests）に更新
- [x] tasks.md を COMPLETE に更新（T0〜T3 全チェック）

## コードレビュー指摘と対応

- [HIGH] code-reviewer: `pipeline.fav` の `Ok(amount)` / `Err("...")` は未定義コンストラクタ → `Result.ok(amount)` / `Result.err("...")` に修正
- [MED] code-reviewer: `skip_list` の `ends_with` 条件が死に体（`strip_prefix` 後の rel は `/` で始まらないため常に false） → `rel == *s` のみに簡略化
- [LOW] code-reviewer: `unwrap_or_default` でファイル読み取りエラーを無音スキップ → `unwrap_or_else(|e| panic!(...))` に変更
- [MED] code-reviewer: `custom_effects.fav` は `regex_like_match` の対象外（`public effect` 構文のみ）だが skip_list に残留 → 将来の `-> RetType !Effect` 追加時の安全網として保持（実害なし）
