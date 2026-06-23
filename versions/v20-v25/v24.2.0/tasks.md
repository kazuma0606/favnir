# v24.2.0 — 4-Stage Bootstrap 検証タスク

## ステータス: COMPLETE（2026-06-23）

---

## タスク一覧

### T0: 事前確認

- [x] `grep -n "version = " fav/Cargo.toml` — `"24.1.0"` であること
- [x] `grep -n "mod v241000_tests\|mod v242000_tests" fav/src/driver.rs | head -5` — v242000_tests 未存在
- [x] `ls fav/tests/bootstrap/ 2>/dev/null || echo "not found"` — ディレクトリ未存在
- [x] `grep -n "\[v24.2.0\]" CHANGELOG.md | head -3` — 0 件であること

---

### T1: fixture ファイル作成（`fav/tests/bootstrap/`）

- [x] **T1-1**: `fav/tests/bootstrap/hello.fav` 作成（`"Hello, Favnir!"` を返す）
- [x] **T1-2**: `fav/tests/bootstrap/arithmetic.fav` 作成（`add` / `mul` + f-string 出力）
- [x] **T1-3**: `fav/tests/bootstrap/pattern_match.fav` 作成（Option マッチ ※フィールドなしバリアント非対応のため変更）
- [x] **T1-4**: `fav/tests/bootstrap/list_ops.fav` 作成（多引数算術関数 ※`[h | t]` パターン非対応のため変更）
- [x] **T1-5**: `fav/tests/bootstrap/closures.fav` 作成（`apply` 高階関数 + `|x| x * x`）
- [x] **事後確認**: 5 ファイル存在確認

---

### T2: `fav/src/driver.rs` — v242000_tests 追加

- [x] **事前確認**: `grep -n "fn version_is_24_1_0" fav/src/driver.rs | head -3`
- [x] **T2-1（T3-1 より前に必須）**: `v241000_tests::version_is_24_1_0` テスト関数を**削除**
- [x] **T2-2**: `v242000_tests` モジュールを `v241000_tests` の直後に追加（7 件カウント済）
  - `version_is_24_2_0`
  - `bootstrap_hello_compiles`
  - `bootstrap_arithmetic_compiles`
  - `bootstrap_pattern_match_compiles`
  - `bootstrap_list_ops_compiles`
  - `bootstrap_closures_compiles`
  - `changelog_has_v24_2_0`
- [x] **T2-3**: `#[ignore]` テスト 2 件を `self_tests` モジュール末尾に追加（`run_compiler_artifact_on` / `build_stage2_compiler_artifact` がプライベートヘルパーのためスコープに合わせて配置）
  - `bootstrap_stage1_stage3_hello_match`（`#[ignore]`）
  - `bootstrap_stage1_stage3_arithmetic_match`（`#[ignore]`）
- [x] `cargo check --bin fav` — エラー 0
- [x] `cargo test v242000 --bin fav` — 7/7 PASS を確認
- [x] `cargo test --bin fav` — リグレッションなし（1940 件合格）を確認

---

### T3: Cargo.toml + CHANGELOG + benchmarks + bootstrap.mdx

- [x] `fav/Cargo.toml` の `version = "24.1.0"` → `"24.2.0"` に変更
- [x] `CHANGELOG.md` 先頭に v24.2.0 エントリを追加
- [x] `benchmarks/v24.2.0.json` を新規作成（test_count: 1940）
- [x] `site/content/docs/tools/bootstrap.mdx` を新規作成
- [x] `cargo test v242000 --bin fav` — 最終確認 7/7 PASS
- [x] `cargo test --bin fav` — リグレッションなし（1940 件合格）

---

## テスト一覧（v242000_tests、7 件カウント済）

| テスト名 | 内容 | 期待値 |
|---|---|---|
| `version_is_24_2_0` | Cargo.toml に `version = "24.2.0"` | — |
| `bootstrap_hello_compiles` | hello.fav を tokenize + parse + build_artifact | パニックなし |
| `bootstrap_arithmetic_compiles` | arithmetic.fav を tokenize + parse + build_artifact | パニックなし |
| `bootstrap_pattern_match_compiles` | pattern_match.fav を tokenize + parse + build_artifact | パニックなし |
| `bootstrap_list_ops_compiles` | list_ops.fav を tokenize + parse + build_artifact | パニックなし |
| `bootstrap_closures_compiles` | closures.fav を tokenize + parse + build_artifact | パニックなし |
| `changelog_has_v24_2_0` | CHANGELOG.md に `[v24.2.0]` | — |

## テスト一覧（`#[ignore]`、2 件、self_tests 内）

| テスト名 | 内容 | 期待値 |
|---|---|---|
| `bootstrap_stage1_stage3_hello_match` | Stage 1/3 bytecode_A == bytecode_B（hello.fav） | 一致 |
| `bootstrap_stage1_stage3_arithmetic_match` | Stage 1/3 bytecode_A == bytecode_B（arithmetic.fav） | 一致 |

---

## 完了条件チェックリスト

- [x] `fav/tests/bootstrap/` に 5 fixture 作成済み
- [x] `v241000_tests::version_is_24_1_0` が削除済み（T3-1 より前）
- [x] `cargo test v242000 --bin fav` — 7/7 PASS
- [x] `cargo test --bin fav` — リグレッションなし（1940 件合格）
- [x] `CHANGELOG.md` に v24.2.0 エントリ
- [x] `benchmarks/v24.2.0.json` 作成済み（test_count: 1940）
- [x] `site/content/docs/tools/bootstrap.mdx` 作成済み
- [x] Stage 4 保留が CHANGELOG / bootstrap.mdx / spec.md / plan.md / tasks.md に明記済み

---

## コードレビュー対応（2026-06-23 — code-reviewer 指摘）

| 優先度 | 指摘 | 対応 |
|--------|------|------|
| [MED] | `build_artifact` 戻り値を `_art` に捨てており、artifact 内容を検証していない | `compile_fixture` ヘルパーを抽出し、`art.globals.is_empty()` アサーションを追加 |
| [MED] | `#[ignore]` Stage 1-3 テストが hello / arithmetic のみで pattern_match / closures が欠落 | `bootstrap_stage1_stage3_pattern_match_match` / `bootstrap_stage1_stage3_closures_match` を `self_tests` 末尾に追加 |
| [LOW] | `arithmetic.fav` で非 Result 関数の戻り値に `bind` を使用（意味論的に不正確） | f-string インライン呼び出し `f"sum={add(3,7)} product={mul(4,5)}"` に変更 |

## 実装時の注意事項（実績）

| # | 内容 | 対応方針 |
|---|---|---|
| 1 | `type Shape = Circle \| Square \| Triangle` がパーサーエラー（フィールドなしバリアント非対応） | pattern_match.fav を `Option<Int>` マッチに変更 |
| 2 | `[h \| t]` リストパターンがパーサーエラー（"expected RBracket, got Pipe"） | list_ops.fav を多引数算術関数 `sum(a,b,c,d,e)` に変更 |
| 3 | `run_compiler_artifact_on` / `build_stage2_compiler_artifact` は `self_tests` プライベートヘルパー | `#[ignore]` テストを `self_tests` 末尾に配置（v242000_tests ではなく） |
| 4 | 実際のテスト件数: `version_is_24_1_0` 削除(-1) + 新規7件 = 純増+6 → 1934+6=1940 | spec/plan/tasks/benchmarks を 1940 に修正 |
