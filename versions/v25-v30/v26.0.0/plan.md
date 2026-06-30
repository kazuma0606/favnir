# v26.0.0 実装プラン — Rune Foundation マイルストーン宣言

## 実装方針

v26.0.0 はマイルストーン宣言バージョン。
v25.1〜v25.9 の成果をまとめ、宣言ドキュメント・デモファイル・テストを追加する。
新規の Rust コア機能追加はなし。

---

## 実装ステップ

### Step 0: 事前確認

```bash
grep 'version = ' fav/Cargo.toml        # "25.9.0" であること
cargo test --bin fav 2>&1 | tail -3     # 2035 件 PASS であること
grep 'version_is_' fav/src/driver.rs | grep v259000  # version_is テストが存在するか確認
```

### Step 1: `fav/Cargo.toml` bump（25.9.0 → 26.0.0）

```toml
version = "26.0.0"
```

### Step 2: `MILESTONE.md` 更新

既存の `MILESTONE.md`（v25.0.0 で作成）に「Rune Foundation」セクションを追記:

```markdown
## v26.0.0 — Rune Foundation（2026-XX-XX）

コア 8 Rune（postgres / s3 / redis / mysql / mongodb / dynamodb / kafka / elasticsearch）が
「動く Rune の 5 条件」をすべてクリアした。

「Favnir で書いたパイプラインが実際の本番データを動かせる」状態を達成。

...
```

### Step 3: `examples/` デモファイル作成（3 件）

**`examples/postgres_etl.fav`**:
```favnir
import runes/postgres

type User = { id: Int, name: String, active: Bool }
type Summary = { user_id: Int, total: Int }

stage LoadUsers: Unit -> List<User> !Db = |_| {
  bind conn <- Postgres.connect(config.postgres)
  Postgres.query[User](conn, "SELECT id, name, active FROM users WHERE active = $1", [true])
}

stage Aggregate: List<User> -> List<Summary> = |users| {
  Result.ok(List.map(users, |u| Summary { user_id: u.id, total: 1 }))
}

stage SaveSummaries: List<Summary> -> Unit !Db = |summaries| {
  bind conn <- Postgres.connect(config.postgres)
  Postgres.execute_many(conn, "INSERT INTO summaries (user_id, total) VALUES ($1, $2)", summaries)
}

LoadUsers |> Aggregate |> SaveSummaries
```

**`examples/s3_csv_to_parquet.fav`**:
```favnir
import runes/s3
import runes/csv
import runes/parquet

type Row = { id: Int, value: String }

stage DownloadCsv: String -> List<Row> !Io = |key| {
  bind bytes <- S3.get_object("my-bucket", key)
  Csv.decode[Row](bytes)
}

stage UploadParquet: List<Row> -> Unit !Io = |rows| {
  bind bytes <- Parquet.encode(rows)
  S3.put_object("my-bucket", "output/result.parquet", bytes)
}

DownloadCsv("input/data.csv") |> UploadParquet
```

**`examples/full_etl.fav`**（`"postgres"` を含む必須）:
```favnir
import runes/postgres
import runes/s3
import runes/kafka

type Order = { id: Int, amount: Float, status: String }
type Summary = { total_orders: Int, total_amount: Float }

stage LoadOrders: Unit -> List<Order> !Db = |_| {
  bind conn <- Postgres.connect(config.postgres)
  Postgres.query[Order](conn, "SELECT id, amount, status FROM orders WHERE status = $1", ["completed"])
}

stage Summarize: List<Order> -> Summary = |orders| {
  let total = List.length(orders)
  let amount = List.fold(orders, 0.0, |acc, o| acc + o.amount)
  Result.ok(Summary { total_orders: total, total_amount: amount })
}

stage SaveToS3: Summary -> Unit !Io = |summary| {
  bind json <- Json.encode(summary)
  S3.put_object("my-bucket", "etl/summary.json", Bytes.from_string(json))
}

stage NotifyKafka: Summary -> Unit !Io = |summary| {
  bind msg <- Json.encode(summary)
  Kafka.produce("etl-events", "summary", msg)
}

LoadOrders |> Summarize |> SaveToS3 |> NotifyKafka
```

### Step 4: `README.md` 更新

"v26.0" または "Rune Foundation" を含む記述を追記:
```markdown
## v26.0 — Rune Foundation

コア 8 Rune が実際のデータベース・ストレージ・メッセージングサービスに接続し、
本番データを処理できる状態（Rune Foundation）を達成しました。
```

### Step 5: `site/content/docs/rune-foundation.mdx` 作成

