# Favnir ロードマップ v10.1.0 → v11.0.0 — Snowflake ネイティブ対応

## 前提（v10.0.0 完了時点）

- セルフホスト完成（checker.fav / compiler.fav / cli.fav）
- エフェクト型：`!Io` / `!Db` / `!Http` / `!Llm` が動作済み
- Rune エコシステム：AWS / DuckDB / SQL / http / grpc / graphql / llm / csv / json / gen 等
- テスト 1260 件通過・OSS 公開準備完了

---

## 方針

Snowflake は **Snowflake SQL API v2**（REST）経由でアクセスする。
`ureq` はすでに `!Llm` 実装で導入済みのため、追加依存ゼロで接続できる。

エフェクト追加パターンは `!Http`（v9.5.0）・`!Llm`（v9.6.0）と同じ 8 ファイル更新で実現する。
Rune 設計は「型安全なクエリ・スキーマ自動生成・リネージ可視化」を三本柱とする。

---

## v10.1.0 — インフラ構築（Terraform）

**テーマ**: Snowflake on AWS の Terraform 基盤を整備する

**背景**

Snowflake は AWS 上でホストでき、既存の `infra/` Terraform 構成と統合できる。
実 Snowflake インスタンスへの接続情報（アカウント ID・ウェアハウス・ロール等）を
安全に管理する仕組みをこのフェーズで確立する。

**やること**

- `infra/snowflake/` ディレクトリ作成（Terraform）
- Snowflake プロバイダー設定（`snowflake-labs/snowflake`）
- 接続情報を AWS SSM Parameter Store / Secrets Manager で管理
- ウェアハウス・データベース・スキーマ・ロールの Terraform リソース定義
- `infra/snowflake/README.md`（セットアップ手順）

**完了条件**

- `terraform plan` が通る
- Snowflake 接続情報が SSM に格納されている
- `infra/snowflake/README.md` に手順が記載されている

---

## v10.2.0 — VM Primitive 追加（Snowflake SQL API v2）

**テーマ**: Rust VM に Snowflake 接続用 primitive を追加する

**背景**

Snowflake SQL API v2 は JWT Bearer 認証の REST API。
`ureq`（`!Llm` で既導入）で `POST https://<account>.snowflakecomputing.com/api/v2/statements` を叩く。
このフェーズでは Rune・エフェクト型は追加せず、VM primitive のみを実装する。

**やること**

- `vm.rs`: `Snowflake.execute_raw(account, token, sql) -> Result<String, String>` 追加
- `vm.rs`: `Snowflake.query_raw(account, token, sql) -> Result<String, String>` 追加
- JWT 生成（RSA キーペアによる署名）ヘルパーを `vm.rs` に実装
- 環境変数: `SNOWFLAKE_ACCOUNT` / `SNOWFLAKE_PRIVATE_KEY` / `SNOWFLAKE_USER` / `SNOWFLAKE_ROLE`
- テスト: mock なし（環境変数未設定時は `Err("SNOWFLAKE_ACCOUNT is not set")` を返す）

**完了条件**

- `vm.rs` に `Snowflake.*_raw` primitive が追加されている
- 環境変数未設定時に適切な `Err` が返る
- `cargo test` 全件通過

---

## v10.3.0 — Effect::Snowflake 追加（8 ファイル更新）

**テーマ**: `!Snowflake` エフェクト型を言語に追加する

**背景**

`!Http`（v9.5.0）・`!Llm`（v9.6.0）と同じパターンで 8 ファイルを更新する。
`stage Query: String -> List<Row> !Snowflake` のように書けるようになる。

**やること**

- `ast.rs`: `Effect::Snowflake` 追加
- `parser.rs`: `"!Snowflake"` トークン解析
- `fmt.rs`: エフェクト文字列表現
- `lineage.rs`: リネージ解析に Snowflake エフェクト追加
- `driver.rs`: エフェクト表示・ドキュメント生成対応
- `ast_lower_checker.rs`: エフェクト lowering
- `checker.rs`: `require_snowflake_effect`（E0320）追加・Snowflake.* 型シグネチャ追加
- `reachability.rs`: 到達可能性解析

**完了条件**

- `stage Foo: String -> String !Snowflake = ...` が型チェックを通る
- エフェクト未宣言の stage で `Snowflake.*` を呼ぶと E0320 が出る
- `fav explain --lineage` で `!Snowflake` が表示される
- `cargo test` 全件通過

