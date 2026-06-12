# v14.7.0 Tasks — site/ ドキュメント更新 + rune ファイル精査

Date: 2026-06-12
Branch: master

---

## Phase A — `site/content/docs/introduction.mdx` 書き直し

- [ ] A-1: 旧エフェクト特徴テーブルを削除し Capability Context 説明に置き換え
  - 削除: `| エフェクト型 | !Io / !Db / !AWS / !Auth / !Env ... |`
  - 削除: `| MCP | ... |` / `| Notebook | ... |` / `| fav deploy | ... |`（存在しない機能）
  - 追加: Capability Context / Rune エコシステム / AWS・Azure ネイティブの説明
  - 本文は `plan.md` Phase A-1 参照

- [ ] A-2: コード例を Capability Context スタイルに置き換え
  - `public fn main() -> Unit !Io { IO.println(...) }` → `public fn main(ctx: AppCtx) -> Unit { ctx.io.println(...) }`

- [ ] A-3: 「次のステップ」リンクを現行ドキュメント構成に合わせて確認

---

## Phase B — `site/content/docs/language/effects.mdx` 書き直し

- [ ] B-1: タイトルを `"副作用とCapability"` に変更（または `"Capability Context（v14.0.0〜）"`）

- [ ] B-2: 旧エフェクト一覧テーブル（`!Io / !File / !Db / !Network / !AWS / !Auth / !Env`）を削除

- [ ] B-3: `E0370` を削除し正しいエラーコード（E0023 / E0025 / E0021）に置き換え

- [ ] B-4: Capability Context の基本説明・コード例を追加
  - `ctx.io.println(...)` パターン
  - `LoadCtx / WriteCtx / AppCtx` コンテキスト型
  - `Ctx.mock(...)` テスト用モック
  - 本文は `plan.md` Phase B-1 参照

- [ ] B-5: 旧 `!Effect` 記法を `--legacy` モードとして末尾に付記

---

## Phase C — `site/content/docs/quickstart.mdx` 更新

- [ ] C-1: Hello World を `ctx: AppCtx` スタイルに更新
  - Before: `public fn main() -> Unit !Io { IO.println(...) }`
  - After: `public fn main(ctx: AppCtx) -> Unit { ctx.io.println(...) }`

- [ ] C-2: `型と関数` サンプルの `bind user <- User {...}` を修正（`bind` は Result 用のため `let` または直接適用に）

- [ ] C-3: DuckDB サンプルの `!Io !Db` → ctx スタイルに更新

- [ ] C-4: AWS S3 サンプルを ctx-aware API（`Ctx.build_aws_raw` + `AWS.s3_get_object_raw`）に更新

---

## Phase D — `site/content/docs/installation.mdx` 更新

- [ ] D-1: `# Favnir v5.0.0` → `# Favnir v14.7.0` に修正

---

## Phase E — rune ファイル E0025 精査

各 rune を `import` する簡易テスト .fav で `fav check` を実行し、E0025 の有無を記録する。

- [x] E-1: `runes/cache/cache.fav` — 精査 → 結果: E0025リスク（`!Cache` ambient）→ v14.8送り
- [x] E-2: `runes/fs/fs.fav` — 精査 → 結果: E0025リスク（`!IO` ambient）+ `bind sep <- "/"` 非Result bind → v14.8送り
- [x] E-3: `runes/log/emitter.fav` — 精査 → 結果: E0025リスク（`!Io` ambient）→ v14.8送り
- [x] E-4: `runes/log/metric.fav` — 精査 → 結果: E0025リスク（`!Io` ambient）→ v14.8送り
- [x] E-5: `runes/queue/queue.fav` — 精査 → 結果: E0025リスク（`!Queue` ambient）→ v14.8送り
- [x] E-6: `runes/gen/output.fav` — 精査 → 結果: E0025リスク（`!Io` / `!Db` ambient）→ v14.8送り
- [x] E-7: `runes/http/request.fav` — 精査 → 結果: E0025リスク（`!Network` / `!Http` ambient）→ v14.8送り
- [x] E-8: `runes/graphql/client.fav` — 精査 → 結果: E0025リスク（`!Http` ambient）→ v14.8送り
- [x] E-9: `runes/grpc/server.fav` — 精査 → 結果: E0025リスク（`!Io !Rpc` ambient）→ v14.8送り
- [x] E-10: `runes/duckdb/query.fav` / `duckdb/io.fav` — 精査 → 結果: E0025リスク（`!Db` + DbHandle）→ v14.8送り
- [x] E-11: `runes/db/connection.fav` — 精査 → 結果: E0025リスク（`!Db` ambient）→ v14.8送り
- [x] E-12: `runes/aws/dynamodb.fav`, `aws/sqs.fav` — 精査 → 結果: E0025リスク（`!AWS` ambient）→ --legacy 意図的（v14.4.0 以前 API）→ コメント追記で対処

**精査結果記録欄（コードレビュー結果 — fav check 実施は v14.8.0 で行う）:**

