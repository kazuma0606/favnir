# v41.0.0 実装計画

## 概要

Streaming Foundations スプリント完結版。マイルストーン宣言を行う。
Rust コードの機能追加なし。`MILESTONE.md` / `README.md` 更新と
driver.rs テスト更新、Cargo.toml バージョン bump（40.9.0 → 41.0.0）が主な作業。
★クリーンアップ（`cargo clean`）を実施する。

---

## 実装ステップ

### Step 1 — 事前確認
- `cargo test` が 2841 tests / 0 failures であることを確認
- `Cargo.toml` version が `40.9.0` であることを確認
- `v40900_tests::cargo_toml_version_is_40_9_0` が NOTE コメント付きライブアサーションであることを確認し行番号を記録
- `driver.rs` に `v41000_tests` モジュールが存在しないことを確認
- `MILESTONE.md` に `Streaming Foundations` が含まれないことを確認
- `README.md` に `Streaming Foundations` が含まれないことを確認

### Step 2 — MILESTONE.md 更新
v41.0.0 エントリを v40.0.0 エントリの直前（先頭）に追加。
`Streaming Foundations` という文字列を含めること。

### Step 3 — README.md 更新
README.md に `Streaming Foundations`（v41.0）の記述を追加。

### Step 4 — Cargo.toml バージョン bump
`fav/Cargo.toml` の `version = "40.9.0"` → `"41.0.0"` に変更。

### Step 5 — CHANGELOG.md 更新
`[v41.0.0]` エントリを `[v40.9.0]` の直後に追加。

### Step 6 — driver.rs テストモジュール更新
1. `v40900_tests::cargo_toml_version_is_40_9_0` をスタブ化
2. `v41000_tests` モジュール（4 テスト）を末尾に追加（`use super::*` 不要）

### Step 7 — cargo test 実行（クリーンアップ前）
`cargo test` で 2845 tests / 0 failures を確認。

### Step 8 — ★cargo clean + hello.fav 復元 + cargo test 再実行
1. `cargo clean` を実行
2. `fav/tmp/hello.fav` を復元（内容: `fn add(a: Int, b: Int) -> Int { a + b }` + `fn main() -> Bool { add(1, 2) == 3 }`）
3. `cargo test` を再実行し 2845 passed / 0 failed を確認

### Step 9 — バージョン管理ドキュメント更新
`versions/current.md`・ロードマップ完了マーク・`tasks.md` COMPLETE 更新。

---

## 依存関係

```
Step 1（確認）
  └→ Step 2（MILESTONE.md）
       └→ Step 6（driver.rs — milestone_has_streaming_foundations）
  └→ Step 3（README.md）
       └→ Step 6（driver.rs — readme_mentions_streaming_foundations）
  └→ Step 4（Cargo.toml）
       └→ Step 6（driver.rs — cargo_toml_version_is_41_0_0）
  └→ Step 5（CHANGELOG）
       └→ Step 6（driver.rs — changelog_has_v41_0_0）
            └→ Step 7（cargo test 初回）
                 └→ Step 8（cargo clean + hello.fav 復元 + cargo test 再実行）
                      └→ Step 9（docs 更新）
```

Step 2〜5 は相互に独立しており並列実施可能。

---

## リスクと注意点

- **テスト数差異**: ロードマップ記載は「≥ 2840」だが、v40.9.0 実績（2841）を起点に 2845 を採用する（spec.md §ロードマップとの差異 参照）。
- `cargo clean` 後に `fav/tmp/hello.fav` が削除される → `bootstrap_c2_artifact_roundtrip` テストが FAIL。必ず復元すること。
  - 正しい内容（2 行）: `fn add(a: Int, b: Int) -> Int { a + b }` と `fn main() -> Bool { add(1, 2) == 3 }`
- `include_str!` パスは `../` で `fav/`、`../../` で `favnir/` ルート。
  - `include_str!("../../MILESTONE.md")` → `favnir/MILESTONE.md`
  - `include_str!("../../README.md")` → `favnir/README.md`
- `v41000_tests` は `include_str!` のみ使用のため `use super::*` 不要。
- Cargo.toml のバージョンが `40.9.0` → `41.0.0`（メジャーバージョン変更）のため、`Cargo.lock` も更新される点に留意。
