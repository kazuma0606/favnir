# Roadmap v49.1.0 〜 v50.0.0 — Production 2.0

Date: 2026-07-15
Status: 計画中（v49.0 完了後に開始）

---

## 前提

- 直前完了: v49.0.0「Module & Package 2.0」（v49.0 宣言後、tests ≥ 3062）
- マスターロードマップ: `roadmap-v45.1-v50.0.md`
- 本文書はマスターの v50.0 スプリント部分の詳細版

---

## 目標

v46〜v49 の全機能を統合・安定化し、ドキュメントと品質を整備して
**「迷わず使える実用言語」として Favnir v50.0 を宣言する**。

---

## バージョン計画

### v49.1.0 — 全機能統合テスト + E2E デモ更新

v46〜v49 の全機能を使った E2E デモを更新。
`return` ガード節 / 新 import 構文 / stdlib 2.0 / `fav test` ブロックを
統合したパイプラインデモ（`examples/v50-demo/`）を作成。

```favnir
import "./stages/validate" as validate
import kafka

pipeline OrderIngestion {
  stage Consume: Stream<RawOrder> -> Stream<Order> = |raw| {
    bind order <- kafka.consume("orders")
    bind valid <- validate.run(order)
    Ok(valid)
  }
}

#[test]
fn test_validate() {
  assert_ok(validate.run(good_order))
  assert_err(validate.run(bad_order))
}
```

**完了条件**: Rust テスト 2 件（実績推定 3064 tests passed, 0 failed）
- `e2e_demo_v50_structure`
- `e2e_demo_uses_new_import`

**実績**: 3071 tests passed, 0 failed（2026-07-18 完了）✅

---

### v49.2.0 — パフォーマンス計測 + ボトルネック修正

`fav bench --all` で v46〜v49 機能追加後の速度計測。
checker.rs / compiler.rs のホットパスを特定し改善。
計測結果を `benchmarks/v49.2.0.json` に JSON で保存（フラット命名慣例）。

**完了条件**: Rust テスト 2 件（実績推定 3073 tests passed, 0 failed）
- `bench_all_result_recorded`
- `checker_perf_regression_none`

**実績**: 3073 tests passed, 0 failed（2026-07-18 完了）✅

---

### v49.3.0 — `fav check` インクリメンタル型チェック

変更ファイルのみ再チェック（SHA-256 フィンガープリント）。
大規模プロジェクトでの `fav check` 速度改善。
`.fav-cache/` ディレクトリにフィンガープリントを保存。

**完了条件**: Rust テスト 2 件（実績推定 3068 tests passed, 0 failed）
- `incremental_check_skips_unchanged`
- `incremental_check_detects_change`

**実績**: 3075 tests passed, 0 failed（2026-07-18 完了）✅

---

### v49.4.0 — ドキュメントサイト全面更新 Phase 1

v46〜v48 機能の docs MDX 更新。
- `return` 構文: `site/content/docs/syntax/return.mdx`
- stdlib 2.0: `site/content/docs/stdlib/` 各ページ
- import 2.0: `site/content/docs/modules/import.mdx`

**完了条件**: Rust テスト 2 件（実績推定 3070 tests passed, 0 failed）
- `docs_return_syntax_exists`
- `docs_import_v2_exists`

**実績**: 3077 tests passed, 0 failed（2026-07-18 完了）✅

---

### v49.5.0 — cookbook 更新

新機能を活用したレシピを追加。
- `return` ガード節パターン: `site/content/cookbook/return-guard-pattern.mdx`
- `fav test` 活用レシピ: `site/content/cookbook/inline-testing.mdx`
- 新 import 構文: `site/content/cookbook/modular-pipelines.mdx`

**完了条件**: Rust テスト 2 件（実績推定 3072 tests passed, 0 failed）
- `cookbook_return_guard_exists`
- `cookbook_fav_test_exists`

**実績**: 3079 tests passed, 0 failed（2026-07-18 完了）✅

---

### v49.6.0 — WASM / Python transpiler 互換確認

v46〜v49 の新構文が WASM ビルドと Python transpiler で正しく動作することを確認。
`return` / 新リテラル / 新 import が各ターゲットで処理されること。
`emit_python.rs` に `Stmt::Return` の完全実装確認（stub → 実装）。

**完了条件**: Rust テスト 2 件（実績推定 3074 tests passed, 0 failed）
- `wasm_compat_return_stmt`
- `python_emit_return_stmt`

**実績**: 3081 tests passed, 0 failed（2026-07-18 完了）✅

---

### v49.7.0 — セキュリティ審査 2.0

import 2.0 でのパストラバーサル検証（`"../../etc/passwd"` 等の拒否）。
`fav install` でのパッケージ名バリデーション（英数字 + `-` のみ許可）。
`driver.rs` の `validate_import_path` / `validate_rune_name` ヘルパーを追加。

**完了条件**: Rust テスト 2 件（実績推定 3083 tests passed, 0 failed）
- `import_path_traversal_rejected`
- `install_invalid_name_rejected`

**実績**: 3083 tests passed, 0 failed（2026-07-18 完了）✅

---

### v49.8.0 — ドキュメントサイト全面更新 Phase 2 + CHANGELOG 整理

v49〜全体の CHANGELOG / MILESTONE 整理。
`site/content/docs/` 全ページの最終チェック。
`site/content/docs/language-maturity-overview.mdx` の骨子作成。

**完了条件**: Rust テスト 2 件（実績推定 3085 tests passed, 0 failed）
- `docs_site_v50_overview_exists`
- `milestone_has_language_maturity`

**実績**: 3085 tests passed, 0 failed（2026-07-18 完了）✅

---

### v49.9.0 — v50.0 前調整・安定化

コードフリーズ。`site/content/docs/language-maturity-overview.mdx` を完成させる。
全 lint / clippy クリーン確認。
`cargo test` 3087 tests passed, 0 failed を確認して v50.0 へ。

**完了条件**: Rust テスト 2 件（実績推定 3087 tests passed, 0 failed）
- `cargo_toml_version_is_49_9_0`
- `language_maturity_overview_doc_exists`

**実績**: 3087 tests passed, 0 failed（2026-07-18 完了）✅

---

### v50.0.0 — Production 2.0 宣言 ★クリーンアップ

**宣言文**:

> 「`return` による安全なガード節、成熟した標準ライブラリ、
>  明確なモジュールシステム、インラインテストが揃い、
>  Favnir は迷わず使える実用言語になった。
>
>  これが Favnir v50.0 — Production 2.0 の姿である。」

**完了条件**:
- v49.1〜v49.9 の全機能が動作する
- `cargo test` 全通過（failures=0 かつテスト数 ≥ **3091**）
- `v50000_tests` 4 件 pass:
  - `cargo_toml_version_is_50_0_0`
  - `changelog_has_v50_0_0`
  - `milestone_has_language_maturity` — MILESTONE.md に `"Language Maturity"` が含まれる
  - `readme_mentions_language_maturity`
- `MILESTONE.md` に `"Language Maturity"` が含まれる
- `★クリーンアップ`（`cargo clean`）完了

**実績**: 3091 tests passed, 0 failed（2026-07-18 完了）✅ — Production 2.0 宣言

---

## 参考リンク

- マスターロードマップ: `versions/roadmap/roadmap-v45.1-v50.0.md`
- 前サブスプリント（アクティブ）: `versions/roadmap/roadmap-v48.1-v49.0.md`
- 次フェーズ（v50.0 完了後）: 次マスターロードマップ（v50.1〜v55.0）
- 達成宣言: `MILESTONE.md`
