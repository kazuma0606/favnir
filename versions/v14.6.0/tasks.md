# v14.6.0 Tasks — ドキュメント整備（README + CHANGELOG）

Date: 2026-06-12
Branch: master

---

## Phase A — `CHANGELOG.md`: v14.1.0〜v14.5.0 エントリ追加

- [ ] A-1: `## [v14.5.0]` エントリを追加（`## [v14.0.0]` の直前）
  - テーマ: Azure Blob Storage Rune
  - 主な追加: `azure_blob_sign` / `AzureBlob.put_raw/get_raw/list_raw/delete_raw` / runes/azure-blob/ / E0317
  - 本文は `plan.md` Phase A-1 参照

- [ ] A-2: `## [v14.4.0]` エントリを追加（v14.5.0 の直後）
  - テーマ: AWS Rune 正式パッケージング
  - 主な追加: `AWS.secrets_get_raw` / `Ctx.aws_get_field_raw` / runes/aws/secrets.fav / s3.fav ctx-aware ラッパー

- [ ] A-3: `## [v14.3.0]` エントリを追加
  - テーマ: Azure lineage + !AzureStorage エフェクト
  - 主な追加: `ast::Effect::AzureStorage` / `EffectKind::AzureBlob*` / `BUILTIN_EFFECTS` 更新

- [ ] A-4: `## [v14.2.0]` エントリを追加
  - テーマ: AzureCtx / AwsCtx + fav.toml [azure]
  - 主な追加: `Ctx.build_aws_raw` / `Ctx.build_azure_raw` / `Ctx.aws_get_field_raw` / `Ctx.azure_get_field_raw` / fav.toml [azure]

- [ ] A-5: `## [v14.1.0]` エントリを追加
  - テーマ: Azure PostgreSQL Rune
  - 主な追加: `AzurePostgres.execute_raw/query_raw` / `require_azure_db_effect` (E0316) / runes/azure-postgres/ / SSL 対応

- [ ] A-6: CHANGELOG の順序確認（新しい順: v14.5.0 → v14.4.0 → v14.3.0 → v14.2.0 → v14.1.0 → v14.0.0）

---

## Phase B — `README.md` 更新

- [ ] B-1: 「現在の状態」見出しを `v14.6.0` に更新
  - 見出し: `**v14.6.0（2026-06-12）— ドキュメント整備完了**`
  - テスト件数: `1530+ 件` に更新
  - v14.0.0 段落の後に v14.1.0〜v14.6.0 の説明文を追記

- [ ] B-2: 機能一覧表に Azure 行を追加
  - `Azure Blob Storage（AzureBlob.*、Shared Key 認証）` 行
  - `Azure PostgreSQL（AzurePostgres.*、SSL 対応）` 行
  - AWS 行の説明を `S3 / SQS / DynamoDB / Secrets Manager` に更新

- [ ] B-3: 旧 `!Effect` コード例に注記（blockquote）を追加
  - 「基本パイプライン」コードブロックの直前に警告注記
  - 書き直しは不要、注記追加のみ

- [ ] B-4: ロードマップ表に v14.1.0〜v14.6.0 行を追記
  - `v14.0.0` 行の直後に追加
  - `| v14.1.0〜v14.5.0 | Azure Blob/Postgres Rune / AWS Secrets Manager 等 | 完了 |`
  - `| **v14.6.0** | **ドキュメント整備** | **完了** |`

---

## Phase C — `fav/src/driver.rs`: v146000_tests + バージョンバンプ

- [ ] C-1: `v146000_tests` モジュールを追加（`v145000_tests` の直前）
  - [ ] `version_is_14_6_0` — `CARGO_PKG_VERSION == "14.6.0"` 確認
  - [ ] `changelog_has_v14_5_0_entry` — `CHANGELOG.md` に `[v14.5.0]` と `[v14.1.0]` が存在
  - [ ] `readme_mentions_azure_blob` — `README.md` に `Azure Blob` と `v14.5.0` or `v14.6.0` が存在

  テスト本文は `plan.md` Phase C-1 参照。

- [ ] C-2: `v145000_tests` の `version_is_14_5_0` を `>=` 比較に修正
  ```rust
  assert!(env!("CARGO_PKG_VERSION") >= "14.5.0", ...);
  ```

- [ ] C-3: `fav/Cargo.toml` バージョンを `"14.6.0"` にバンプ

- [ ] C-4: `cargo test v146000` で 3 件全パス確認

---

## Phase D — 全テスト + コミット

- [ ] D-1: `cargo test v146000` 全 3 件パス
- [ ] D-2: `cargo test` 全件パス（リグレッションなし）
- [ ] D-3: `git commit -m "feat: v14.6.0 — ドキュメント整備（README + CHANGELOG v14.1.0〜v14.5.0）"`

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `CHANGELOG.md` に `[v14.5.0]` エントリが存在 | [ ] |
| `CHANGELOG.md` に `[v14.1.0]` エントリが存在 | [ ] |
| `README.md` に `v14.6.0` の記述が存在 | [ ] |
| `README.md` のロードマップ表に `v14.5.0` 行が存在 | [ ] |
| `README.md` に `Azure Blob` への言及が存在 | [ ] |
| `cargo test v146000` 全 3 件パス | [ ] |
| `cargo test` 全件パス（リグレッションなし） | [ ] |
| `CARGO_PKG_VERSION == "14.6.0"` | [ ] |

---

## 参照ファイル

| ファイル | 目的 |
|---|---|
| `versions/v14.6.0/spec.md` | 仕様・ユーザー体験 |
| `versions/v14.6.0/plan.md` | 実装詳細・コードスニペット |
| `versions/roadmap-v14.1-v15.0.md` | v14.6.0 の位置づけ・依存関係 |
| `CHANGELOG.md` | 追記対象 |
| `README.md` | 更新対象 |