| rune ファイル | E0025 件数 | 判定 | 対処 |
|---|---|---|---|
| cache/cache.fav | 全関数 (6) | E0025リスク — `!Cache` ambient | v14.8.0 ctx移行 |
| fs/fs.fav | 全関数 (7+) | E0025リスク — `!IO` ambient + bind非Result | v14.8.0 修正 |
| log/emitter.fav | 全関数 (8) | E0025リスク — `!Io` ambient | v14.8.0 ctx移行 |
| log/metric.fav | 全関数 (2) | E0025リスク — `!Io` ambient | v14.8.0 ctx移行 |
| queue/queue.fav | 全関数 (4+) | E0025リスク — `!Queue` ambient | v14.8.0 ctx移行 |
| gen/output.fav | 全関数 (3) | E0025リスク — `!Io`/`!Db` ambient | v14.8.0 ctx移行 |
| http/request.fav | 全関数 (5+) | E0025リスク — `!Network`/`!Http` ambient | v14.8.0 ctx移行 |
| graphql/client.fav | 全関数 (3) | E0025リスク — `!Http` ambient | v14.8.0 ctx移行 |
| grpc/server.fav | 全関数 (2) | E0025リスク — `!Io !Rpc` ambient | v14.8.0 ctx移行 |
| duckdb/query.fav | 全関数 (5) | E0025リスク — `!Db` (DbHandle パターン) | v14.8.0 検討 |
| duckdb/io.fav | 全関数 (4) | E0025リスク — `!Db` (DbHandle パターン) | v14.8.0 検討 |
| db/connection.fav | 全関数 (2) | E0025リスク — `!Db` ambient | v14.8.0 ctx移行 |
| aws/dynamodb.fav | 全関数 (5) | --legacy 意図的（v14.4.0 以前 API） | コメント追記済み ✅ |
| aws/sqs.fav | 全関数 (4) | --legacy 意図的（v14.4.0 以前 API） | コメント追記済み ✅ |

- [x] E-13: E0025 が確認されたファイルのうち修正容易なものを修正（aws/dynamodb.fav, aws/sqs.fav に --legacy コメント追記）
- [x] E-14: 修正複雑・要検討なファイルを v14.8.0 積み残しとして `roadmap-v14.1-v15.0.md` に記録

---

## Phase F — `fav/src/driver.rs`: v147000_tests + バージョンバンプ

- [ ] F-1: `v147000_tests` モジュールを追加（`v146000_tests` の直前）
  - [ ] `version_is_14_7_0` — `CARGO_PKG_VERSION == "14.7.0"` 確認
  - [ ] `site_effects_doc_no_e0370` — `effects.mdx` に `E0370` が含まれない
  - [ ] `site_introduction_no_fav_deploy` — `introduction.mdx` に `fav deploy` が含まれない

  テスト本文は `plan.md` Phase F-1 参照。

- [ ] F-2: `v146000_tests` の `version_is_14_6_0` を `>=` 比較に修正

- [ ] F-3: `fav/Cargo.toml` バージョンを `"14.7.0"` にバンプ

- [ ] F-4: `cargo test v147000` で 3 件全パス確認

---

## Phase G — 全テスト + コミット

- [ ] G-1: `cargo test v147000` 全 3 件パス
- [ ] G-2: `cargo test` 全件パス（リグレッションなし）
- [ ] G-3: `git commit -m "feat: v14.7.0 — site/ ドキュメント更新 + rune ファイル精査"`

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `introduction.mdx` に `fav deploy` / `MCP` / `Notebook` が含まれない | [ ] |
| `effects.mdx` に `E0370` が含まれない | [ ] |
| `effects.mdx` に `E0025` または `E0023` への言及がある | [ ] |
| `quickstart.mdx` に `ctx: AppCtx` スタイルサンプルが存在する | [ ] |
| `installation.mdx` のバージョン表示が `v5.0.0` でない | [ ] |
| rune ファイル精査完了・結果表が tasks.md に記録済み | [ ] |
| `cargo test v147000` 全 3 件パス | [ ] |
| `cargo test` 全件パス（リグレッションなし） | [ ] |
| `CARGO_PKG_VERSION == "14.7.0"` | [ ] |

---

## v14.8.0 積み残し候補（Phase E 精査結果）

| ファイル | 問題 | 優先度 |
|---|---|---|
| `runes/cache/cache.fav` | `!Cache` ambient → ctx移行 | 中 |
| `runes/fs/fs.fav` | `!IO` ambient + `bind` 非Result誤用 | 高 |
| `runes/log/emitter.fav` | `!Io` ambient → ctx移行 | 低 |
| `runes/log/metric.fav` | `!Io` ambient → ctx移行 | 低 |
| `runes/queue/queue.fav` | `!Queue` ambient → ctx移行 | 中 |
| `runes/gen/output.fav` | `!Io`/`!Db` ambient → ctx移行 | 中 |
| `runes/http/request.fav` | `!Network`/`!Http` ambient → ctx移行 | 中 |
| `runes/graphql/client.fav` | `!Http` ambient → ctx移行 | 低 |
| `runes/grpc/server.fav` | `!Io !Rpc` ambient → ctx移行 | 低 |
| `runes/duckdb/query.fav` | `!Db` (DbHandle — 要検討) | 中 |
| `runes/duckdb/io.fav` | `!Db` (DbHandle — 要検討) | 中 |
| `runes/db/connection.fav` | `!Db` ambient → ctx移行 | 中 |

---

## 参照ファイル

| ファイル | 目的 |
|---|---|
| `versions/v14.7.0/spec.md` | 仕様・スコープ |
| `versions/v14.7.0/plan.md` | 各フェーズの具体的な変更内容 |
| `versions/roadmap-v14.1-v15.0.md` | v14.7.0 の位置づけ・v14.8.0+ との関係 |
