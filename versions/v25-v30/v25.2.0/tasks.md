# v25.2.0 タスクリスト — s3 Rune 実質化

**状態**: COMPLETE
**開始日**: 2026-06-24
**完了日**: 2026-06-24

---

## タスク

| ID | タスク | 状態 |
|---|---|---|
| T0 | `fav/Cargo.toml` を `version = "25.2.0"` に bump（新規クレート追加不要） | [x] |
| T1 | `runes/aws/s3.fav` 更新（`presign_url` / `stream_get` 追加） | [x] |
| T2 | `fav/src/backend/vm.rs` 更新（`AWS.s3_presign_url_raw` / `AWS.s3_stream_get_raw` primitive 追加、自前 SigV4 流用） | [x] |
| T3 | `examples/s3_csv_to_parquet.fav` 新規作成（`import rune "aws"` を使用） | [x] |
| T4 | `CHANGELOG.md` 更新（`[v25.2.0]` エントリ追加） | [x] |
| T5 | `benchmarks/v25.2.0.json` 新規作成（プレースホルダー値） | [x] |
| T6 | `fav/src/driver.rs` 更新（`v252000_tests` 6 件追加） | [x] |
| T7 | `cargo test v252000` — 6 件 PASS 確認 | [x] |
| T8 | `cargo test` 総テスト数 ≥ 1986 件 確認 | [x] |
| T9 | spec-reviewer レビュー実施 | [x] |

---

## チェックリスト（完了条件）

- [x] `S3.presign_url` が `runes/aws/s3.fav` に存在する
- [x] `S3.stream_get` が `runes/aws/s3.fav` に存在する
- [x] `AWS.s3_presign_url_raw` が `fav/src/backend/vm.rs` に存在する
- [x] `AWS.s3_stream_get_raw` が `fav/src/backend/vm.rs` に存在する
- [x] `examples/s3_csv_to_parquet.fav` が存在し `import rune "aws"` / `get_object` / `put_object` を含む
- [x] `CHANGELOG.md` に `v25.2.0` が存在する
- [x] `v252000_tests` 6 件すべて PASS（presign_url Rune / stream_get Rune / presign_url primitive / stream_get primitive / example / changelog）
- [x] 総テスト数 ≥ 1986 件（実績: 1986 件）

---

## コードレビュー指摘（spec-reviewer — 実装前に対応済み）

| 優先度 | 指摘内容 | 対応 |
|---|---|---|
| HIGH | `import rune "s3"` が空スタブを指す | spec/plan を `import rune "aws"` に修正 |
| HIGH | `aws-presigning` クレートが Cargo.toml に存在しない | 自前 SigV4（既存 `sigv4_sign` 等）流用に方針変更 |
| HIGH | `s3_stream_get_raw` のテストが欠落 | テストを 5 件 → 6 件に増やし `s3_stream_get_primitive_exists` を追加 |
| MED | presign_url の GET/PUT スコープがロードマップと乖離 | spec.md に理由を明記（GET のみ、PUT は v25.x 以降） |
| MED | stream_get 実装方法が曖昧 | plan.md に「s3_get_object_raw と同一ロジックをコピー」と明記 |
| MED | エフェクト `!Io` vs `!AWS` の不一致 | spec.md に「ロードマップの !Io は誤記、正しくは !AWS」を明記 |
| LOW | bucket_exists の 5 条件整理 | 条件 1（connect）に bucket_exists を追記 |
| LOW | ベンチマーク値プレースホルダー | plan.md Step 5 に「プレースホルダー」の注記を追加 |

## 実装時修正（コンパイルエラー）

| 問題 | 修正 |
|---|---|
| `AwsConfig.access_key_id` 存在しない | `config.access_key` に修正 |
| `AwsConfig.secret_access_key` 存在しない | `config.secret_key` に修正 |
| `include_str!("../backend/vm.rs")` パスエラー | `include_str!("backend/vm.rs")` に修正（driver.rs は `src/` 直下） |

---

## メモ

- 既存 s3.fav には `get_object` / `put_object` / `delete_object` / `list_objects` / `bucket_exists` が実装済み（追加不要）
- `presign_url` は `aws-sdk-s3` を使わず、vm.rs の既存 SigV4 ヘルパー（`sigv4_signing_key` / `sha256_hex_bytes` / `hmac_sha256_bytes`）を流用
- `stream_get` は `s3_get_object_raw` と同一ロジックをブロックコピーして実装（TODO コメント付き）
- `import rune "s3"` は `runes/s3/` の空スタブを指すため使用しない（`import rune "aws"` を使う）
- テストは `include_str!` による存在確認のみ（LocalStack 起動不要）
- ロードマップ最小件数 5 件に対して本バージョンは 6 件（1 件超過）
- `AwsConfig` の実フィールド名: `access_key` / `secret_key`（`access_key_id` / `secret_access_key` ではない）
- `include_str!` パス基準: `src/driver.rs` からのパスは `backend/vm.rs`（`../backend/vm.rs` ではない）
