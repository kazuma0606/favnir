# Tasks: v48.3.0 — `fav.toml [runes]` 解決ロジック

Status: COMPLETE
Date: 2026-07-18

---

## T0 — 事前確認

- [x] `cargo test` 3049 passed, 0 failed を確認（ベース確認）
- [x] `toml.rs` の `FavToml` struct に `runes` フィールドが存在しないことを確認
- [x] `toml.rs` の `"runes"` アームが `path` キーのみ処理していることを確認（`kafka = "2.1.0"` が無視される）
- [x] `error_catalog.rs` の E0417 が予約コメントのみで `ErrorEntry` がないことを確認

## T1 — `toml.rs` 変更

- [x] `FavToml` struct に `pub runes: std::collections::HashMap<String, String>` フィールド追加（`stream` フィールドの直後）
- [x] `parse_fav_toml` に `let mut runes_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();` 追加（`stream_cfg` 直後）
- [x] `"runes"` アームを更新
  - [x] `key == "path"` → `runes_path`（既存、変更なし）
  - [x] それ以外 → `runes_map.insert(key.to_string(), val.to_string())`（新規追加）
- [x] `FavToml { ... }` 構造体初期化に `runes: runes_map` を追加

## T2 — コンパイルエラー対応（6 箇所・必ず発生）

- [x] `cargo build` を実行してコンパイルエラーを確認（実際は 6 箇所中 1 箇所は cargo build で検出、残 5 箇所は cargo test 時に検出）
- [x] `FavToml { ... }` を直接構築している以下 6 箇所に `runes: std::collections::HashMap::new()` を追記
  - [x] `fav/src/driver.rs` 行 4623 付近（`cargo build` で検出）
  - [x] `fav/src/middle/resolver.rs` 行 348 付近
  - [x] `fav/src/middle/resolver.rs` 行 444 付近
  - [x] `fav/src/middle/resolver.rs` 行 556 付近
  - [x] `fav/src/middle/checker.rs` 行 8518 付近
  - [x] `fav/src/middle/checker.rs` 行 8614 付近
- [x] `cargo build` で全エラーが解消されたことを確認

## T3 — `error_catalog.rs` + `driver.rs` + バージョン更新・完了

- [x] `error_catalog.rs`: E0416 の直後、E0420 の前の予約コメントを E0417 `ErrorEntry` に差し替え（予約コメントを E0418〜E0419 に更新）
- [x] `driver.rs`: `v483000_tests` モジュールを `v482000_tests` の直前に追加（2テスト）
  - [x] `rune_resolution_from_toml`: `[runes]\nkafka = "2.1.0"` が `toml.runes.get("kafka") == Some("2.1.0")` でアクセスできる（`path` 混入なしも確認）
  - [x] `e0417_rune_not_in_toml`: `ERROR_CATALOG` に `code == "E0417"` のエントリが存在する
- [x] `fav/Cargo.toml` version → `"48.3.0"`
- [x] `CHANGELOG.md` に v48.3.0 エントリ追加
- [x] `cargo test` 3051 passed, 0 failed（3049 + 2 件）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `versions/current.md` を v48.3.0（3051 tests）に更新、進行中バージョンを `v48.4.0` に更新（サブスプリントリンクは変更不要）
- [x] `versions/roadmap/roadmap-v48.1-v49.0.md` の v48.3.0 完了条件テスト数（推定 3051）を実績値に更新
- [x] tasks.md を COMPLETE に更新（T0〜T3 全 `[x]`）

---

> **注記**: E0417 の実際の発行ロジック（`checker.rs` での `ImportKind::Package` × `FavToml.runes` 突き合わせ）は v48.5.0 以降のスコープ
> **注記**: マスターロードマップ（`roadmap-v45.1-v50.0.md`）への反映は v49.0.0 マイルストーン宣言時に実施

---

## 実装時の追加発見

| 内容 | 対応 |
|---|---|
| `cargo build` で検出されるのは `driver.rs` 1 箇所のみ（`resolver.rs` / `checker.rs` の 5 箇所は `#[cfg(test)]` 内のため `cargo test` 時に検出される） | `cargo test` でコンパイルエラーを確認し 5 箇所を追加修正 |

## コードレビュー指摘と対応（code-reviewer）

| 重大度 | 内容 | 対応 |
|---|---|---|
| [MED] | E0417 emit ロジックが未実装（`ERROR_CATALOG` 登録のみ） | v48.5.0 スコープとして注記に追記（既にスコープ外として明記済み） |
| [MED] | `parse_kv` が `=` を含む値を切り捨てる可能性（既存バグ） | v48.3.0 で導入した問題ではないため記録のみ・runes のバージョン文字列には無影響 |
| [LOW] | `rune_resolution_from_toml` テストに `path = "..."` 入力が含まれておらず `contains_key("path")` の検証が自明条件だった | `path = "libs/runes"` を入力に追加し `runes_map` 混入なし・`runes_path` への設定を明示検証 |
| [LOW] | `std::collections::HashMap` 完全修飾パスの繰り返し | 既存スタイルに統一されているため許容 |

## コードレビュー指摘と対応（spec-reviewer）

| 重大度 | 内容 | 対応 |
|---|---|---|
| [HIGH] | ロードマップとスコープ分割の根拠不足 | `roadmap-v48.1-v49.0.md` の v48.3.0 セクションにスコープ分割（v48.3.0 vs v48.5.0）を明記 |
| [HIGH] | ロードマップテスト推定値 3046 が逆転不整合 | `roadmap-v48.1-v49.0.md` を 3046 → 3051 に修正 |
| [HIGH] | `FavToml { ... }` 直接構築 6 箇所が spec/plan/tasks に未列挙 | spec.md 注意事項・plan.md Step 5・tasks.md T2 に 6 箇所を明示 |
| [MED] | `path` キー混入確認テスト不足 | spec.md + driver.rs テストに `contains_key("path") == false` アサーション追加 |
| [MED] | `[rune]` / `[runes]` 区別の未記載 | spec.md + plan.md 注意事項に明記 |
| [MED] | tasks.md T3 の「実績で確認」が読み取りのみに見える | 「実績値に更新」に修正 |
| [LOW] | current.md サブスプリントリンク変更不要が未明示 | tasks.md に「（サブスプリントリンクは変更不要）」を追記 |
