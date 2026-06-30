# Roadmap v25.1.0 〜 v26.0.0 — Rune Foundation

Date: 2026-06-24

## 目標

v25.0.0「Practical Self-Hosting」をもって、Favnir は言語として完成した。
コンパイラ・型チェッカー・CLI・VM 仕様がすべて Favnir で記述されている。

しかし「言語として完成した」と「実業務で使える」は別の話だ。
50 を超える Rune カタログの大半はインターフェース定義（スタブ）に留まっており、
「postgres に繋ごうとしたら動かなかった」という状況が現実に起きている。

このフェーズでは、**コア 8 Rune を本当に動く状態に実質化する**。
データエンジニアが最初に手を伸ばすサービス群（postgres / s3 / redis / mysql /
mongodb / dynamodb / kafka / elasticsearch）を完全実装し、
「Favnir で書いたパイプラインが実際の本番データを動かせる」を達成する。

vm.fav Phase 6（CallFn オペコード）も並行して完成させ、
「Favnir で書いた VM が Favnir を完全に実行する」を実現する。

> **Rune Foundation の定義（本プロジェクト固有）**
> 「コア 8 Rune がすべて 5 条件（connect / read / write / error / test）を満たし、
>  `fav run examples/full_etl.fav`（postgres → 集計 → s3 → kafka 通知）が動く」状態を指す。

**完了条件（最終テスト）:**

```bash
# 1. 全 Rust テストが通る
cargo test

# 2. postgres ETL が実際に動く
fav run examples/postgres_etl.fav

# 3. s3 → parquet 変換が動く
fav run examples/s3_csv_to_parquet.fav

# 4. フル ETL デモが動く（postgres → 集計 → s3 → kafka 通知）
fav run examples/full_etl.fav

# 5. vm.fav Phase 6: compiler.fav が vm.fav 経由で動く
cargo test -- --ignored bootstrap
fav run --vm=self/vm.fav self/compiler.fav -- tests/bootstrap/hello.fav
```

---

## 設計決定事項

| 項目 | 決定 |
|---|---|
| 「動く Rune」の 5 条件 | connect / read / write / error / test（詳細は下記） |
| モックテスト最小件数 | postgres: 5 / s3: 5 / redis: 5 / mysql: 4 / mongodb: 5 / dynamodb: 5 / kafka: 5 / es: 5 |
| s3 / dynamodb のローカルテスト環境 | LocalStack（Docker）を使用。本番 AWS は不要 |
| kafka のローカルテスト環境 | Redpanda（Docker）を使用。Confluent は不要 |
| postgres / mysql の DbConn interface | `interface DbConn { connect / query / execute / transaction }` を共通化。DB 切り替えを注入パターンで実現 |
| vm.fav Phase 6 の対象 | `CallFn(fn_idx, argc)` オペコードのみ。クロージャのキャプチャは次フェーズ |
| Cargo.toml バージョン進め方 | 各バージョン実装開始時に bump（v25.1.0 / v25.2.0 / ... / v26.0.0） |
| 破壊的変更 | なし（STABILITY.md v1.x ポリシーに従う） |

---

## 「動く Rune」の 5 条件

スタブから実質化への基準。すべての Rune はこの 5 条件で評価する:

```
1. connect  — 接続・認証が実際に確立できる（ローカル Docker で検証）
2. read     — データを Favnir の型として読み込める（型変換・デシリアライズ）
3. write    — Favnir のデータをサービスに書き込める
4. error    — 失敗時に型付きエラー（Result.err）が返る
5. test     — cargo test でモックを使った自動テストが通る（最小 N 件）
```

---

## バージョン計画

### v25.1 — postgres Rune 実質化 **[COMPLETE]**

**テーマ**: コア 8 Rune の筆頭。データエンジニアリングで最も使われる DB を「本当に動く」状態にする。

**依存関係**: なし（v25.0.0 で Rune スタブ + `runes/postgres/` ディレクトリが存在）