---

## v10.4.0 — checker.fav 更新（Snowflake 型チェック）

**テーマ**: セルフホスト型チェッカーに Snowflake を認識させる

**背景**

Rust checker に `!Snowflake` を追加しても、セルフホスト経路（checker.fav）は
独立して更新が必要。`!Http` / `!Llm` と同じパターンで追加する。

**やること**

- `checker.fav`: `snowflake_fn` 追加（Snowflake.* の型シグネチャ）
- `checker.fav`: `builtin_ret_ty` に Snowflake エントリ追加
- `checker.fav`: `ns_to_effect` に `"Snowflake" -> "!Snowflake"` 追加
- `checker.fav`: E0320 エラーコード追加
- テスト: `snowflake_effect_checker_fav`
- `cargo test checker_fav_wire_self_check` 通過確認

**完了条件**

- `fav check` が `!Snowflake` を正しく解析できる
- Snowflake.* を `!Snowflake` なし stage で使うと E0320
- `cargo test checker_fav_wire_self_check` 通過

---

## v10.5.0 — compiler.fav 更新（Snowflake NS 登録）

**テーマ**: セルフホストコンパイラに Snowflake 名前空間を登録する

**背景**

compiler.fav の builtin NS リストに `"Snowflake"` を追加しないと
Favnir pipeline 経由で `Snowflake.*` を呼ぶと `"global index out of bounds"` になる
（`!Llm` 追加時に同じ問題を踏んだ）。

**やること**

- `compiler.fav`: builtin NS リストの 2 箇所に `"Snowflake"` 追加
- テスト: `snowflake_compiles_with_favnir_pipeline`
- `cargo test bootstrap` 通過確認

**完了条件**

- Favnir pipeline で Snowflake.* を含む stage がコンパイルできる
- `cargo test bootstrap` 通過

---

## v10.6.0 — Snowflake Rune 実装（runes/snowflake/）

**テーマ**: `import rune "snowflake"` で使える Rune を実装する

**背景**

DuckDB Rune（v4.x）・SQL Rune（v7.2.0）と同じ構造で Snowflake Rune を実装する。
型安全なクエリ・レコード挿入・バルクロードを Favnir コードから書けるようにする。

**やること**

- `runes/snowflake/rune.toml` 作成
- `runes/snowflake/client.fav`: 接続管理（`connect(config) -> SnowflakeConn !Snowflake`）
- `runes/snowflake/query.fav`: `query<T>(conn, sql) -> List<T> !Snowflake`
- `runes/snowflake/execute.fav`: `execute(conn, sql) -> Int !Snowflake`（DML）
- `runes/snowflake/bulk.fav`: `copy_into(conn, stage, table) -> Int !Snowflake`（COPY INTO）
- テスト: Rune ロード確認（実接続なし）

**完了条件**

- `import rune "snowflake"` が通る
- `snowflake.query<Order>(conn, "SELECT ...")` が型チェックを通る
- テスト通過

---

## v10.7.0 — fav.toml Snowflake 設定対応

**テーマ**: プロジェクト設定で Snowflake 接続先を管理する

**背景**

DB Rune は `fav.toml` の `[database]` セクションで接続 URL を管理している。
Snowflake も同様に `[snowflake]` セクションで設定を管理できるようにする。
環境変数への参照（`${SNOWFLAKE_ACCOUNT}`）も書けるようにする。

**やること**

- `toml.rs`: `[snowflake]` セクション解析（account / user / role / warehouse / database / schema）
- `vm.rs`: `fav.toml` の Snowflake 設定を Rune 実行時に注入
- `fav new` テンプレートに `[snowflake]` コメントアウト例を追加
- テスト: `toml_snowflake_section_parsed`

**完了条件**

- `fav.toml` に `[snowflake]` を書くと設定が読み込まれる
- `${ENV_VAR}` 形式の環境変数参照が展開される
- テスト通過

---

## v10.8.0 — fav infer --from snowflake（スキーマ自動生成）

**テーマ**: Snowflake テーブル定義から Favnir 型を自動生成する

**背景**

`fav infer` は CSV / JSON からの型推論を v6.6.0 で実装済み。
Snowflake の `INFORMATION_SCHEMA.COLUMNS` を参照して
Favnir の `type` 定義を自動生成する機能を追加する。

**やること**

