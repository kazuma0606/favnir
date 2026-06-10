# v14.0.0 Tasks — 能力型完成宣言

Date: 2026-06-11
Branch: feat/v13-capability-context

---

## Phase A — CI self-check テスト追加

- [ ] A-1: `fav/src/driver.rs` に `v140000_tests` モジュールを追加
- [ ] A-2: 以下のテストを実装:
  - [ ] `version_is_14_0_0` — `CARGO_PKG_VERSION == "14.0.0"`
  - [ ] `e0025_self_compiler_zero` — `self/compiler.fav` に E0025 が 0 件
  - [ ] `e0025_self_checker_zero` — `self/checker.fav` に E0025 が 0 件
  - [ ] `e0023_and_e0025_both_zero_compiler` — compiler.fav は E0023 + E0025 ともに 0 件
  - [ ] `capability_context_design_complete` — 全 capability lint 関数の存在確認
- [ ] A-3: `cargo test v140000` で全件パス確認（5/5）

---

## Phase B — CHANGELOG 更新

- [ ] B-1: `CHANGELOG.md` に v14.0.0 セクションを追加（先頭）
  - Breaking Changes: E0025（bang notation）、E0023（ambient effect）
  - New Features: v13.1.0〜v13.10.0 の機能を集約
  - Error Codes: W008/E0020〜E0025/W009/W010
  - Migration: `fav migrate --from-effects` の使い方

---

## Phase C — README 更新

- [ ] C-1: `README.md` の Effects セクションを Capability Context セクションに更新
  - 旧 `!Postgres` → 新 `ctx: LoadCtx` のコード例置き換え
  - `fav migrate --from-effects` の使い方追記
  - capability-context 設計の概要（1 段落）追加

---

## Phase D — バージョンバンプ + コミット

- [ ] D-1: `fav/Cargo.toml` → `version = "14.0.0"`
- [ ] D-2: `cargo test` 全件パス確認
- [ ] D-3: `git commit -m "feat: v14.0.0 — 能力型完成宣言"`

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `self/compiler.fav` に E0025 が 0 件 | |
| `self/checker.fav` に E0025 が 0 件 | |
| `cargo test v140000` 全件パス（5/5） | |
| `cargo test` 全件パス | |
| `CARGO_PKG_VERSION == "14.0.0"` | |
| CHANGELOG に v14.0.0 セクションあり | |
| README に capability-context 説明あり | |

---

## E2E デモ実行（v14.0.0 リリース後）

以下は v14.0.0 本体には含めず、リリース後に別途実施:

- [ ] `infra/e2e-demo/fav2py/` 実行 PASS=5 確認（ECS Fargate / RDS PostgreSQL）
- [ ] `infra/e2e-demo/airgap/` 実行 PASS=5 確認（Airgap VPC）
- 確認後にこのチェックボックスを更新する

---

## 実装ノート

- **`self/compiler.fav` の E0025 確認**: v13.8.0 で E0023 移行済みのため、`effects` フィールドは空であるはず。テストで確認。
- **CHANGELOG の形式**: 既存の CHANGELOG.md フォーマットに合わせる（`## [X.Y.Z] — DATE — タイトル` 形式）。
- **README の更新範囲**: Effects/エフェクト型に言及している箇所のみ。他のセクションは変更しない。
- **v13.x テストの `version_is_*`**: v13.6〜13.9 の `version_is_*` テストは v13.10.0 で `!version.is_empty()` に修正済み。v14.0.0 では同様に修正不要。