```favnir
import runes/postgres

stage LoadUsers: Unit -> List<User> !Db = |_| {
  bind conn <- Postgres.connect(config.postgres)
  bind rows <- Postgres.query[User](conn, "SELECT * FROM users WHERE active = $1", [true])
  Result.ok(rows)
}

stage SaveResult: List<Summary> -> Unit !Db = |summaries| {
  bind conn <- Postgres.connect(config.postgres)
  bind _ <- Postgres.execute_many(conn, "INSERT INTO summaries VALUES ($1, $2)", summaries)
  Result.ok(unit)
}
```

実装する関数:

| 関数 | 内容 |
|---|---|
| `Postgres.connect(config)` | 接続確立（SSL 対応、接続タイムアウト設定） |
| `Postgres.query[T](conn, sql, params)` | 型付きクエリ（行 → T にデシリアライズ） |
| `Postgres.execute(conn, sql, params)` | 更新・削除・DDL（影響行数を返す） |
| `Postgres.execute_many(conn, sql, rows)` | バッチ挿入（COPY プロトコル利用） |
| `Postgres.transaction(conn, fn)` | トランザクション（エラー時ロールバック自動） |
| `Postgres.Pool.create(config)` | コネクションプール（v20.8 の機能を Rune に昇格） |

`cargo test postgres` でモックテスト 5 件以上 PASS、
`fav run examples/postgres_etl.fav`（ローカル Docker + Postgres）が動くことを確認する。

---

### v25.2 — s3 Rune 実質化 **[COMPLETE]**

**テーマ**: データ基盤の起点。ほぼすべての ETL が S3 を経由する。LocalStack で完全検証する。

**依存関係**: v25.1 完了後（examples/ の構造確認のため）

```favnir
import runes/s3

stage DownloadCsv: String -> List<Row> !Io = |key| {
  bind bytes <- S3.get_object(config.s3.bucket, key)
  bind rows  <- Csv.decode[Row](bytes)
  Result.ok(rows)
}

stage UploadParquet: List<Row> -> Unit !Io = |rows| {
  bind bytes <- Parquet.encode(rows)
  bind _     <- S3.put_object(config.s3.bucket, "output/result.parquet", bytes)
  Result.ok(unit)
}
```

実装する関数:

| 関数 | 内容 |
|---|---|
| `S3.get_object(bucket, key)` | オブジェクト取得（Bytes 返却） |
| `S3.put_object(bucket, key, bytes)` | オブジェクト書き込み（Content-Type 自動推定） |
| `S3.list_objects(bucket, prefix)` | プレフィックス一覧（ページネーション対応） |
| `S3.delete_object(bucket, key)` | 削除 |
| `S3.presign_url(bucket, key, ttl)` | 署名付き URL 生成（GET / PUT） |
| `S3.stream_get(bucket, key)` | ストリーミング取得（大容量・チャンク分割） |

LocalStack（`docker compose up localstack -d`）で全関数が動作し、
`fav run examples/s3_csv_to_parquet.fav` が動くことを確認する。

---

### v25.3 — redis Rune 実質化 **[COMPLETE]**

**テーマ**: キャッシュ・セッション・レート制限・Pub/Sub のハブ。postgres と並行して実装できる。

**依存関係**: v25.1 と並行可能

実装する関数:

| 関数 | 内容 |
|---|---|
| `Redis.get[T](conn, key)` | 型付き GET（JSON デシリアライズ） |
| `Redis.set(conn, key, value, ttl)` | SET with TTL（`Option<Duration>`） |
| `Redis.del(conn, key)` | DELETE |
| `Redis.incr(conn, key)` | INCR（カウンタ・レート制限） |
| `Redis.lpush(conn, key, value)` | リスト先頭追加 |
| `Redis.rpop(conn, key)` | リスト末尾取得（キュー操作） |
| `Redis.publish(conn, channel, msg)` | Pub/Sub 送信 |
| `Redis.subscribe(conn, channel, fn)` | Pub/Sub 受信ループ |

MockRedis を使った `cargo test redis` で 5 件以上 PASS。

---

### v25.4 — mysql Rune 実質化 **[COMPLETE]**

**テーマ**: 企業 DB の定番。`interface DbConn` を通じて postgres との API を統一する。

**依存関係**: v25.1 完了（`interface DbConn` 定義が完成しているため）

