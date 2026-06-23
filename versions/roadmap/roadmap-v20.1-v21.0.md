# Roadmap v20.1.0 〜 v21.0.0 — Runtime Excellence

Date: 2026-06-18

## 目標

v20.0.0「Production Performance」で「本番で速い」を達成した。
このフェーズはその速さの上限を追求する——**VM が限界まで速い**。

「速くなった気がする」という印象で終わらないために、
最初の一手はベンチマーク基盤の整備（v20.1）である。
計測なしに最適化しない。数字を持った判断から始める。

**成功基準（項目別 SLO）:**

| ベンチマーク | v20.0.0 基準 | v21.0 目標 | 達成方法 |
|---|---|---|---|
| cold_start_precompiled | 18ms | **< 10ms** | NaN-boxing + 起動パス最適化 |
| csv_10gb_throughput | ~340 MB/s | **> 1 GB/s (3x)** | mmap + SIMD CSV パーサー |
| tight_loop_10m_iter | ~85ms | **< 30ms (2.8x)** | スーパー命令 |
| record_transform_1m | ~210ms | **< 80ms (2.6x)** | NaN-boxing |
| duckdb_query（集計） | VM 実行 | **DuckDB 委譲（10〜100x）** | pushdown |

> 注: cold_start は絶対値での改善（-8ms）が目標。割合（1.8x）が他項目より低いのは設計上の意図による。

---

## 設計決定事項

| 項目 | 決定 |
|---|---|
| ベンチマーク基準ファイル | `benchmarks/v20.0.0.json`（CI が生成・コミット） |
| `benchmarks/results.md` | `compare.fav --emit-md` が JSON から生成（手書き禁止） |
| スーパー命令の選定方針 | v20.1 実測ホットパスを分析してから top 10〜20 パターンを選定 |
| NaN-boxing フォールバック | `--legacy-value-repr` フラグで旧表現にフォールバック可能 |
| DuckDB pushdown の対象 | filter / group_by / sum / map（投影）/ count の5パターン（Phase 1） |
| mmap + SIMD CSV | `memmap2` + `arrow-csv`（v19.5 Arrow 基盤と組み合わせ） |
| io_uring 対象 OS | Linux 5.1+（Windows / macOS は epoll / kqueue に自動フォールバック） |
| Arena スコープ | 1 chunk（= 1000 行）= 1 arena。chunk 完了時に一括解放 |
| コネクションプール宣言 | `seq Pipeline [pool: Postgres.Pool]` — pipeline レベルで共有 |

---

## バージョン計画

### v20.1 — ベンチマーク基盤整備（Benchmark Infrastructure）

**テーマ**: 最初の一手。**何も最適化しない**。計測だけを整える。

#### 計測対象（ベンチマークスイート）

```
benchmarks/suite/
├── 01_cold_start.sh          # Lambda コールドスタート（--precompiled あり/なし）
├── 02_csv_10gb.fav           # 10GB CSV ストリーミング（スループット / ピークメモリ）
├── 03_tight_loop.fav         # 整数演算タイトループ（純粋 VM 速度）
├── 04_record_transform.fav   # レコード変換 100万行（アロケーション速度）
├── 05_compile_time.sh        # コンパイル時間（cold / incremental）
├── 06_duckdb_query.fav       # DuckDB クエリ（比較用）
├── 07_arrow_parquet.fav      # Arrow → Parquet 書き込み（I/O スループット）
└── 08_concurrent_stages.fav  # par [A, B] 並列 stage（スレッド効率）
```

#### CI 統合

```yaml
# .github/workflows/bench.yml（master push ごとに実行）
- name: Run benchmarks
  run: bash benchmarks/suite/run_all.sh --format json > benchmarks/latest.json

- name: Compare with baseline
  run: |
    fav run benchmarks/compare.fav \
      --baseline benchmarks/v20.0.0.json \
      --current  benchmarks/latest.json \
      --threshold 10  # 10% 以上の劣化で warning
```

#### v20.0.0 ベースライン確定（参考値 / 実測後に更新）

```
cold_start_full:         ~320ms
cold_start_precompiled:   ~18ms
csv_10gb_throughput:     ~340 MB/s
tight_loop_10m_iter:      ~85ms
record_transform_1m:     ~210ms
compile_cold:            ~2.4s
compile_incremental:     ~0.18s
arrow_parquet_write_1gb: ~3.2s
```

#### 完了成果物

