# v78.0.0 実装計画 — Verifiable Pipelines 宣言 ★クリーンアップ

Date: 2026-08-16

---

## 実装順序

### Step 1: 事前確認
- `fav/Cargo.toml` のバージョンが `77.9.0` であることを確認
- `fav/tmp/hello.fav` が存在することを確認（cargo clean 前に確認必須）
- `cargo test` が 3756 tests all pass であることを確認

### Step 2: cargo clean
- `cargo clean` を実施してビルドキャッシュを削除する
- `fav/tmp/hello.fav` が残っていることを確認（cargo clean で消えないが念のため）

### Step 3: MILESTONE.md 更新
先頭（`## v77.0.0` エントリの前）に v78.0.0 宣言エントリを追加。

### Step 4: README.md 更新
`## v77.0` セクションの前に `## v78.0 — Verifiable Pipelines 宣言（2026-08-16）` セクションを追加。

### Step 5: CHANGELOG.md 更新（テスト追加より先）
先頭に v78.0.0 エントリを追加。

### Step 6: driver.rs — テストモジュール追加
`fav/src/driver.rs` の末尾（`// --- v77.9.0` ブロックの後）に `v78000_tests` モジュールを追加（`use super::*` 不要 — `include_str!` マクロのみ使用）：

1. `cargo_toml_version_is_78_0_0` — `include_str!("../Cargo.toml")` で確認（Cargo.toml は fav/ 直下）
2. `changelog_has_v78_0_0` — `include_str!("../../CHANGELOG.md")`
3. `milestone_has_verifiable_pipelines` — `include_str!("../../MILESTONE.md")`
4. `readme_mentions_verifiable_pipelines` — `include_str!("../../README.md")`

> **注意**: この時点では `cargo test v78000` を実行しない。`cargo_toml_version_is_78_0_0` が Cargo.toml 更新（Step 7）前のため必ず失敗する。

### Step 7: Cargo.toml バージョン更新
- `77.9.0` → `78.0.0` に変更
- driver.rs 内の `77.9.0` バージョン文字列アサーションを一括更新（`replace_all: true`）
- grep で `// --- v77.9.0: 安定化・コードフリーズ ---` が維持されていることを確認（上書きされていたら戻す）

### Step 8: versions/current.md 更新
- `## 進行中バージョン` 欄を `**v78.0.0**（Verifiable Pipelines 宣言 ★クリーンアップ）` に更新
- `## 最新安定版` 欄を v77.0.0 の位置に v78.0.0 を追加（または更新）
- `## 次に切る版` 欄を `**v78.1.0**（次スプリント開始）` に更新

### Step 9: 最終確認
- `cargo test` が 3760 tests all pass であることを確認
- `cargo test v78000` で 4 件が pass することを確認