```favnir
// interface DbConn を mysql で impl することで、
// DB 依存を注入パターンで切り替え可能にする
import runes/mysql

stage LoadOrders: Unit -> List<Order> !Db = |_| {
  bind conn <- MySQL.connect(config.mysql)
  bind rows <- MySQL.query[Order](conn, "SELECT * FROM orders WHERE status = ?", ["pending"])
  Result.ok(rows)
}
```

postgres と同一の API シグネチャで 4 関数（`connect / query[T] / execute / transaction`）を実装。
`cargo test mysql` でモックテスト 4 件以上 PASS。
postgres・mysql 両方に同じコードが動くことを integration テストで確認する。

---

### v25.5 — mongodb Rune 実質化 **[COMPLETE]**

**テーマ**: ドキュメント系 NoSQL の代表。JSON / BSON との親和性から、
イベントログ・ユーザープロファイル・半構造データに多用される。

**依存関係**: なし（postgres / mysql と独立した API 体系）

実装する関数:

| 関数 | 内容 |
|---|---|
| `Mongo.find[T](coll, filter)` | 型付き find（BSON → T デシリアライズ） |
| `Mongo.find_one[T](coll, filter)` | 1 件取得（`Option<T>` 返却） |
| `Mongo.insert_one(coll, doc)` | ドキュメント挿入（挿入 ID を返す） |
| `Mongo.insert_many(coll, docs)` | バッチ挿入 |
| `Mongo.update_one(coll, filter, update)` | 更新（`$set` / `$inc` 等演算子対応） |
| `Mongo.delete_one(coll, filter)` | 削除 |
| `Mongo.aggregate[T](coll, pipeline)` | 集計パイプライン（`$match / $group / $sort`）|

`cargo test mongodb` でモックテスト 5 件以上 PASS。

---

### v25.6 — dynamodb Rune 実質化 **[COMPLETE]**

**テーマ**: AWS ユーザーの KV / NoSQL の中心。s3 で整備した LocalStack 基盤を共有する。

**依存関係**: v25.2（s3 Rune）完了後推奨（LocalStack の AWS 認証設定を共有）

実装する関数:

| 関数 | 内容 |
|---|---|
| `DynamoDB.get_item[T](table, key)` | 型付き GetItem（PK + SK） |
| `DynamoDB.put_item(table, item)` | PutItem（conditional expression 対応） |
| `DynamoDB.delete_item(table, key)` | DeleteItem |
| `DynamoDB.query[T](table, condition)` | 型付き Query（GSI 対応） |
| `DynamoDB.scan[T](table, filter)` | Scan（全件、ページネーション対応） |
| `DynamoDB.batch_write(table, items)` | BatchWriteItem（最大 25 件） |
| `DynamoDB.transact_write(ops)` | TransactWriteItems（ACID トランザクション）|

LocalStack で全関数が動作。`cargo test dynamodb` でモックテスト 5 件以上 PASS。

---

### v25.7 — kafka Rune 実質化 **[COMPLETE]**

**テーマ**: ストリーミングパイプラインの中核。次フェーズ（Streaming Native）の入口として完成させる。

**依存関係**: なし（ストリーミング Rune の先行・後続の v26.x への橋渡し）

実装する関数:

| 関数 | 内容 |
|---|---|
| `Kafka.produce(topic, key, value)` | メッセージ送信（非同期、確認待ちオプション） |
| `Kafka.consume[T](topic, group_id, fn)` | 型付き consumer ループ（自動オフセット管理） |
| `Kafka.consume_batch[T](topic, group_id, size, fn)` | バッチ消費（最大 N 件をまとめて処理） |
| `Kafka.commit(consumer)` | 手動オフセットコミット |
| `Kafka.seek(consumer, partition, offset)` | オフセット指定（リプレイ用） |

Redpanda（Kafka 互換、Docker）でモックテスト 5 件以上 PASS。

---

### v25.8 — elasticsearch Rune 実質化 **[COMPLETE]**

**テーマ**: 全文検索・ログ分析・ベクトル検索（ES 8.x+）。
後のフェーズ（v29.1 の Rune Registry 検索基盤）でも再利用する。