| 成果物 | 目的 |
|---|---|
| `.github/workflows/bench.yml` | master push ごとに benchmarks/suite/ を実行 |
| `benchmarks/suite/run_all.sh` | 全スイートを JSON 形式で出力するラッパー |
| `benchmarks/suite/01_cold_start.sh` 〜 `08_concurrent_stages.fav` | 8 計測スクリプト |
| `benchmarks/compare.fav` | ベースライン比較スクリプト（劣化検知・`--emit-md` で results.md を更新） |
| `benchmarks/v20.0.0.json` | v20.0.0 実測ベースライン（CI で生成・コミット） |

---

### v20.2 — スーパー命令（Superinstruction）

**テーマ**: 最もコストが低く、即効性がある VM 最適化。

#### 仕組み

```
現状（3 回ディスパッチ）:
  LoadLocal(0) → Add → StoreLocal(1)

スーパー命令（1 回ディスパッチ）:
  AddLocalLocal(0, 1)   // "src0 + src1 を src0 に格納"
```

#### 実装方針

- v20.1 のベンチマーク結果から実測ホットパスを分析
- 頻出する opcode ペアを top-N でリストアップ
- 上位 10〜20 パターンをスーパー命令に融合

#### 期待改善（v20.0.0 比）

- `tight_loop_10m_iter`: **+20〜30%**（ディスパッチ削減）
- `record_transform_1m`: **+10〜15%**（フィールドアクセスパターン改善）

---

### v20.3 — NaN-boxing（VMValue の圧縮）

**テーマ**: VM の根幹。慎重に実施するが、インパクトは大きい。

#### 現状の問題

```rust
// 現状: enum は最大バリアントのサイズを全バリアントに適用
enum VMValue {
    Int(i64),      // 8 + 8 bytes（タグ + パディング）
    Float(f64),    // 8 + 8 bytes
    Bool(bool),    // 8 + 8 bytes
    Str(String),   // 8 + 24 bytes（String は 24 bytes）
    // ...
}
// → Vec<VMValue> は各要素が 32〜40 bytes、キャッシュミス多発
```

#### NaN-boxing 後

```
IEEE 754 の NaN には 2^52 個の "quiet NaN" パターンがある。
これらを使って型タグ + ポインタを 8 bytes に詰める。

[0x7FF8_0000_0000_0000] → NaN（Float）
[0x7FFx_xxxx_xxxx_xxxx] → タグ付きポインタ（Str/List/Record）
[0xFFF0_0000_xxxx_xxxx] → Int（32bit 範囲内）
[0xFFF1_0000_0000_000x] → Bool / Null
```

#### 期待改善（v20.0.0 比）

- `tight_loop_10m_iter`: **+2〜3x**（キャッシュヒット率大幅改善）
- `record_transform_1m`: **+1.5〜2x**

#### リスク管理

- 既存テスト（全件）がすべて通ることを移行の完了条件とする
- `--legacy-value-repr` フラグで旧表現にフォールバック可能にする

---

### v20.4 — DuckDB プッシュダウン最適化パス

**テーマ**: Favnir 固有の最大の武器。`fav explain --lineage` の静的解析を活用。

#### コンセプト

```favnir
// 現状: Favnir VM でフィルタ・集計を実行
stage Filter: List<Row> -> List<Row> = |rows| {
  List.filter(rows, |r| r.amount > 1000.0 && r.status == "active")
}
stage Aggregate: List<Row> -> List<Summary> = |rows| {
  List.group_by(rows, |r| r.category)
  |> Map.map(|k, v| Summary { category: k, total: List.sum_by(v, |r| r.amount) })
}
seq Report = LoadFromDb |> Filter |> Aggregate
```

↓ 最適化パスが検出して DuckDB SQL に変換

```sql
-- Favnir コンパイラが自動生成
SELECT category, SUM(amount) as total
FROM rows
WHERE amount > 1000.0 AND status = 'active'
GROUP BY category
```

#### 変換可能なパターン（Phase 1）

| Favnir 操作 | SQL 変換 |
|---|---|
| `List.filter(rows, \|r\| r.field > n)` | `WHERE field > n` |
| `List.group_by(rows, \|r\| r.key)` | `GROUP BY key` |
| `List.sum_by(rows, \|r\| r.val)` | `SUM(val)` |
| `List.map(rows, \|r\| r.field)` | `SELECT field` |
| `List.length(rows)` | `COUNT(*)` |

#### 期待改善（v20.0.0 比）

- `duckdb_query` ベンチマーク: **+10〜100x**（集計クエリの場合）

---

### v20.5 — mmap + SIMD CSV パーサー

**テーマ**: I/O 層の根本的な改善。v19.5 の Arrow 基盤と組み合わせる。

