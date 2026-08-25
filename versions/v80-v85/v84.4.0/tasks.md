# v84.4.0 タスクリスト

Status: COMPLETE

---

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3,915 tests, 0 failures を確認する（前提: v84.3.0 完了済み）
- [x] `grep -m1 '^version' fav/Cargo.toml` の出力が `version = "84.0.0"` であることを確認する
  （v84.x マイナーバージョンは Cargo.toml を更新しない慣例。v85.0.0 宣言時に一括更新する）
- [x] `fav/src/driver.rs` に `mod v84300_tests` が存在することを確認する（v84.3.0 完了済みの証拠）

## T1: `contract.fav` を完成版に更新

- [x] `infra/e2e-demo/favnir4-showcase/contract.fav` の末尾に以下を追加する
  - `SlaContract` 型宣言（`name: String`, `target: SlaTarget`, `adaptive_strategy: Option<String>`, `cache_ttl_secs: Option<Int>`）
  - `ContractDependency` 型宣言（`upstream: String`, `downstream: String`, `output_contract: String`）
  - 型名は `Showcase` プレフィックスなし（Rust テストが `"SlaContract"` / `"ContractDependency"` で完全一致検索するため）
- [x] `bind` 構文は不要（型宣言のみ）
- [x] コメント行で Sprint 3 への言及を追加する

## T2: `pipeline.fav` に契約統合セクションを追加

- [x] 現在の `pipeline.fav` 末尾に `-- ── 契約統合セクション（Sprint 3: Pipeline Contracts 1.0）──────────────` コメントを先頭に追加する
- [x] `showcase_contract_registry` 関数を追加する
  - `ContractRegistry { entries: List.empty() }` で空レジストリを初期化
  - `IoContract { name, version, input: List.empty(), output: List.empty() }` を構築
  - `ContractRegistryEntry { name, version: ContractVersion { major, minor, patch }, contract }` を構築
  - `registry.register(entry)` で登録し `Result.ok(...)` で返す
- [x] `bind` 構文を使用する（`let` は使わない）
- [x] フィールド名が spec.md の型定義テーブルと一致することを確認する

## T3: `fav/src/driver.rs` に `v84400_tests` を追加

- [x] `mod v84300_tests { ... }` の直後に `#[cfg(test)] mod v84400_tests { ... }` を追加する
  - `include_str!` は `"../../infra/..."` 形式（パス起点: `fav/src/driver.rs`）
- [x] `showcase_contract_verified` テストを実装する
  - `include_str!("../../infra/e2e-demo/favnir4-showcase/contract.fav")` に `"SlaContract"` が含まれること（メッセージ付き）
  - 同ファイルに `"ContractDependency"` が含まれること（メッセージ付き）
- [x] `showcase_contract_registry_registered` テストを実装する
  - `include_str!("../../infra/e2e-demo/favnir4-showcase/pipeline.fav")` に `"ContractRegistry"` が含まれること（メッセージ付き）
  - 同ファイルに `"IoContract"` が含まれること（メッセージ付き）

## T4: テスト通過確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3,917 tests, 0 failures（+2）であることを確認する

## T5: CHANGELOG 更新

- [x] `CHANGELOG.md` の先頭に v84.4.0 エントリを追加する

> 注: 本バージョンは `contract.fav` / `pipeline.fav` 更新とテスト追加のみ。`site/` MDX 追加は v84.6.0 で実施する。

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