**依存関係**: なし

実装する関数:

| 関数 | 内容 |
|---|---|
| `ES.index(index, doc)` | ドキュメントインデックス（ID 自動生成） |
| `ES.index_with_id(index, id, doc)` | ID 指定インデックス |
| `ES.search[T](index, query)` | 型付き検索（hits → `List<T>` に変換） |
| `ES.bulk(index, docs)` | バルクインデックス（高スループット） |
| `ES.delete(index, id)` | ドキュメント削除 |
| `ES.knn_search[T](index, vector, k)` | ベクトル近傍検索（ES 8.x+ / kNN） |
| `ES.create_index(index, mapping)` | インデックス作成（マッピング定義）|

`cargo test elasticsearch` でモックテスト 5 件以上 PASS。

---

### v25.9 — vm.fav Phase 6（CallNamed 実装）**[COMPLETE: 2026-06-26]**

**テーマ**: ユーザー定義関数のディスパッチを vm.fav 内で完結させる。
「Favnir で書いた VM が Favnir プログラム（関数呼び出しあり）を完全に実行できる」最後のピース。

**依存関係**: なし（Rune 実質化と独立。v25.1〜v25.8 と並行実施可能）

実装した opcode: `CallNamed(name_idx, argc)` (0x56、5 バイト)

> 注: ロードマップ初稿では `CallFn` と記載していたが、実装では `CallNamed` に確定。
> `name_idx` は定数プール内の関数名（`Constant::Name`）のインデックス。

完了確認:
- `CallNamed(Int, Int)` opcode が `fav/self/vm.fav` に実装済み（`v259000_tests` 7 件 PASS）
- `fav run --vm <path> --compile <src>` CLI モード追加
- `build_vm_program_json` / `run_via_vm` が `fav/src/driver.rs` に追加済み

---

## v26.0 — Rune Foundation マイルストーン宣言 **[宣言済み: 2026-06-26]**

**完了条件:**

| コンポーネント | 完了基準 |
|---|---|
| postgres Rune | 5 条件クリア + 5 件テスト + examples/postgres_etl.fav 動作 |
| s3 Rune | 5 条件クリア + 5 件テスト + examples/s3_csv_to_parquet.fav 動作 |
| redis Rune | 5 条件クリア + 5 件テスト |
| mysql Rune | 5 条件クリア + 4 件テスト（DbConn interface 統一確認） |
| mongodb Rune | 5 条件クリア + 5 件テスト |
| dynamodb Rune | 5 条件クリア + 5 件テスト + LocalStack 動作 |
| kafka Rune | 5 条件クリア + 5 件テスト |
| elasticsearch Rune | 5 条件クリア + 5 件テスト |
| vm.fav Phase 6（CallNamed, 0x56） | `v259000_tests` 7 件 PASS（v25.9.0 完了）|
| full_etl.fav デモ | postgres → 集計 → s3 → kafka 通知が動く |

**最終テスト（全件 PASS が完了条件）:**

```bash
# 1. 全 Rust テストが通る
cargo test

# 2. コア ETL デモが動く
docker compose up localstack redpanda postgres -d
fav run examples/postgres_etl.fav
fav run examples/s3_csv_to_parquet.fav

# 3. フル ETL デモが動く
fav run examples/full_etl.fav
# → postgres からデータを読み、集計して s3 に保存し、kafka に通知

# 4. vm.fav Phase 6 検証
cargo test -- --ignored bootstrap
fav run --vm=self/vm.fav self/compiler.fav -- tests/bootstrap/hello.fav
```

> 「`fav run examples/full_etl.fav` が実際の Docker 環境で動く」
> = Rune Foundation の完成を象徴するデモ

---

## 参考リンク

- マスタースケジュール: `versions/roadmap/roadmap-v25.1-v30.0.md`
- 前フェーズ: `versions/roadmap/roadmap-v24.1-v25.0.md`
- 次フェーズ: `versions/roadmap/roadmap-v26.1-v27.0.md`
- Rune 5 条件定義: `versions/roadmap/roadmap-v25.1-v30.0.md#動く-runeの定義`