"Rune Foundation" を含む MDX ドキュメント:
- 8 Rune の 5 条件クリア状況表
- `examples/full_etl.fav` のコード例
- Docker Compose での実行手順
- vm.fav Phase 6（CallNamed）の達成サマリー

### Step 6: `versions/roadmap/roadmap-v25.1-v26.0.md` 更新

- 各バージョン（v25.1〜v25.9）に「COMPLETE」ステータスを追記
- v26.0.0 に「宣言済み（2026-XX-XX）」を追記

### Step 7: `fav/src/driver.rs` — v260000_tests 追加

`v259000_tests` の直後に `v260000_tests` モジュールを追加（5 件）:

```rust
#[cfg(test)]
mod v260000_tests {
    #[test]
    fn milestone_md_has_rune_foundation() {
        let content = include_str!("../../MILESTONE.md");
        assert!(content.contains("Rune Foundation"), ...);
    }
    #[test]
    fn readme_mentions_v26_0() {
        let content = include_str!("../../README.md");
        assert!(content.contains("v26.0"), ...);
    }
    #[test]
    fn site_rune_foundation_page_exists() {
        let content = include_str!("../../site/content/docs/rune-foundation.mdx");
        assert!(content.contains("Rune Foundation"), ...);
    }
    #[test]
    fn examples_full_etl_exists() {
        let content = include_str!("../../examples/full_etl.fav");
        assert!(content.contains("postgres"), ...);
    }
    #[test]
    fn changelog_has_v26_0_0() {
        let content = include_str!("../../CHANGELOG.md");
        assert!(content.contains("[v26.0.0]"), ...);
    }
}
```

### Step 8: `CHANGELOG.md` 更新

先頭に `[v26.0.0]` エントリを追加:
```markdown
## [v26.0.0] — Rune Foundation マイルストーン宣言

### Milestone
- Rune Foundation: コア 8 Rune（postgres / s3 / redis / mysql / mongodb / dynamodb / kafka / elasticsearch）が 5 条件クリア
- `examples/full_etl.fav` — postgres → 集計 → s3 → kafka 通知デモ
- vm.fav Phase 6（CallNamed opcode）完了: multi-function プログラムを vm.fav で実行可能
- 「Favnir で書いたパイプラインが実際の本番データを動かせる」状態を達成
```

### Step 9: `benchmarks/v26.0.0.json` 作成

```json
{"version":"26.0.0","test_count":2040,"timestamp":"2026-06-26"}
```

### Step 10: テスト確認

```bash
cargo test v260000 --bin fav    # 5/5 PASS
cargo test --bin fav             # 2040 件 PASS、リグレッションなし
```

---

## ファイル変更一覧

| ファイル | 操作 |
|---|---|
| `fav/Cargo.toml` | version bump 25.9.0 → 26.0.0 |
| `MILESTONE.md` | 「Rune Foundation」セクション追記 |
| `examples/postgres_etl.fav` | 新規作成 |
| `examples/s3_csv_to_parquet.fav` | 新規作成 |
| `examples/full_etl.fav` | 新規作成 |
| `README.md` | `"v26.0"` / `"Rune Foundation"` 追記 |
| `site/content/docs/rune-foundation.mdx` | 新規作成 |
| `versions/roadmap/roadmap-v25.1-v26.0.md` | v25.1〜v26.0 ステータス更新 |
| `fav/src/driver.rs` | `v260000_tests`（5 件）追加 |
| `CHANGELOG.md` | `[v26.0.0]` エントリ先頭に追加 |
| `benchmarks/v26.0.0.json` | 新規作成 |

---

## 注意事項

- `examples/` ディレクトリは v25.1〜v25.8 で作成済み（postgres_etl.fav 等 8 ファイル存在）。`full_etl.fav` のみ欠けているため T5 で新規作成する。`ls examples/full_etl.fav` で事前確認。
- `MILESTONE.md` は v25.0.0 で作成済み。**追記**（上書き不可）。
- v259000_tests に `version_is_` テストは存在しない（T0 で確認）。目標テスト数: 2035 + 5 = **2040 件**（固定）。
- `include_str!` のパスは `fav/src/driver.rs` から見た相対パス:
  - `MILESTONE.md` → `"../../MILESTONE.md"`
  - `README.md` → `"../../README.md"`
  - `examples/full_etl.fav` → `"../../examples/full_etl.fav"`
  - `site/content/docs/rune-foundation.mdx` → `"../../site/content/docs/rune-foundation.mdx"`
  - `CHANGELOG.md` → `"../../CHANGELOG.md"`
- T9（v260000_tests 追加）は T5（full_etl.fav 作成）より後に実施すること（include_str! コンパイルエラー防止）。
