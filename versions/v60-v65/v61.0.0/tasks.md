# v61.0.0 Tasks — Developer Experience 2.0 宣言 ★クリーンアップ

Date: 2026-07-31
Status: COMPLETE

---

## T0: 事前確認

- [x] `cargo test` でベースラインが 3349 tests passed, 0 failed であることを確認
  （注: ロードマップ記載は 3348 だが v60.8.0 で XSS テスト追加のため実際は 3349）
  - `grep '^version' fav/Cargo.toml` → `version = "60.0.0"`
- [x] `fav/Cargo.toml` のバージョンが `"60.0.0"` であることを確認
- [x] `v61000_tests` がまだ存在しないことを確認
  - `grep -c 'v61000_tests' fav/src/driver.rs` = 0 件
- [x] `v60900_tests` が存在すること（挿入先が実在すること）を確認
  - `grep -c 'v60900_tests' fav/src/driver.rs` ≥ 1 件

---

## T1: `fav/Cargo.toml` — バージョン更新

```toml
version = "61.0.0"
```

- [x] `version = "61.0.0"` に更新した
- [x] `cargo build` でコンパイルエラーがないことを確認

---

## T2: `MILESTONE.md` — Developer Experience 2.0 宣言エントリ追加

ファイル先頭（`# Favnir Milestones` 直後）に以下を追加。

```markdown
## v61.0.0（2026-07-31）— Developer Experience 2.0

> 「エラーはソース位置を指し、修正候補は即座に現れる。
>  エディタは意図を理解し、フォーマッタはコメントを守る。
>  REPL でパイプラインを対話的に探索でき、ドキュメントは自動生成される。
>
>  Favnir のエラーメッセージはデータエンジニアの道標になった。
>
>  これが Favnir v61.0 — Developer Experience 2.0 の姿である。」

**Developer Experience 2.0** の宣言バージョン。v60.1〜v60.9 で実装した全 DX 機能を統合し、
「エラーメッセージから修正まで一気通貫」の開発体験を確立した。

**v60.1〜v60.9 達成内容:**
- v60.1（エラー span 表示）...
- v60.9（安定化）...
```

- [x] `MILESTONE.md` に Developer Experience 2.0 宣言エントリを追加した
- [x] `"Developer Experience 2.0"` が含まれていることを確認

---

## T3: `CHANGELOG.md` — v61.0.0 エントリ追加

```markdown
## [v61.0.0] — 2026-07-31 — Developer Experience 2.0 宣言

### Added
- Developer Experience 2.0 正式宣言（v60.1〜v60.9 全 DX 機能統合完了）
- v60.1〜v60.9 全機能リスト
- `★クリーンアップ`（`cargo clean`）完了
```

- [x] `CHANGELOG.md` に v61.0.0 エントリを先頭に追加した
- [x] v60.1〜v60.9 全機能が列挙されていることを確認

---

## T4: `README.md` — v61.0.0 / Developer Experience 2.0 言及追加

v60.0.0 Enterprise 1.0 段落の直後に追加。

- [x] `README.md` に `"Developer Experience 2.0"` を含む段落を追加した

---

## T5: `driver.rs` — `v61000_tests` モジュール追加

`v60900_tests` の直前（上側）に挿入する。

```rust
// -- v61000_tests (v61.0.0) -- Developer Experience 2.0 宣言 --
#[cfg(test)]
mod v61000_tests {
    #[test]
    fn cargo_toml_version_is_61_0_0() { ... }

    #[test]
    fn changelog_has_v61_0_0() { ... }

    #[test]
    fn milestone_has_dx2() { ... }

    #[test]
    fn readme_mentions_dx2() { ... }
}
```

- [x] `v61000_tests` モジュールを `v60900_tests` の直前（上側）に追加した
- [x] `use super::*` なし（`include_str!` のみ使用）
- [x] 4 件全テストが含まれている

---

## T6: 旧 version assertion 更新

対象 9 モジュール:
`v56300_tests`, `v56900_tests`, `v57000_tests`, `v57900_tests`, `v58000_tests`,
`v58900_tests`, `v59000_tests`, `v59900_tests`, `v60000_tests`

各モジュールの `cargo_toml_version_is_*` テストのアサーション文字列（9 件）と
エラーメッセージ（9 件）を `"60.0.0"` → `"61.0.0"` に一括更新。

- [x] `version = \"60.0.0\"` の assertion 9 件を `"61.0.0"` に更新した
- [x] エラーメッセージ `"should be 60.0.0"` 9 件を `"should be 61.0.0"` に更新した

---

## T7: テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` を実行
- [x] `v61000_tests` 4 件 pass
- [x] 総テスト数 **3353** tests passed, 0 failed を確認

---

## T8: ★クリーンアップ

- [x] `cargo clean` を実行（10.2 GiB 削除）
- [x] `fav/tmp/hello.fav` が残存していることを確認

---

## T9: 事後処理

- [x] `versions/current.md` を v61.0.0 / 3353 tests に更新
- [x] `versions/roadmap/roadmap-v60.1-v61.0.md` の v61.0.0 実績欄を更新
- [x] `versions/roadmap/roadmap-v60.1-v61.0.md` テスト数推移表の v61.0.0 行（3352 → 3353）および v60.9.0 行（3348 → 3349）を修正
- [x] CHANGELOG.md: v61.0.0 エントリ（v60.1〜v60.9 全機能を含む）を追加
- [x] このファイル（tasks.md）を COMPLETE ステータスに更新

---

## コードレビュー指摘と対応

- **[MED] 旧 version assertion のエラーメッセージが `"60.0.0"` のまま（9 件）**:
  `"Cargo.toml version should be 61.0.0"` に一括更新（T6 で対応）。

---

Status: COMPLETE
