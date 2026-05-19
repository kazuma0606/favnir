# Favnir v5.2.0 タスクリスト — パッケージ仕様 + Registry 拡張

作成日: 2026-05-20

---

## Phase A: `rune.toml` フォーマット定義

- [ ] 各 Rune のエントリポイント（`<name>.fav` または `main.fav`）を確認
- [ ] `runes/auth/rune.toml` を作成
- [ ] `runes/aws/rune.toml` を作成
- [ ] `runes/csv/rune.toml` を作成
- [ ] `runes/db/rune.toml` を作成
- [ ] `runes/duckdb/rune.toml` を作成
- [ ] `runes/env/rune.toml` を作成
- [ ] `runes/gen/rune.toml` を作成
- [ ] `runes/grpc/rune.toml` を作成
- [ ] `runes/http/rune.toml` を作成
- [ ] `runes/incremental/rune.toml` を作成
- [ ] `runes/json/rune.toml` を作成
- [ ] `runes/log/rune.toml` を作成
- [ ] `runes/parquet/rune.toml` を作成
- [ ] `runes/stat/rune.toml` を作成
- [ ] `runes/validate/rune.toml` を作成
- [ ] `spec.md` の `rune.toml` フォーマット（A-1, A-2）を `docs/rune-toml-spec.md` として公開用にコピー（任意）

---

## Phase B: 新規 VM Primitive

- [ ] vm.rs の `String.base64_encode` 実装を読んで crate 名・定数名を確認
- [ ] `String.base64_decode` を vm.rs に実装
- [ ] `AWS.s3_get_object_base64_raw` を vm.rs に実装
- [ ] `AWS.s3_put_bytes_raw` を vm.rs に実装
- [ ] `AWS.s3_list_objects_raw` を vm.rs に実装
- [ ] 上記 4 関数の型シグネチャを checker.rs に追加
- [ ] vm_stdlib_tests.rs に `test_string_base64_decode` を追加
- [ ] vm_stdlib_tests.rs に `test_string_base64_decode_invalid` を追加
- [ ] vm_stdlib_tests.rs に `test_string_base64_roundtrip` を追加
- [ ] `cargo test` が通る

---

## Phase C: Registry S3 キー変更

- [ ] `rune-registry/src/main.fav` の `save_rune` を zip blob 対応に変更
  - [ ] `String.base64_decode(zip_b64)` → `List<Int>`
  - [ ] `AWS.s3_put_bytes_raw(bucket, "{name}/{version}.zip", bytes)` で保存
  - [ ] DynamoDB 書き込みは変更なし（name, version, description）
- [ ] `handle_publish` を変更: `zip` フィールドを必須取得
- [ ] S3 キー形式が `{name}/{version}.zip` になっていることをコードで確認

---

## Phase D: 新エンドポイント実装

- [ ] `String.contains` / `String.index_of` の存在を vm.rs で grep 確認
  - [ ] 存在しない場合: 代替実装を plan.md の注意事項に従って決定
- [ ] `handle_versions(name)` 関数を main.fav に追加
  - [ ] `AWS.s3_list_objects_raw` でキー一覧取得
  - [ ] `{name}/{version}.zip` → `{version}` を抽出
  - [ ] バージョン配列を JSON で返す
- [ ] `handle_download(name, version)` 関数を main.fav に追加
  - [ ] `AWS.s3_get_object_base64_raw` で取得
  - [ ] `is_base64: "true"` を含むレスポンスマップを返す
- [ ] `route()` 関数を更新: 3 セグメントパスを処理
  - [ ] `/runes/{name}/versions` → `handle_versions`
  - [ ] `/runes/{name}/download` → `handle_download`（version クエリパラメータ対応）
  - [ ] version 未指定時は DynamoDB から最新バージョンを取得
- [ ] `main()` 関数に `FAV_QUERY_VERSION` 読み取りを追加
- [ ] `route()` の req マップに `"query_version"` フィールドを追加

---

## Phase E: Bootstrap 変更

- [ ] `rune-registry/bootstrap` を読んで現在の構造を確認
- [ ] `FAV_QUERY_VERSION` 抽出（`queryStringParameters.version`）を追加
- [ ] `is_base64: "true"` レスポンスへの対応を追加
  - [ ] `isBase64Encoded: true` を Lambda レスポンスに含める
  - [ ] `Content-Type: application/zip` を設定

---

## Phase F: デプロイ + エンドツーエンドテスト

- [ ] `cargo build` が通る
- [ ] `cargo test` が全件（956 件 + 新規テスト）pass
- [ ] master push で GitHub Actions が自動ビルド・デプロイ完了
- [ ] エンドツーエンドテスト:
  - [ ] `POST /runes/csv` に base64 zip を送って `201 published` が返る
  - [ ] `GET /runes/csv/versions` が `["0.2.0"]` 形式の配列を返す
  - [ ] `GET /runes/csv/download?version=0.2.0` が zip バイナリを返す
  - [ ] `GET /runes` が既存の一覧を返す（後退テスト）
  - [ ] `GET /runes/csv` が既存の詳細を返す（後退テスト）
- [ ] `rune-registry/SPEC.md` を更新（新 API エンドポイントを追記）

---

## 完了条件

- [ ] `cargo build` が通る
- [ ] 既存テスト全件 + 新規 3 件（base64_decode 系）が pass
- [ ] 全 15 Rune に `rune.toml` が存在する
- [ ] Registry の新エンドポイント（/versions, /download）が本番で動作する
- [ ] `POST /runes/{name}` が zip blob を受け付け `{name}/{version}.zip` で S3 保存する

完了予定: v5.3.0 開始前

---

## 備考

- **`String.contains` / `String.index_of` 不在の場合**: Phase D のルーター実装前に vm.rs + checker.rs に追加するか、Favnir コードで代替実装（文字列分割）を使う。追加する場合は Phase B に含める。
- **`Json.write_array_raw` の制約**: バージョン文字列の純粋な JSON 配列が組み立てられない場合、文字列ジョイン方式（plan.md 参照）で対応。
- **旧 S3 データ**: `{name}` 形式（バージョンなし）の旧キーは削除 OK。本番 S3 には現時点でテストデータのみ存在するため影響なし。
