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

**実装済み（v7.3.0 COMPLETE）**: fs / slack / queue / cache（email は v7.4.0 以降）

---

## v7.4.0 — stdlib 高レベル層（Favnir 化）

**テーマ**: VM primitive の上に乗る高レベルな stdlib 操作を Favnir で書き直す。

### 方針

VM に残すもの（thin primitive）:
- `List.map` / `List.filter` / `List.fold_left` 等の基本操作
- `String.concat` / `IO.read_file_raw` 等の I/O primitive

Favnir 化するもの（`runes/stdlib/` へ移動）:
- `List.group_by` / `List.zip` / `List.zip_with` / `List.sort_by` / `List.chunk`
- `String.pad_left` / `String.pad_right` / `String.split`（区切り文字指定）
- `Map.merge` / `Map.from_list` / `Map.map_values`

### やること

- `runes/stdlib/list.fav` — 高レベルリスト操作（group_by / zip / chunk / sort_by）
- `runes/stdlib/string.fav` — 高レベル文字列操作（split / pad / replace）
- `runes/stdlib/map.fav` — 高レベル Map 操作（merge / from_list / map_values）
- 統合テスト + `site/content/docs/stdlib/` ドキュメント

### 完了条件

- `runes/stdlib/` の各ファイルが `fav check` を通る
- 既存テストがすべて通る
- email Rune（`Smtp.send_raw` 追加）も本バージョンで対応

---

## v7.5.0 — Rune 読み込みのセルフホスト化

**テーマ**: `fav rune add` / import 解決を Rust から Favnir に移す。

### 依存

- `fs` Rune（✓ v7.3.0 実装済み）
- ミニ TOML パーサー（新規実装）

### やること

- `runes/toml/toml.fav` — `rune.toml` を読んで `name` / `version` / `entry` / `effects` を取り出す簡易パーサー
  - フル TOML 実装は不要。`key = "value"` / `[section]` / 配列 `["a", "b"]` のみ対応
- `runes/rune_loader/loader.fav` — `rune_modules/` → `runes/` → `~/.fav/registry/` の順に解決
- Rust 側の `resolve_rune_path` を Favnir 製に差し替え

### 完了条件

- `use sql` が Favnir 製 loader 経由で解決できる
- バージョン制約（`^1.0`）の基本的な比較が動く
- 既存の rune install / resolve テストがすべて通る

---

## v7.6.0 — CLI の部分セルフホスト化

**テーマ**: `fav check` / `fav explain` / `fav rune` コマンドを Favnir 製 CLI に置き換える。

### 前提

- `IO.argv()` ✓
- コンパイルパイプライン（compiler.fav）✓
- Rune loader（✓ v7.5.0）
- `fav run` だけは VM 呼び出しのため Rust ラッパーが残る

### やること

- `fav/self/cli.fav` — サブコマンド dispatch（check / explain / rune / version / help）
- `fav check <file>` → compiler.fav の型チェックパスを呼ぶ
- `fav explain --lineage <file>` → lineage_analysis ロジックを Favnir で再実装
- `fav rune add/list/info` → rune_loader.fav を使用
- Rust の `main.rs` は「VM を起動して cli.fav を実行する」だけの薄いラッパーに

### 完了条件

- `fav check` / `fav rune list` が Favnir 製 CLI 経由で動作する
- 既存の CLI 統合テストが通る

---

## v7.7.0 — checker.fav 基本機能パリティ

**テーマ**: self/checker.fav を checker.rs の基本機能と同等にする。

### 現状の checker.fav ギャップ

現在の checker.fav（513行）は Bootstrap 検証用の簡略版で、実際の `fav check` には使われていない。

### やること

