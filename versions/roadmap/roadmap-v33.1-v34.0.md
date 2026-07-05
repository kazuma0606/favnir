# Roadmap v33.1.0 〜 v34.0.0 — Performance & Tooling

Date: 2026-07-01
Status: 骨格確定 + エフェクトシステム統一方針追記（2026-07-04）

---

## 目標

v33.0「Language Power」で「型で設計できる」を実現した。
次のフェーズは **「本番で速い」** だ。

現在の Favnir の性能上の制約:

```
✗ AOT ネイティブバイナリ
    → VM（バイトコード実行）のみ。Cranelift は Cargo.toml に存在するが未接続。
    → Lambda コールドスタートに約 200ms かかる

✗ インクリメンタルコンパイル
    → 毎回全ファイルを再コンパイル
    → 大規模プロジェクト（50 ファイル以上）で遅い

✗ ストリーミング評価
    → eager evaluation のみ。10GB CSV を処理するとメモリ不足になる

✗ Arrow 列指向統合
    → stage の出力は常にヒープ上の Vec<Value>（行指向）
    → Parquet 書き込みにコピーが発生

✗ WASM 最適化
    → Playground の初期ロードが遅い（WASM サイズが大きい）
```

> **Performance & Tooling の定義（本プロジェクト固有）**
> 「`fav build --target native` でネイティブバイナリが生成でき、
>  10GB CSV を定常メモリで処理でき、
>  Lambda コールドスタートが 100ms 以下になること」

---

## ⚠️ 重要：v33.0 完了後に更新が必要

このファイルは **骨格のみ** である。

v32.1〜v32.9 と v33.0 のドッグフード・宣言完了後に、
以下の判断を加えて各節を具体化する:

1. 実案件でのボトルネックが判明した後に AOT / Streaming / Arrow の優先度を確定
2. WASM と Native のどちらが先か（Playground の要求 vs Lambda の要求）
3. インクリメンタルコンパイルの実装コスト確認
4. v33.5〜v33.9 に何を入れるか

**更新担当**: v33.0 リリース時

---

## 設計決定事項（暫定）

| 項目 | 暫定決定 | 確定時期 |
|---|---|---|
| AOT バックエンド | Cranelift（`cranelift-codegen` v0.117 が Cargo.toml に存在）| v33.0 後 |
| コンパイルキャッシュの場所 | `~/.fav/cache/<project-hash>/` | v33.0 後 |
| ストリーミング構文 | `#[streaming(chunk_size = 1000)]` アノテーション | v33.0 後 |
| Arrow バージョン | `arrow = "52"`（Cargo.toml に存在）| v33.0 後 |
| WASM 最適化ツール | `wasm-opt`（Binaryen）統合 | v33.0 後 |
| **`!Effect` アノテーション** | **v34 系で廃止（ctx ベースに統一）** | **v34.x** |
| 破壊的変更 | `!Effect` 廃止のみ（v34.x、移行ガイド付き）| v34.x |

---

## エフェクトシステム統一方針（v34 系実装）

> 「型で設計する」を徹底するため、`!Effect` アノテーションを廃止し、
> Capability Context（ctx パラメータ）に一本化する。

### 設計原則

**純粋関数 = `ctx` パラメータなし**。コンパイラが副作用呼び出しを禁止する。

```favnir
// 純粋関数 — シグネチャに ctx なし、副作用呼び出し不可
fn transform(row: Row) -> Row {
    { ...row, amount: row.amount * 1.1 }
}
```

**副作用あり関数 = `ctx` パラメータあり**。`ctx` の型が「何の能力を使うか」を宣言する。

```favnir
// 副作用あり — ctx: LoadCtx が DbRead + IoCtx 能力を宣言
fn process(ctx: LoadCtx, id: Int) -> Result<Row, String> {
    bind { io, db } <- ctx                           // フィールドを束縛（alias）
    bind _ <- io.println("fetching " + id)
    bind rows <- db.query("SELECT * FROM t WHERE id = $1", [id])
    let result = transform(rows[0])                  // 純粋関数はそのまま呼べる
    Result.ok(result)
}
```

### `bind { ... } <- ctx` 構文

`ctx` のフィールドを短い名前に束縛する。新キーワード不要、既存の `bind` 構文を流用。
意味論的には **値レベルの alias**（`io` と `ctx.io` は同じ値）。

```favnir
bind { io, db } <- ctx       // ctx.io → io、ctx.db → db
bind { io } <- ctx           // io だけ取り出す
```

Haskell の `do` 記法における `let x = ...` と `<-` の関係に近いが、
Favnir では `bind` を純粋・副作用問わず統一的に使う。

### `!Effect` → ctx フィールド対応表

