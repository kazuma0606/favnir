# Favnir ロードマップ v6.3.0 → v7.x

作成日: 2026-05-26

v6.2.0（Bootstrap 検証完了）以降の進化の方針。

---

## 前提：v6.2.0 完了時点の状態

- Bootstrap 検証済み（`compiler.fav` が自分自身をコンパイル、bytecode_A == bytecode_B）
- `stage` / `seq` / `|>` 実装済み
- `abstract stage` / `abstract seq` 実装済み
- `fav explain` 実装済み（pipeline セクション対応）
- `fav infer`（CSV / SQLite / PostgreSQL / proto → 型定義）実装済み
- `schemas/*.yaml` 読み込み実装済み
- `fav build --schema`（DDL 生成）実装済み
- `T.validate` checker 統合（部分的）
- `favnir-wasm`（`fav_check` / `fav_compile`）実装済み
- Playground ページ実装済み（WASM 未デプロイ、基本型のみ対応）
- `fav deploy`（Lambda のみ）実装済み
- テスト: 1032 件通過

---

## 方針

**v6.x シリーズはバージョン数を固定しない。**
各バージョンは 1〜2 週間で完了できる粒度を目安とする。
v6.3、v6.4 … v6.9、v6.10 と続いても構わない。
v7.0.0 は「Schema Authority 完成」という明確なマイルストーンで区切る。

```
v6.x（柔軟な数）: セルフホスト仕上げ・ツールチェーン・公開準備
v7.0.0          : Schema Authority（外部データを型で守る完成形）
v7.1+           : データリネージ・SQL Rune・Rune エコシステム拡充
```

---

## v6.x シリーズ — 仕上げフェーズ

以下を**この順番で**進める。各項目が 1 バージョンに相当する。
スコープが広がりそうなら分割して次番号に回す。

---

### v6.3.0 — Self-host stage/seq

**テーマ**: `compiler.fav` が `stage` / `seq` / `|>` を処理できるようにする。

`compiler.fav` はこれらの構文を処理できず、Bootstrap 後に
stage/seq を使ったプログラムをコンパイルできない状態。

**やること**
- `compiler.fav` に `stage` / `seq` / `|>` のパース・lowering を追加
- `cargo test bootstrap_full_self_hosting` が引き続き通ること
- stage/seq を使ったプログラムに対する bootstrap 比較テストを追加

**完了条件**
- `compiler.fav` で stage/seq/|> を使ったプログラムをコンパイルできる
- 既存の bootstrap テストがすべて通る

---

### v6.4.0 — Playground 改善

**テーマ**: Playground WASM をデプロイ可能にし、対応型を拡張する。

現状 `/wasm/favnir.js` が未デプロイで、実行できる型も
Int/Float/Bool/String/Unit のみ。データエンジニア向けデモとして機能していない。

**やること**
- `wasm-pack build` の出力を `site/public/wasm/` に配置するデプロイパイプラインを整備
- WASM バックエンドで List / Record 型を対応
- Playground のサンプルコードを `stage`/`seq` を使ったパイプライン例に更新

**完了条件**
- `fav deploy-site` 後に Playground で型チェックと実行が動く
- List を使ったサンプルが Playground で実行できる

---

### v6.5.0 — サイトドキュメント補完

**テーマ**: 実装済みだがドキュメントが存在しない機能の docs を追加する。

**追加するドキュメント**
- `language/pipeline.mdx` — `stage` / `seq` / `|>` / `abstract seq`
- `language/schema.mdx` — `schemas/*.yaml` の書き方、制約一覧
- `stdlib/infer.mdx` — `fav infer --csv` / `--db` / `--proto` の使い方
- `rune-cli.mdx` 更新 — `fav deploy` / `fav build --schema` を追記

**完了条件**
- 上記 4 ページがサイトに追加されている
- コード例がすべて有効な Favnir 構文になっている

---

### v6.6.0 — T.validate 完成

**テーマ**: schema 駆動のランタイムバリデーションを完全実装する。

`T.validate` は部分的にしか実装されておらず、ユーザー向け API として
ドキュメント化もされていない。

**やること**
- `T.validate : Map<String, String> -> Result<T, List<String>>` の完全実装
  （positive / max_length / pattern / nullable 等、全制約を網羅）
