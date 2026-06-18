# v14.6.0 Spec — ドキュメント整備（README + CHANGELOG）

Date: 2026-06-12

---

## 目的

v14.1.0〜v14.5.0 で実装した機能のドキュメントが積み残し状態になっている。
v15.0.0 CrossCloud E2E デモの前に、README と CHANGELOG を現状に合わせて修正する。

コードの変更は原則なし。純粋なドキュメント更新バージョン。

---

## 現状（v14.5.0 時点）

| リソース | 状態 |
|---|---|
| `CHANGELOG.md` v14.1.0〜v14.5.0 エントリ | **欠落**（v14.0.0 の次が v13.0.0 になっている） |
| `README.md`「現在の状態」見出し | **`v10.0.0` のまま**（実際は v14.5.0） |
| `README.md` ロードマップ表 | **v14.0.0 で止まっている**（v14.1.0〜v14.5.0 未掲載） |
| `README.md` コード例（stage 定義） | **旧 `!Effect` スタイル**（`!Io`, `!Db` 等）で記述されており、v14.0.0 Capability Context との矛盾が見える |
| `README.md` Rune 一覧 | Azure Blob / Azure Postgres が未掲載 |
| `fav/src/driver.rs` バージョン | `14.5.0`（バンプ対象） |

---

## ユーザー体験（Before / After）

### Before（v14.5.0 まで）

```
README を見ると「現在の状態: v10.0.0」「ロードマップ: v14.0.0 が最新」
CHANGELOG を見ると v14.1.0〜v14.5.0 のエントリが存在しない
→ 外部から見てプロジェクトが止まっているように見える
```

### After（v14.6.0）

```
README: 「現在の状態: v14.6.0」「ロードマップ: v14.1.0〜v14.6.0 完了」
CHANGELOG: v14.1.0〜v14.6.0 のエントリが揃っている
コード例: 旧スタイルに「--legacy モードでのみ有効」の注記、または v14.0.0 スタイルに差し替え
```

---

## スコープ

### In Scope

| 項目 | 内容 |
|---|---|
| `CHANGELOG.md` 追記 | v14.1.0〜v14.5.0 の各バージョンエントリ |
| `README.md` 現在の状態 | 見出しを v14.6.0 に更新、説明文に v14.1.0〜v14.6.0 を追記 |
| `README.md` ロードマップ表 | v14.1.0〜v14.6.0 を「完了」として追記 |
| `README.md` コード例 | 旧 `!Effect` スタイルに注記を追加 |
| `README.md` Rune 一覧 | Azure Blob Storage / Azure PostgreSQL 行を追加 |
| `fav/Cargo.toml` バージョン | `14.5.0` → `14.6.0` |
| `driver.rs` v146000_tests | 3 件のスモークテスト |

### Out of Scope

- site/content/docs/ の更新（→ v14.7.0）
- rune ファイルの修正（→ v14.7.0）
- 機能追加・コード変更

---

## CHANGELOG エントリ設計

追加する 5 エントリの概要:

| バージョン | 日付 | テーマ | 主な追加 |
|---|---|---|---|
| v14.1.0 | 2026-06-12 | Azure PostgreSQL Rune | AzurePostgres.* VM primitive / checker / lineage / runes/azure-postgres/ |
| v14.2.0 | 2026-06-12 | AzureCtx + fav.toml [azure] | Ctx.build_azure_raw / Ctx.azure_get_field_raw / Ctx.build_aws_raw / fav.toml [azure] |
| v14.3.0 | 2026-06-12 | Azure lineage + !AzureStorage | Effect::AzureStorage / lineage AzureBlobRead/Write / BUILTIN_EFFECTS 更新 |
| v14.4.0 | 2026-06-12 | AWS Rune 正式パッケージング | AWS.secrets_get_raw / runes/aws/secrets.fav / runes/aws/s3.fav ctx-aware ラッパー |
| v14.5.0 | 2026-06-12 | Azure Blob Storage Rune | azure_blob_sign / AzureBlob.put_raw/get_raw/list_raw/delete_raw / runes/azure-blob/ |

---

## 完了条件

| 確認項目 | 目標 |
|---|---|
| `CHANGELOG.md` に `[v14.5.0]` エントリが存在 | ✅ |
| `CHANGELOG.md` に `[v14.1.0]` エントリが存在 | ✅ |
| `README.md` に `v14.6.0` の記述が存在 | ✅ |
| `README.md` のロードマップ表に `v14.5.0` 行が存在 | ✅ |
| `README.md` に `AzureBlob` への言及が存在 | ✅ |
| `cargo test v146000` 全 3 件パス | ✅ |
| `cargo test` 全件パス（リグレッションなし） | ✅ |
| `CARGO_PKG_VERSION == "14.6.0"` | ✅ |