| 廃止する `!Effect` | 代替 ctx フィールド | 型 |
|---|---|---|
| `!Io` | `ctx.io` | `IoCtx` |
| `!DbRead` / `!DbWrite` | `ctx.db` | `DbRead` / `DbWrite` |
| `!Http` | `ctx.http` | `HttpClient` |
| `!Llm` | `ctx.llm` | `LlmClient` |
| `!Postgres` / `!MySQL` | `ctx.db` | `PgConn` / `MySqlConn` |
| `!Redis` | `ctx.redis` | `RedisClient` |
| `!Stream` | `ctx.stream` | `StreamClient` |
| `!Trace` | `ctx.tracer` | `Tracer` |
| `!Snowflake` | `ctx.warehouse` | `SnowflakeConn` |
| `!Emit<T>` | `ctx.emitter` | `Emitter<T>` |

### 移行対象（破壊的変更）

`!Effect` 廃止は **破壊的変更** であるため、以下すべての移行が必要：

1. **コンパイラ・型チェッカー**
   - `fav/src/ast.rs` — `Effect` enum を deprecated 化（最終的に削除）
   - `fav/src/middle/checker.rs` — `!Effect` チェックロジックを ctx 検査に置換
   - `fav/src/frontend/parser.rs` — `!Effect` パース継続（移行期）→ 警告発行
   - `fav/src/lint.rs` — W??? `deprecated_effect_annotation` ルール追加

2. **セルフホストファイル**
   - `fav/self/compiler.fav` — `!Effect` 宣言を ctx 引数に書き換え
   - `fav/self/checker.fav` — 同上

3. **Rune ファイル**（`runes/` 配下、全 50+ rune）
   - `runes/postgres/client.fav` 等の `!Postgres` → `ctx.db: PgConn`
   - `runes/redis/redis.fav` の `!Redis` → `ctx.redis: RedisClient`
   - 他すべての rune の `!Effect` 宣言

4. **E2E デモ・examples**
   - `infra/e2e-demo/` 配下の `.fav` ファイル
   - `examples/` 配下の `.fav` ファイル
   - `fav/tests/fixtures/` 配下のテストフィクスチャ

5. **ドキュメントサイト・README**
   - `site/content/docs/` 配下の MDX（コードサンプルを全更新）
   - `site/content/learn/` の入門記事（チュートリアルの `!Effect` 例を ctx 構文に書き換え）
   - `site/content/docs/runes/` 各 Rune ページ（`!Postgres` / `!Redis` 等の使用例を更新）
   - `README.md` — 言語概要・コードサンプルの `!Effect` 構文を ctx 構文に更新
   - `README.md` — エフェクトシステム統一の設計思想セクションを追記

### v33.x での準備作業（v34 本実装前）

- `W0XX deprecated_effect_annotation` 警告をコンパイラに追加（`!Effect` 使用時に警告）
- `fav migrate --effects` コマンド（自動移行ツール）の設計・実装
- `IoCtx` interface 定義（`io.println` / `io.read_file` / `io.env` 等）の追加

---

## バージョン計画（骨格）

### v33.1 — AOT ネイティブバイナリ（Cranelift）

**テーマ**: `fav build --target native` でネイティブバイナリを生成する。

```bash
# ネイティブバイナリとしてビルド
fav build --target native src/main.fav -o pipeline
./pipeline

# Lambda 向けの静的バイナリ
fav build --target native --static src/main.fav -o bootstrap
```

**背景**: `cranelift-codegen = "0.117"` がすでに `Cargo.toml` の依存に存在する。
これを VM バイトコードの実行に使うか、直接ネイティブコードを生成するかを確定する。

実装方針（骨格）:
- `fav/src/backend/aot.rs` を新規作成
- IR → Cranelift IR → ネイティブバイナリの変換パイプラインを実装
- `fav run`（VM）との互換性維持（`--target vm` で従来通り動作）

---

### v33.2 — インクリメンタルコンパイル

**テーマ**: 変更ファイルのみ再コンパイルして開発サイクルを高速化する。

```
~/.fav/cache/
  <project-hash>/
    <file-hash>.ast    # AST キャッシュ
    <file-hash>.types  # 型情報キャッシュ
    <file-hash>.ir     # IR キャッシュ
```

実装方針（骨格）:
- ファイルのコンテンツハッシュ（SHA256）でキャッシュヒットを判定
- 依存グラフ追跡（A が B を import していたら B の変更で A も無効化）
- `fav build --no-cache` でキャッシュを無視

---

### v33.3 — ストリーミング評価（#[streaming]）

**テーマ**: `#[streaming]` でパイプラインをストリーミング評価に切り替える。