- **エフェクト追跡** — `!DbRead` / `!IO` / `!Cache` 等のアノテーション検証
- **全 builtin 名前空間の型シグネチャ登録** — `List.*` / `String.*` / `Cache.*` / `Queue.*` 等
- **エラーコード（E0xxx）** — 現在は単純な String エラーのみ
- **match 網羅性チェック（基本）** — `None`/`Some` 両腕の有無確認
- `fav check compiler.fav` を checker.fav 経由で実行し、Rust版と同じエラーを検出できることを確認

### 完了条件

- checker.fav がエフェクト違反・基本型ミスマッチを Rust版と同じく検出できる
- bootstrap テストが引き続き通る

---

## v7.8.0 — checker.fav ジェネリクス対応

**テーマ**: 型変数と parameterized types を checker.fav で扱えるようにする。

### やること

- **型変数の追跡** — `List<A>` の `A` を変数として管理
- **組み込みジェネリクス型の instantiation** — `List<Int>` / `Option<String>` / `Result<A,B>`
- **ユーザー定義ジェネリクス型** — `type Pair<A, B> = { fst: A, snd: B }` のチェック
- **基本的な単一化（unification）** — `A = String` のような単純な型変数の代入

> **注意**: Favnir 自体が `bind inside closure 不可` 制約を持つため、substitution map の実装には工夫が必要。`List<KVPair>` association list で代用する見通し。

### 完了条件

- `List.map(xs, |x| x + 1)` の型を `List<Int>` と正しく推論できる
- ユーザー定義ジェネリクス型の型ミスマッチを検出できる

---

## v7.9.0 — checker.fav HM 型推論（基礎）✓ COMPLETE

**テーマ**: Hindley-Milner 型推論の基礎部品を Favnir で実装する。

### 実装済み内容

- **`occurs_in`** — 型変数が型文字列に出現するか確認（E0006 無限型防止）
- **`unify_deep`** — `"List<A>"` vs `"List<Int>"` のネスト型を outer 比較 + inner 再帰で単一化
- **`InfState` / `InfResult`** — 推論状態レコード（subst + counter）
- **`fresh_var`** — `"t0"`, `"t1"`, ... 新鮮型変数生成
- **`infer_hm`** — HM 推論パス（ELit / EVar / EBind / EIf + infer_expr フォールバック）
- **`check_fn_def` 更新** — `infer_expr` → `infer_hm` に切り替え
- Favnir 内テスト 10 件 + driver.rs 統合テスト 3 件

### 残った制約（v8.0.0 へ持ち越し）

- Let 多相（型スキーム generalization）未実装
- 多文字型変数（`"T1"`, `"Elem"` 等）未対応（1 文字大文字のみ）
- ネスト 2 段以上のパラメータ化型の完全単一化未対応
- `checker.fav` はまだ `fav check` の実処理に接続されていない

---

## v8.0.0 — checker.fav 完全統合（Let 多相 + fav check 差し替え）

**テーマ**: checker.fav を checker.rs の完全な代替として `fav check` コマンドに組み込む。
v8.0.0 の完了をもって「型チェッカーのセルフホスト完成」とする。

### 背景

現状の `fav check foo.fav` は Rust 製 checker.rs を使用している。
checker.fav は v7.7.0〜v7.9.0 で機能的に同等に近づいたが、
Let 多相がなく本番パイプラインにも接続されていない。

```
現状:  fav check foo.fav → checker.rs (Rust)
v8.0: fav check foo.fav → checker.fav (Favnir、Rust VM の上で動く)
```

### やること

**A. Let 多相（型スキーム generalization）**

```favnir
// 現状: is_type_var("A") の 1 文字大文字のみ
// v8.0: 多文字型変数 "T1" / "Elem" / "Key" 等を追加

fn is_type_var_extended(s: String) -> Bool { ... }
```

型スキームを表現するレコードを追加：

```favnir
type TypeScheme = {
    vars:  List<String>   // 全称量化された型変数
    body:  String         // 型本体（"List<T>" 等）
}
```

`check_fn_def` で関数ごとに型スキームを構築し、
呼び出し側で `instantiate(scheme, counter)` して fresh_var を割り当てる。

