# v24.6.0 — セキュリティ審査（エフェクトシステム形式検証）タスク

## ステータス: COMPLETE

---

## タスク一覧

### T0: 事前確認 + `fav/src/lint.rs` — W021 追加

- [x] `grep -n "version = " fav/Cargo.toml` — `"24.5.0"` であること
- [x] `grep -n "mod v246000_tests" fav/src/driver.rs | head -3` — 未存在
- [x] `grep -n "W021\|check_w021" fav/src/lint.rs | head -5` — 全 0 件
- [x] `ls SECURITY.md SECURITY_MODEL.md 2>/dev/null` — ファイルが存在しないこと
- [x] **T0-1**: `check_w021_block` / `check_w021_expr` を `lint.rs` の `check_w020_*` 直後に追加
- [x] **T0-2**: `pub fn check_w021_pure_fn_calls_effectful(program: &Program, errors: &mut Vec<LintError>)` を追加
  - `fd.effects.iter().any(|e| e != &Effect::Pure)` でエフェクト有り判定
  - `fd.effects.is_empty() || fd.effects.iter().all(|e| e == &Effect::Pure)` で純粋判定
  - `use crate::ast::*;` がすでに `lint.rs` 先頭にあるため `Effect` は追加 import 不要
- [x] **T0-3**: `lint_program()` の `check_w020_deprecated_call` 直後に追加:
  ```rust
  check_w021_pure_fn_calls_effectful(program, &mut errors);
  ```
- [x] **事後確認**: `cargo check --bin fav` — エラー 0

---

### T1: `SECURITY.md` 作成

- [x] `C:\Users\yoshi\favnir\SECURITY.md` を新規作成
  - `"security@favnir.dev"` を含む（テスト要件）
  - `"90"` を含む（90日ルール、テスト要件）
  - CVE 番号付与プロセス・パッチリリースポリシーを記載

---

### T2: `SECURITY_MODEL.md` 作成

- [x] `C:\Users\yoshi\favnir\SECURITY_MODEL.md` を新規作成
  - `"Capability"` を含む（テスト要件）
  - `"Purity"` を含む（テスト要件）
  - 公理 4 条（Purity / Effect Propagation / Capability Confinement / Composition）
  - 推論規則（T-Pure / T-Effect / T-Compose）
  - W021 との対応説明
  - 外部審査依頼事項

---

### T3: `fav/src/driver.rs` — v246000_tests 追加

- [x] **事前確認**: `grep -n "fn version_is_24_5_0" fav/src/driver.rs | head -3`
- [x] **T3-1（T5-1 より前に必須）**: `v245000_tests::version_is_24_5_0` テスト関数を**削除**（モジュール自体と他4件のテストは保持すること）
- [x] **T3-2**: `v246000_tests` モジュールを `v245000_tests` の直後に追加（5 件）
  - `version_is_24_6_0`
  - `security_md_has_disclosure_policy`
  - `security_model_md_exists`
  - `w021_pure_fn_calls_effectful_lint`
  - `changelog_has_v24_6_0`
- [x] `cargo test v246000 --bin fav` — 5/5 PASS を確認
- [x] `cargo test --bin fav` — リグレッションなし（1957 件合格）を確認
  > 件数計算: 1953 (現在) - 1 (version_is_24_5_0 削除) + 5 (v246000_tests) = 1957

---

### T4: ドキュメントサイト更新

- [x] `site/content/docs/tools/security.mdx` を新規作成（セキュリティモデル・CVE プロセス解説）

---

### T5: Cargo.toml + CHANGELOG + benchmarks

- [x] `fav/Cargo.toml` の `version = "24.5.0"` → `"24.6.0"` に変更（T3-1 完了後）
- [x] `CHANGELOG.md` 先頭に v24.6.0 エントリを追加
- [x] `benchmarks/v24.6.0.json` を新規作成（test_count: 1957、duration_ms: 17000）
- [x] `cargo test v246000 --bin fav` — 最終確認 5/5 PASS
- [x] `cargo test --bin fav` — リグレッションなし（1957 件合格）

---

## テスト一覧（v246000_tests、5 件）

| テスト名 | 内容 | 期待値 |
|---|---|---|
| `version_is_24_6_0` | Cargo.toml に `version = "24.6.0"` | — |
| `security_md_has_disclosure_policy` | `SECURITY.md` に `"security@favnir.dev"` と `"90"` が含まれる | — |
| `security_model_md_exists` | `SECURITY_MODEL.md` に `"Capability"` と `"Purity"` が含まれる | — |
| `w021_pure_fn_calls_effectful_lint` | 純粋関数 `process` から `!Http` 関数 `fetch_data` を呼ぶと W021 が 1 件 | `warnings.len() == 1` |
| `changelog_has_v24_6_0` | `CHANGELOG.md` に `[v24.6.0]` | — |

---

## 完了条件チェックリスト

- [x] `check_w021_pure_fn_calls_effectful()` 実装済み（`lint_program()` に組み込み）
- [x] `SECURITY.md` 作成済み（`security@favnir.dev` / `90` 日 / CVE プロセス）
- [x] `SECURITY_MODEL.md` 作成済み（`Capability` / `Purity` / 推論規則）
- [x] `v245000_tests::version_is_24_5_0` が削除済み（T5-1 より前）
- [x] `cargo test v246000 --bin fav` — 5/5 PASS
- [x] `cargo test --bin fav` — リグレッションなし（1957 件合格）
- [x] `CHANGELOG.md` に v24.6.0 エントリ
- [x] `benchmarks/v24.6.0.json` 作成済み（test_count: 1957）
- [x] `site/content/docs/tools/security.mdx` 作成済み

---

## コードレビュー対応（実施済み）

spec-reviewer 指摘:
- [HIGH] `use crate::ast::Effect;` は `lint.rs` に `use crate::ast::*;` が既にあるため不要 → 実装で除外済み
- [LOW] `Forall` は `iter` フィールドなし（`guard` + `body` のみ）→ コメントで明示
- [LOW] `spec.md` 完了条件の `1953 → 1957` 整合確認 → 修正済み