- `db.query<T>` / `aws.s3.read_csv<T>` が内部で `T.validate` を自動呼び出し
- 統合テスト 10 件以上

**完了条件**
- `T.validate` が全制約を検査できる
- `db.query<Order>` が制約違反データで `Err` を返す

---

### v6.7.0 — fav deploy ECS/Fargate 対応

**テーマ**: `fav deploy` を Lambda のみから ECS/Fargate にも対応させる。

**やること**
- `fav deploy --target ecs` の実装（Docker ビルド → ECR push → ECS rolling update）
- `fav.toml [deploy] target = "ecs"` の設定スキーマを追加
- E2E デモ Terraform（`infra/e2e-demo/`）整備
  - EC2 版：Machine A（ツールチェーン）→ .fvc → Machine B（ランタイム）
  - EKS 版：favnir/toolchain Pod → favnir/runtime Pod

**完了条件**
- `fav deploy --target ecs --dry-run` が正しいステップを出力する
- E2E デモの Terraform が `terraform plan` を通る

---

### v6.8.0 — Rune エコシステム補完

**テーマ**: 既知の Rune 不足・ドキュメント不備を解消する。

**やること**
- `duckdb` rune: S3 統合クエリの確認
- `http` rune: `Http.serve<T>` の実装確認と補完
- `db` rune: `with_transaction` / `paginate` の確認
- 各 rune のサイトドキュメント不備を修正

**完了条件**
- 主要 rune（aws / duckdb / db / http / auth）の動作が統合テストで確認済み

---

### v6.9.0 — OSS 公開準備

**テーマ**: GitHub Public 化に向けた最終準備。

**やること**
- `CONTRIBUTING.md` の作成
- GitHub Actions CI 整備（`cargo test` → `fav check` → lint）
- `LICENSE`（MIT）確認・配置
- `CHANGELOG.md` 初版（v4.0.0 〜 v6.9.0 サマリー）
- GitHub リポジトリを Public に変更
- 発表準備（ブログ下書き・connpass LT 登録）

**完了条件**
- GitHub Public リポジトリとして公開されている
- CI が main ブランチで green になっている

> **注**: v6.9.0 は目安。v6.10.0 以降に延びても構わない。
> 「OSS 公開の準備が整った」タイミングで切る。

---

## v7.0.0 — Schema Authority

**テーマ**: 「外部データを型で守る」Favnir のコアユースケースを完成させる。

### 設計哲学

```
外部データ（CSV / DB / S3 / API）
    ↓  fav infer    → Favnir 型定義を自動生成
    ↓  schemas/*.yaml → 制約を付与
    ↓  fav check    → コンパイル時検査
    ↓  T.validate   → ランタイム検査
型安全なデータパイプライン
```

### エフェクト細分化（v7.0 で確定）

DB エフェクトを 3 段階に分ける：

```favnir
fn get_user(id: Int) -> Option<User>   !DbRead   // SELECT のみ
fn archive_user(id: Int) -> Unit       !DbWrite  // INSERT/UPDATE/DELETE
fn drop_table(name: String) -> Unit    !DbAdmin  // DDL（CREATE/DROP/ALTER）
```

`fav explain` で自動データリネージが得られる：

```
Effects: !DbRead(users, orders), !DbWrite(audit_log), !Io
→ 「users と orders を読んで audit_log に書く」が静的に保証される
```

### やること

