# v76.3.0 タスクリスト — PII 来歴追跡・GDPR 消去計画

Date: 2026-08-15
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `76.2.0` であることを確認
- [x] `cargo test` が全 pass（3718 tests）であることを確認
- [x] `fav/tmp/hello.fav` が存在することを確認（bootstrap テスト要件）

---

## T1: driver.rs — 型・関数追加

- [x] `fav/src/driver.rs` の末尾に `// --- v76.3.0: PII 来歴追跡・GDPR 消去計画 ---` コメントを追加する
- [x] `PiiProvenanceReport` 構造体を追加する（fields: Vec<String>, source_uri: String, masked: bool）
- [x] `detect_pii_in_tag(tag: &ProvenanceTag) -> Vec<String>` を追加する
  - `pii=true` → `vec!["pii_detected"]`
  - `pii=false` → `vec![]`
- [x] `ErasurePlan` 構造体を追加する（target_uri: String, fields: Vec<String>, reason: String）
- [x] `generate_erasure_plan(tag: &ProvenanceTag) -> Option<ErasurePlan>` を追加する
  - `pii=true` → `Some(ErasurePlan { target_uri: source.uri, fields: detect_pii_in_tag(tag), reason: "GDPR erasure request" })`
  - `pii=false` → `None`
- [x] `cargo check` でコンパイルエラーがないことを確認する

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v76.3.0 エントリを追加する
- [x] Added セクション（struct 2 件・関数 2 件）と Tests セクション（2 件）を含める

---

## T3: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` に `v763000_tests` モジュールを追加する（`use super::*` 必須）
- [x] `pii_detected_in_provenance` テストを実装する
  - `detect_pii_in_tag(pii=true)` → 非空
  - `detect_pii_in_tag(pii=false)` → 空
- [x] `gdpr_erasure_plan_generated` テストを実装する
  - `generate_erasure_plan(pii=true)` → `Some` かつ `target_uri` にソース URI、`reason` に "GDPR" を含む
  - `generate_erasure_plan(pii=false)` → `None`
- [x] `cargo test v763000` で 2 件が pass することを確認する

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"76.2.0"` → `"76.3.0"` に変更する
- [x] `driver.rs` 内の `76.2.0` バージョン文字列アサーションを `76.3.0` に一括更新（replace_all）

---

## T5: versions/current.md 更新

- [x] 「進行中バージョン」を v76.3.0 に更新する
- [x] 「次に切る版」を v76.4.0 に更新する

---

## T6: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3720 tests）
- [x] `cargo test v763000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `76.3.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v76.3.0]` であることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T6）が完了している
- [x] `pii_detected_in_provenance` が pass
- [x] `gdpr_erasure_plan_generated` が pass
- [x] テスト総数: 3720（+2）
- [x] site/ MDX 追加: 本バージョンでは対象外（型基盤のみ）
- [x] `changelog_has_v76_3_0` テストの追加: 対象外（x.0.0 宣言バージョンのみに追加する慣例）。ただし CHANGELOG.md への v76.3.0 エントリ追加自体は T2 で必須
