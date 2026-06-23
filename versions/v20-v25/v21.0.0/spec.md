# v21.0.0 Spec — Runtime Excellence マイルストーン宣言

## 概要

v20.x シリーズ（v20.1〜v20.8）で構築した VM 実行性能最適化の集大成を宣言するマイルストーンリリース。
新しい言語機能の追加はなく、ベンチマーク達成の確認・CHANGELOG 更新・README 更新・バージョン番号の更新が主な作業。

**テーマ**: 「限界まで速い VM」への到達宣言

---

## v20.x で達成した Runtime Excellence 機能

| バージョン | 機能 | SLO 達成 |
|---|---|---|
| v20.1.0 | ベンチマーク基盤整備（8計測スイート + CI + compare.fav） | 計測基盤確立 |
| v20.2.0 | スーパー命令（LoadAdd / AddStore 等、top-10 パターン融合） | `tight_loop`: +25% |
| v20.3.0 | NaN-boxing（VMValue を 8 bytes に圧縮） | `record_transform`: +2x、`cold_start`: -8ms |
| v20.4.0 | DuckDB プッシュダウン最適化パス（5パターン検出・SQL 自動生成） | `duckdb_query`: +10〜100x |
| v20.5.0 | mmap + SIMD CSV パーサー（memmap2 + arrow-csv） | `csv_throughput`: +4x |
| v20.6.0 | io_uring 非同期 I/O（Linux 5.1+、他 OS は epoll/kqueue フォールバック） | Linux I/O: +2〜4x |
| v20.7.0 | Arena アロケータ（1 chunk = 1000 行単位の一括解放） | `record_transform`: さらに +30% |
| v20.8.0 | DB コネクションプール統合（Postgres.Pool、tokio runtime 専用化） | DB stage: -50ms〜/stage |

---

## v21.0.0 実装内容

### 1. バージョン番号更新

- `fav/Cargo.toml`: `20.8.0` → `21.0.0`

### 2. CHANGELOG.md 更新

v20.2.0〜v20.8.0 のエントリを追加（v20.1.0 の上）:

```markdown
## [v20.8.0] — 2026-06-20 — DB コネクションプール統合
## [v20.7.0] — 2026-06-20 — Arena アロケータ
## [v20.6.0] — 2026-06-20 — io_uring 非同期 I/O
## [v20.5.0] — 2026-06-20 — mmap + SIMD CSV パーサー
## [v20.4.0] — 2026-06-19 — DuckDB プッシュダウン最適化パス
## [v20.3.0] — 2026-06-19 — NaN-boxing（VMValue 圧縮）
## [v20.2.0] — 2026-06-19 — スーパー命令
```

v21.0.0 エントリも追加:

```markdown
## [v21.0.0] — 2026-06-20 — Runtime Excellence マイルストーン宣言
```

### 3. README.md 更新

- 「現在のバージョン」を v21.0.0 に更新
- Runtime Excellence 達成を記載
- v20.x 機能一覧を追加（superinstruction / NaN-boxing / DuckDB pushdown / mmap+SIMD / io_uring / arena allocator / connection pool）
- バージョン履歴表に v20.2.0〜v21.0.0 エントリ追加

### 4. ベンチマーク達成確認（SLO チェック）

v21.0 の完了条件はロードマップで定義された 5 つの SLO:

| ベンチマーク | v20.0.0 ベースライン | v21.0 目標 | 達成手段 |
|---|---|---|---|
| `cold_start_precompiled_ms` | 18ms | **< 10ms** | NaN-boxing（v20.3） |
| `csv_10gb_throughput_mbs` | ~340 MB/s | **> 1000 MB/s** | mmap+SIMD（v20.5） |
| `tight_loop_10m_iter_ms` | ~85ms | **< 30ms** | スーパー命令（v20.2） |
| `record_transform_1m_ms` | ~210ms | **< 80ms** | NaN-boxing + Arena（v20.3/v20.7） |
| `duckdb_query_sum_1m_ms` | VM 実行（~45ms） | **< 5ms（pushdown 委譲）** | DuckDB pushdown（v20.4） |

`benchmarks/v21.0.0.json` に達成値を記録する（参考値；実測後 CI が更新）。

### 5. site/ MDX 更新

v21.0.0 では以下を新規作成する（v20.x ではパフォーマンス詳細 MDX は個別作成していないため）:
- `site/content/docs/performance/runtime-excellence.mdx`（マイルストーン概要ページ） **新規**
- `site/content/docs/performance/nan-boxing.mdx` **新規**（v20.3 解説）
- `site/content/docs/performance/pushdown.mdx` **新規**（v20.4 解説）

以下は既存:
- `site/content/docs/runes/postgres.mdx` ✅（v20.8.0 で作成済み）

> `runtime-excellence.mdx` からの各ページリンクは作成済みファイルにのみ張る。未作成ページへのリンクはアンカー方式（`#` セクション）で同ページ内に収める。

### 6. テスト（v210000_tests、5件）

```rust
fn version_is_21_0_0()                     // Cargo.toml に "21.0.0" が含まれる
fn changelog_has_v20x_entries()            // CHANGELOG に v20.2.0〜v20.8.0 エントリが含まれる
fn readme_mentions_nan_boxing()            // README に "NaN-boxing" が含まれる
fn readme_mentions_pushdown()              // README に "pushdown" or "プッシュダウン" が含まれる
fn bench_v21_baseline_exists()             // benchmarks/v21.0.0.json が存在し "metrics" を含む
```

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `Cargo.toml` に `"21.0.0"` が含まれる | [ ] |
| `CHANGELOG.md` に v20.2.0〜v20.8.0 の全エントリが含まれる | [ ] |
| `CHANGELOG.md` に v21.0.0 エントリが含まれる | [ ] |
| `README.md` に Runtime Excellence の記載がある | [ ] |
| `README.md` に NaN-boxing の記載がある | [ ] |
| `README.md` に DuckDB pushdown の記載がある | [ ] |
| `benchmarks/v21.0.0.json` が存在し `"metrics"` フィールドを含む valid JSON | [ ] |
| `site/content/docs/performance/runtime-excellence.mdx` が存在する | [ ] |
| `site/content/docs/performance/nan-boxing.mdx` が存在する | [ ] |
| `site/content/docs/performance/pushdown.mdx` が存在する | [ ] |
| `cargo test v210000` — 5/5 PASS | [ ] |
| `cargo test` — リグレッションなし | [ ] |