```favnir
// 全データをメモリに乗せずに処理
#[streaming(chunk_size = 1000)]
seq LargeDataPipeline = LoadCsv |> Transform |> WriteToDb

// 内部動作:
// 1. LoadCsv が 1000 行ずつ生成
// 2. Transform が 1000 行ずつ処理
// 3. WriteToDb が 1000 行ずつ書き込み
```

実装方針（骨格）:
- `#[streaming]` アノテーションのパース追加
- stage の入出力を `Iterator<Item = T>` に変換するコード生成
- バックプレッシャー対応（`chunk_size` による制御）

---

### v33.4 — Arrow 列指向統合

**テーマ**: stage の出力を Arrow RecordBatch として格納し、Parquet 書き込みをゼロコピーに。

```favnir
// Arrow RecordBatch として直接操作
stage AnalyzeData: ArrowBatch -> ArrowBatch !IO = |batch| {
    bind filtered <- Arrow.filter(batch, |row| row.amount > 100.0)
    bind sorted   <- Arrow.sort_by(filtered, "amount")
    Result.ok(sorted)
}
```

**背景**: `arrow = "52"` がすでに `Cargo.toml` の依存に存在する。

実装方針（骨格）:
- `ArrowBatch` 型を VM に追加
- `Arrow.*` namespace の基本関数（filter / sort_by / select）
- Parquet 書き込み時に ArrowBatch を直接使用（ゼロコピー）

---

### v33.5 — fav run --precompiled

**テーマ**: 事前コンパイル済みアーティファクトで起動して Lambda コールドスタートを削減。

```bash
# 事前コンパイル
fav compile src/main.fav -o pipeline.favc

# キャッシュ済みアーティファクトで起動（コンパイル不要）
fav run --precompiled pipeline.favc
# 起動時間: ~5ms（現在: ~200ms）
```

---

### v33.6 — WASM 最適化

**テーマ**: Playground の初期ロードを高速化する。

目標:
- WASM サイズ 50% 削減
- 初期実行 100ms 以下

実装方針（骨格）:
- `wasm-opt`（Binaryen）による最適化パス統合（CI に追加）
- デッドコード除去（使われていない stdlib 関数を除外）
- WASM コンポーネントモデル対応検討

---

### v33.7 — エフェクトシステム移行準備

**テーマ**: v34 系での `!Effect` 廃止に向けた準備。破壊的変更を段階的に導入。

```favnir
// 移行前（現在）
fn fetch(url: String) -> Result<String, String> !Http {
    HTTP.get(url)
}

// 移行後（v34 系）
fn fetch(ctx: AppCtx, url: String) -> Result<String, String> {
    bind { http } <- ctx
    http.get(url)
}
```

実装内容:
- `W0XX deprecated_effect_annotation` lint ルール追加（`!Effect` 使用箇所に警告）
- `IoCtx` interface 定義（`io.println` / `io.read_file` / `io.env` 等）
- `fav migrate --effects` 自動移行コマンド（`!Effect` → ctx 引数への書き換え支援）
- `AppCtx` に `io: IoCtx` フィールドを追加

---

### v33.8〜v33.9 — 状況で決定

v33.0 完了後のドッグフード・パフォーマンス計測結果で以下から選択:

- プロファイリング強化（flamegraph 生成 / `fav profile --flamegraph`）
- 並列コンパイル（ファイル単位での並列型チェック）
- メモリレイアウト最適化（Arena アロケーション）
- Lambda デプロイ統合（`fav deploy --target lambda`）

---

## v34.0 — Performance & Tooling マイルストーン宣言

**暫定完了条件（v33.0 完了後に確定）:**

| コンポーネント | 暫定完了基準 |
|---|---|
| AOT バイナリ | `fav build --target native` でバイナリが生成される |
| インクリメンタル | 2 回目以降のビルドがキャッシュヒットで高速化される |
| ストリーミング | `#[streaming]` パイプラインが 10GB CSV を定常メモリで処理できる |
| Arrow 統合 | Parquet 書き込みが ArrowBatch 経由でゼロコピーになる |
| precompiled | `fav run --precompiled` で起動時間が大幅に短縮される |
| WASM 最適化 | WASM サイズが v30.0.0 比 50% 削減される |
| 移行準備 | `W0XX deprecated_effect_annotation` が実装済み、`fav migrate --effects` が動作する |

**★ クリーンアップ実施（v34.0 リリース時）:**

```bash
cd /c/Users/yoshi/favnir/fav
cargo clean
cargo build
cargo test 2>&1 | grep "test result"
du -sh target/
```

---

## 参考リンク

- マスタースケジュール: `versions/roadmap/roadmap-v30.1-v35.0.md`
- 前フェーズ: `versions/roadmap/roadmap-v32.1-v33.0.md`
- 次フェーズ: `versions/roadmap/roadmap-v34.1-v35.0.md`