#### 現状の問題

```
現状:
  File → read() syscall → バイト列コピー → csv クレート（行単位パース）
  → Vec<String> アロケーション × N行 → VMValue 変換

最適化後:
  File → mmap（ゼロコピーマッピング） → arrow-csv（SIMD パース）
  → Arrow RecordBatch（列指向メモリ） → VM へ直接渡す
```

#### 実装

```rust
use memmap2::MmapOptions;
use arrow_csv::ReaderBuilder;

fn read_csv_mmap(path: &str) -> Result<RecordBatch, ...> {
    let file = File::open(path)?;
    let mmap = unsafe { MmapOptions::new().map(&file)? };
    let reader = ReaderBuilder::new(schema).build(Cursor::new(&mmap[..]))?;
    reader.next().transpose()?
}
```

#### 期待改善（v20.0.0 比）

- `csv_10gb_throughput`: **+3〜5x**（syscall 削減 + SIMD 解析）
- ピークメモリ: **-40%**（ゼロコピー）

---

### v20.6 — io_uring 非同期 I/O（Linux）

**テーマ**: Linux 5.1+ 限定だが、本番サーバー環境では最大の I/O 改善。

#### epoll vs io_uring

```
epoll:     read() syscall → kernel copy → user buffer
           1ファイル = 2回のコンテキストスイッチ

io_uring:  ring buffer への submit → 完了通知
           1000ファイル = ほぼ 0 回のコンテキストスイッチ
```

#### 実装方針

```toml
# Cargo.toml（Linux のみ）
[target.'cfg(target_os = "linux")'.dependencies]
tokio-uring = "0.4"
```

#### 期待改善（Linux 本番環境）

- 大量ファイル読み込みパイプライン: **+2〜4x**
- DB + ファイル I/O 混在パイプライン: **+1.5〜2x**
- Windows / macOS では自動で epoll / kqueue にフォールバック

---

### v20.7 — Arena アロケータ（GC なし高速アロケーション）

**テーマ**: パイプライン実行中のアロケーションをバッチ化して解放コストをゼロにする。

#### コンセプト

```
現状: VM が stage 実行のたびに Vec<VMValue> を個別アロケート・解放
      → malloc/free のオーバーヘッド

Arena: stage の実行単位（1 chunk = 1000 行）にアリーナを割り当て
      → chunk 処理完了時に一括解放（free が 1 回）
```

#### 期待改善

- `record_transform_1m`: **+20〜40%**（アロケーションコスト削減）
- ストリーミングパイプラインの定常メモリ: **-20%**

---

### v20.8 — DB コネクションプール統合

**テーマ**: データベースを使うパイプラインで「接続確立コスト」を排除する。

#### 現状の問題

```favnir
// 現状: stage ごとに接続を確立・解放
stage LoadUsers: Unit -> List<User> = |_| {
  bind conn <- Postgres.connect()  // ←毎回 ~50ms の接続確立
  Postgres.query(conn, "SELECT * FROM users")
}
```

#### 改善後

```favnir
// コネクションプールを pipeline レベルで共有
seq UserPipeline
  [pool: Postgres.Pool]
= LoadUsers(pool) |> Transform |> Save(pool)
```

```toml
# fav.toml
[postgres]
pool_size = 10
min_idle = 2
```

#### 期待改善

- DB を使う stage の初回実行: **-50ms〜**（接続確立コスト削減）
- 複数 stage が DB を使うパイプライン: **+2〜3x**（接続再利用）

---

## v21.0 — Runtime Excellence マイルストーン宣言

**完了条件:**

| ベンチマーク | v20.0.0 ベースライン | v21.0 目標 | 達成方法 |
|---|---|---|---|
| cold_start_precompiled | 18ms | **< 10ms** | NaN-boxing + 起動最適化 |
| csv_10gb_throughput | ~340 MB/s | **> 1 GB/s** | mmap + SIMD |
| tight_loop_10m_iter | ~85ms | **< 30ms** | スーパー命令 |
| record_transform_1m | ~210ms | **< 80ms** | NaN-boxing |
| duckdb_query（集計） | VM 実行 | **DuckDB 委譲（10〜100x）** | pushdown |

> 注: cold_start は絶対値での改善（-8ms）が目標。

---

## 参考リンク

- 前フェーズ: `versions/roadmap/roadmap-v19.1-v20.0.md`
- 次フェーズ: `versions/roadmap/roadmap-v21.1-v22.0.md`
- マスタースケジュール: `versions/roadmap-v20.1-v25.0.md`