**B. 多文字型変数 + ネスト単一化の拡張**

- `is_type_var` を `is_type_var_extended` に置き換え（`"T1"`, `"Elem"` 等を許容）
- `unify_deep` で 2 段以上のネスト型（`"Map<String, List<Int>>"` 等）を再帰処理

**C. checker.fav を `fav check` に接続**

`Compiler.check_raw` の実装を checker.fav ベースに切り替える方式を検討：

```
方式 A: fav check foo.fav
          → main.rs が checker.fav を読んで VM 上で実行
          → checker.fav が foo.fav の AST を受け取り型チェック
方式 B: checker.rs の内部で checker.fav を評価して結果を使う
```

現実的には方式 A（VM 上で checker.fav を実行）で進める。
`Compiler.check_raw` を「checker.fav をロードして実行する」実装に差し替え。

**D. エラー形式の統一**

現在 checker.rs は `CheckError { message, span }` を返すが、
checker.fav は `"E0xxx: ..."` 形式の文字列を返す。
両者のエラー形式を統一する（または変換レイヤーを追加）。

**E. 既存テストの checker.fav 互換確認**

checker.rs のテストケースを checker.fav で再現し、同じ結果になることを確認。

### 完了条件

- `fav check foo.fav` が checker.fav（Favnir 実装）経由で動作する
- `fav check fav/self/checker.fav` が checker.fav 自身でチェック通過（完全なブートストラップ）
- let 多相で `fn id<A>(x: A) -> A` が `id(1)` にも `id("a")` にも使える
- 既存テストが全件通る

---

## 長期ビジョン（v8.1 以降）

- **Orchestration Rune**: `seq` をそのまま DAG として実行（Airflow/Prefect 代替）
- **SAP / Salesforce Rune**: レガシー統合ピッチの完成
- **エンタープライズ**: Veltra ベータ、コンサル向けサポート
- **コミュニティ**: Discord・外部 Rune 開発者の受け入れ
- **compiler.rs の完全セルフホスト**: `fav run` も Favnir 製コンパイラ経由に

---

## 全体スケジュール概観

| バージョン | テーマ | フェーズ |
|-----------|--------|---------|
| v6.3.0 | Self-host stage/seq | セルフホスト仕上げ |
| v6.4.0 | Playground 改善 | 開発体験 |
| v6.5.0 | サイトドキュメント補完 | エコシステム整備 |
| v6.6.0 | T.validate 完成 | データ品質 |
| v6.8.0 | Rune エコシステム補完 | エコシステム整備 |
| v6.9.0〜 | OSS 公開準備（v6.10 以降に延びても可） | 公開 |
| **v7.0.0** | **Schema Authority** | **コアユースケース完成** |
| v7.1.0 | fav explain --lineage（データリネージ） | データ品質 |
| v7.2.0 | SQL Rune（型安全クエリビルダ） | dbt 代替 |
| v7.3.0 | Rune エコシステム拡充（fs/slack/queue/cache） | エコシステム整備 |
| v7.4.0 | stdlib 高レベル層 Favnir 化 + email Rune | セルフホスト準備 |
| v7.5.0 | Rune 読み込みセルフホスト（TOML パーサー + loader） | セルフホスト |
| v7.6.0 | CLI 部分セルフホスト（check / explain / rune） | セルフホスト |
| v7.7.0 ✓ | checker.fav — エフェクト追跡 + builtin シグネチャ + エラーコード | セルフホスト |
| v7.8.0 ✓ | checker.fav — ジェネリクス + 基本単一化 | セルフホスト |
| v7.9.0 ✓ | checker.fav — occurs_in / unify_deep / infer_hm（HM 基礎） | セルフホスト |
| **v8.0.0** | **checker.fav 完全統合 — Let 多相 + fav check 差し替え** | **型チェッカー完全セルフホスト** |

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
