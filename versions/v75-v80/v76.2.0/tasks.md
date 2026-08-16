# v76.2.0 タスクリスト — `TracedData` 型

Date: 2026-08-15
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `76.1.0` であることを確認
- [x] `cargo test` が全 pass（3716 tests）であることを確認
- [x] `fav/tmp/hello.fav` が存在することを確認（bootstrap テスト要件）

---

## T1: driver.rs — 型・関数追加

- [x] `fav/src/driver.rs` の末尾に `// --- v76.2.0: TracedData 型 ---` コメントを追加する
- [x] `TracedData` 構造体を追加する（data: String, provenance: ProvenanceTag）
- [x] `map_traced(t: TracedData, transform_label: &str) -> TracedData` を追加する
  - `transforms` に `transform_label` を push した新しい `TracedData` を返す
  - `data`・`pii` はそのまま保持する
- [x] `merge_provenance(a: &ProvenanceTag, b: &ProvenanceTag) -> ProvenanceTag` を追加する
  - `source`: `a.source` をベース（左辺優先）
  - `transforms`: `a.transforms` + `b.transforms` 連結
  - `pii`: `a.pii || b.pii`
- [x] `cargo check` でコンパイルエラーがないことを確認する

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v76.2.0 エントリを追加する
- [x] Added セクション（struct 1 件・関数 2 件）と Tests セクション（2 件）を含める

---

## T3: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` に `v762000_tests` モジュールを追加する（`use super::*` 必須）
- [x] `traced_map_appends_transform` テストを実装する
  - 1 回変換 → transforms に追記される
  - 2 回変換 → transforms に 2 件
  - data は変化しない
- [x] `traced_merge_propagates_pii` テストを実装する
  - pii=false + pii=true → merged.pii=true（OR 伝播）
  - pii=false + pii=false → merged.pii=false
  - `merged.source.name == "a"`（左辺ソース優先）
  - `merged.transforms.len() == 2`（a.transforms + b.transforms 連結）
- [x] `cargo test v762000` で 2 件が pass することを確認する

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"76.1.0"` → `"76.2.0"` に変更する
- [x] `driver.rs` 内の `76.1.0` バージョン文字列アサーションを `76.2.0` に一括更新（replace_all）

---

## T5: versions/current.md 更新

- [x] 「進行中バージョン」を v76.2.0 に更新する
- [x] 「次に切る版」を v76.3.0 に更新する

---

## T6: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3718 tests）
- [x] `cargo test v762000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `76.2.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v76.2.0]` であることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T6）が完了している
- [x] `traced_map_appends_transform` が pass
- [x] `traced_merge_propagates_pii` が pass
- [x] テスト総数: 3718（+2）
- [x] site/ MDX 追加: 本バージョンでは対象外（型基盤のみ）
- [x] CHANGELOG テスト: 宣言バージョン限定のため対象外