- `driver.rs`: `cmd_infer_snowflake(table, conn_config) -> String` 追加
- Snowflake → Favnir 型マッピング（NUMBER→Int/Float、VARCHAR→String、BOOLEAN→Bool 等）
- `cli.fav`: `fav infer --from snowflake --table <name>` サブコマンド追加
- 生成例:
  ```favnir
  // Generated by fav infer --from snowflake --table ORDERS
  type Orders = { order_id: Int  customer: String  amount: Float }
  ```
- テスト: 型マッピング単体テスト（実接続不要）

**完了条件**

- `fav infer --from snowflake --table ORDERS` が型定義を標準出力する
- Snowflake 型 → Favnir 型のマッピングテストが通る

---

## v10.9.0 — E2E テスト（実 Snowflake インスタンス）

**テーマ**: 実 Snowflake インスタンスを使った E2E 証明

**背景**

`infra/e2e-demo`（ECS / EKS / Lambda）と同じ構造で Snowflake E2E を実装する。
`infra/e2e-demo/snowflake/` に証跡を残す。

**やること**

- `infra/e2e-demo/snowflake/` ディレクトリ作成
- デモシナリオ: CSV → Snowflake ロード → クエリ → S3 サマリー出力
- `demo.fav`: LoadCsv |> TransformRows |> SnowflakeInsert |> QuerySummary
- Terraform でデモ用 Snowflake リソース（テーブル・ウェアハウス）を定義
- 証跡: `s3://favnir-e2e-demo/proof/snowflake/`
- `infra/e2e-demo/snowflake/README.md`

**完了条件**

- `demo.fav` が実 Snowflake に対して PASS=4 / FAIL=0
- 証跡が S3 に保存されている
- README に実行手順が記載されている

---

## v11.0.0 — Snowflake 統合完成 + リネージ可視化 + ドキュメント

**テーマ**: Snowflake ネイティブ対応の完成宣言・ドキュメント整備

**背景**

v10.1.0〜v10.9.0 の成果を統合し、Snowflake を Favnir の「第一級データソース」として
公式にサポートする。`fav explain --lineage` での Snowflake エフェクト可視化を完成させ、
サイトドキュメント・CHANGELOG を更新する。

**やること**

- `fav explain --lineage` で `!Snowflake(read)` / `!Snowflake(write)` を区別表示
- リネージ出力例:
  ```
  seq ETL: String -> Summary  !Io, !Snowflake(read), !Snowflake(write)
    LoadCsv      !Io
    SnowflakeInsert  !Snowflake(write)
    QuerySummary     !Snowflake(read)
  ```
- CHANGELOG.md に v10.1.0〜v11.0.0 を追記
- README.md の Rune エコシステム表に `snowflake` 追加
- サイトドキュメント: Snowflake Rune リファレンスページ追加
- `fav/Cargo.toml` version → `"11.0.0"`
- `fav/self/cli.fav` の `run_version` → `"11.0.0"`
- `cargo test` 全件通過

**完了条件**

- `!Snowflake(read/write)` がリネージ出力に含まれる
- `cargo test` 全件通過・bootstrap 維持
- サイトに Snowflake Rune ドキュメントが公開されている
- CHANGELOG・README が最新状態

---

## スケジュール概観

| バージョン | テーマ | 主な変更対象 |
|---|---|---|
| v10.1.0 | Terraform インフラ | `infra/snowflake/` |
| v10.2.0 | VM Primitive | `vm.rs` |
| v10.3.0 | Effect::Snowflake | 8 ファイル（ast / parser / checker 等） |
| v10.4.0 | checker.fav 更新 | `fav/self/checker.fav` |
| v10.5.0 | compiler.fav 更新 | `fav/self/compiler.fav` |
| v10.6.0 | Snowflake Rune | `runes/snowflake/` |
| v10.7.0 | fav.toml 設定 | `toml.rs` / `vm.rs` / `cli.fav` |
| v10.8.0 | fav infer --from snowflake | `driver.rs` / `cli.fav` |
| v10.9.0 | E2E テスト | `infra/e2e-demo/snowflake/` |
| v11.0.0 | 統合完成・ドキュメント | リネージ / サイト / CHANGELOG |

**v10.3.0 の 8 ファイル更新パターン（`!Http` / `!Llm` と同じ）:**

```
ast.rs / parser.rs / fmt.rs / lineage.rs
driver.rs / ast_lower_checker.rs / checker.rs / reachability.rs
```