- `!DbRead` / `!DbWrite` / `!DbAdmin` エフェクトを checker.rs + BUILTIN_EFFECTS に追加
- `runes/db/` の各関数にエフェクト宣言を付与・整理
- `fav infer` 強化: 生成型に schemas/*.yaml の制約ヒントを自動付与
- `T.validate` の Favnir 実装（Rust 側から `compiler.fav` 側に委譲）
- `fav build --schema` 強化: 外部 DB との DDL 差分検出
- サイトドキュメント: Schema Authority の全体像を示す「データパイプラインを型で守る」ガイド

### 完了条件

- CSV → `fav infer` → 型定義 → `T.validate` → 型安全な読み込みが
  一貫したワークフローとして機能する
- `schemas/*.yaml` の制約が compile-time と runtime の両方で機能する
- `!DbRead` / `!DbWrite` / `!DbAdmin` が型チェッカーで追跡される
- 既存テストがすべて通る

---

## v7.1.0 — fav explain 強化（データリネージ）

**テーマ**: エフェクトシグネチャを静的解析してデータの流れを可視化する。
数百万円のデータカタログと同等の情報をコードから自動生成する。

```bash
fav explain --lineage pipeline.fav
```

出力例：
```
Sources:
  !DbRead   → users (PostgreSQL), orders (PostgreSQL)
  !Io       → /data/config.csv

Sinks:
  !DbWrite  → audit_log (PostgreSQL)
  !AWS(S3)  → s3://my-bucket/reports/

Transformations:
  users ──→ filter(active) ──→ join(orders) ──→ audit_log
```

---

## v7.2.0 — SQL Rune（型安全クエリビルダ）

**テーマ**: dbt の代替。型安全なクエリを Favnir で記述できるようにする。

```favnir
import rune "sql"

bind result <- Sql.from<User>()
  |> Sql.where(|u| u.active == true)
  |> Sql.join<Order>(|u, o| u.id == o.user_id)
  |> Sql.select(|u, o| { user: u.name  total: o.amount })
  |> Sql.limit(100)
  |> Sql.run  // !DbRead
```

- クエリビルダを Favnir で実装（`Db.query_raw` の上に構築）
- 型パラメータでスキーマ整合性をコンパイル時に検証
- `!DbRead` / `!DbWrite` エフェクト自動付与

---

## v7.3.0 — Rune エコシステム拡充

**テーマ**: VM primitive = 薄い接続層、Favnir = 意味のある操作、のパターンで拡張。

| Rune | VM primitive（Rust） | Favnir 層 |
|------|---------------------|----------|
| `queue` | `Queue.send_raw` / `Queue.recv_raw` | バッチ送信・dead letter・ack 管理 |
| `cache` | `Cache.get_raw` / `Cache.set_raw` | TTL 管理・invalidation パターン |
| `fs` | `IO.read_file_raw`（既存）| ディレクトリ walk・glob・watch |
| `slack` | `Http.send_raw`（既存）| Slack 通知 Rune |
| `email` | `Smtp.send_raw` | メール送信 |

---

## 長期ビジョン（v8 以降）

- **Orchestration Rune**: `seq` をそのまま DAG として実行（Airflow/Prefect 代替）
- **SAP / Salesforce Rune**: レガシー統合ピッチの完成
- **エンタープライズ**: Veltra ベータ、コンサル向けサポート
- **コミュニティ**: Discord・外部 Rune 開発者の受け入れ

---

## 全体スケジュール概観

| バージョン | テーマ | フェーズ |
|-----------|--------|---------|
| v6.3.0 | Self-host stage/seq | セルフホスト仕上げ |
| v6.4.0 | Playground 改善 | 開発体験 |
| v6.5.0 | サイトドキュメント補完 | エコシステム整備 |
| v6.6.0 | T.validate 完成 | データ品質 |
| v6.7.0 | fav deploy ECS/Fargate + E2E デモ | インフラ |
| v6.8.0 | Rune エコシステム補完 | エコシステム整備 |
| v6.9.0〜 | OSS 公開準備（v6.10 以降に延びても可） | 公開 |
| **v7.0.0** | **Schema Authority** | **コアユースケース完成** |
| v7.1.0 | fav explain --lineage（データリネージ） | データ品質 |
| v7.2.0 | SQL Rune（型安全クエリビルダ） | dbt 代替 |
| v7.3.0 | Rune エコシステム拡充 | エコシステム整備 |

---

## 設計原則

**v6.x のバージョン数を固定しない**
v7.0.0 は「Schema Authority が完成した」タイミングで切る。
v6.x の数が増えることを恐れない。各バージョンの完了条件を守ることが優先。

**セルフホストの一貫性を保つ**
新機能を Rust 側に追加したら `compiler.fav` への反映を怠らない。
Bootstrap テストを常に通す。

**ドキュメントは実装と同じバージョンで完成させる**
v6.5.0 でいったん補完した後は、各バージョンで必ず docs を含める。
