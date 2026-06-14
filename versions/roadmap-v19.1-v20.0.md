# Roadmap v19.1.0 〜 v20.0.0 — Production Performance

Date: 2026-06-14

## 目標

v19.0.0「Type System Maturity」で「信頼できる言語」への転換を果たした。
最後のテーマは「**本番で速い言語**」——実際のユーザーが書いたパイプラインを、
本番環境で最大限に速く動かすことである。

**なぜ最後か:**
パフォーマンス最適化は「何を最適化すべきか」がわかってから行うものである。
v17〜v19 で言語が成熟し、実際のユーザーがどのパイプラインを書くかが見えてきてから
ボトルネックを特定・解消する。「早すぎる最適化は諸悪の根源」。

**目標数値:**
- 10GB CSV の処理: 現在比 10x 高速化
- メモリ使用量: 現在比 50% 削減
- Lambda コールドスタート: 100ms 以下
- 2 回目以降のビルド: 5x 高速化（インクリメンタルコンパイル）

- v19.1: ストリーミング評価で大規模データをメモリに乗せずに処理
- v19.2: Cranelift バックエンドでネイティブバイナリ生成（バイトコード VM から脱却）
- v19.3: インクリメンタルコンパイルで変更ファイルのみ再コンパイル
- v19.4: 並列コンパイルでファイル単位の並列ビルドを実現
- v19.5: Arrow 形式によるメモリレイアウト最適化（列指向 + SIMD フレンドリー）
- v19.6: WASM バイナリ最適化（サイズ 50% 削減・初期ロード 100ms 以下）
- v19.7: 事前コンパイルキャッシュ（Lambda 起動 ~5ms）
- v19.8: フレームグラフ生成でプロファイリングを本格化
- v20.0: Production Performance マイルストーン宣言

---

## 設計決定事項

| 項目 | 決定 |
|---|---|
| ストリーミング評価モデル | `#[streaming(chunk_size = N)]` アノテーション。push-based イテレータで stage 間を接続 |
| AOT バックエンド | Cranelift（`cranelift-codegen` crate）。LLVM は依存が重すぎるため採用しない |
| インクリメンタルキャッシュ形式 | コンテンツハッシュ（SHA-256）+ mtime のダブルチェック。`~/.fav/cache/<project-hash>/` |
| 並列コンパイルのスケジューラ | Rust `rayon` による依存グラフのトポロジカルソート後並列実行 |
| Arrow 統合の方針 | stage の出力型を `ArrowBatch<T>` に昇格できる。既存の `List<T>` との互換レイヤーを提供 |
| WASM 最適化ツールチェーン | `wasm-opt`（Binaryen）を `fav build --target wasm` に統合 |
| 事前コンパイルアーティファクト | `.favc`（Favnir Compiled）形式。バイトコード VM 命令列 + メタデータ |
| フレームグラフ形式 | `inferno` 互換（Flamegraph.pl 互換の折り畳みスタックフォーマット）|

---

## バージョン計画

### v19.1.0 — 遅延評価パイプライン（Lazy / Streaming Evaluation）

**テーマ**: 大規模データを定常メモリで処理できるようにする。
現在: 全ステージで全データをメモリに乗せる（eager evaluation）。
目標: `#[streaming]` アノテーションで chunk 単位のストリーミング評価に切り替える。

**現状の問題:**

```fav
// 現状: 10GB CSV を全部メモリに乗せてから Transform / Save を実行
seq LargeDataPipeline = LoadCsv |> Transform |> WriteToDb
// LoadCsv が全行を List<Row> として返す → 10GB がメモリ上に展開される
```

**ストリーミング評価:**

```fav
// #[streaming] でチャンク単位の評価に切り替え
#[streaming(chunk_size = 1000)]
seq LargeDataPipeline = LoadCsv |> Transform |> WriteToDb

// 内部動作:
// 1. LoadCsv が 1000 行ずつ生成（Iterator<Item = List<Row>>）
// 2. Transform が 1000 行ずつ処理
// 3. WriteToDb が 1000 行ずつ書き込み
// → 最大メモリ使用量 ≈ chunk_size × row_size（全データを保持しない）

// chunk_size を指定しない場合はデフォルト値を使用
#[streaming]
seq Pipeline = LoadCsv |> Transform |> Save

// ストリーミングと通常のパイプラインを混在（自動的にバッファリング）
#[streaming(chunk_size = 500)]
seq Mixed =
  LoadCsv        // ストリーミング
  |> Transform   // ストリーミング
  |> Aggregate   // ストリーミング（各 chunk で集計 → 最終 merge）
  |> Save        // 集計結果のみ保存（小さい）
```

**stage の対応:**

```fav
// ストリーミング対応 stage: List<T> を受け取り List<U> を返す（chunk 単位で呼ばれる）
stage Transform(rows: List<RawRow>) -> List<OutputRow> {
  Result.ok(List.map(rows, transform_row))
  // #[streaming] パイプラインからは chunk ごとに呼ばれる
}

// ステートフル stage（chunk 間で状態を持つ）
#[stateful]
stage RunningAvg(rows: List<Row>, state: AvgState) -> Pair<List<Row>, AvgState> {
  let new_state = update_state(state, rows)
  Result.ok(Pair(annotate_with_avg(rows, new_state), new_state))
}
```

**実装内容:**

- `fav/src/ast.rs`:
  - `Annotation::Streaming { chunk_size: Option<Int> }` 追加
  - `Annotation::Stateful` 追加
  - `SeqPipeline.streaming: bool` フィールド追加

- `fav/src/middle/compiler.rs`:
  - `compile_streaming_pipeline`: 通常の `SeqCall` opcode の代わりに
    `StreamInit` / `StreamNext(chunk_size)` / `StreamMap(stage_idx)` / `StreamEnd` opcode 列を生成

- `fav/src/backend/vm.rs`:
  - `StreamInit` opcode: パイプラインをストリーミングモードで初期化
  - `StreamNext(n)` opcode: 最大 n 要素を source stage から取得
  - `StreamMap(stage)` opcode: chunk を stage に渡して変換
  - `StreamEnd` opcode: ストリームの終了処理
  - `FileRowIterator`: CSV / JSONL ファイルを行単位で読み込む Iterator

- `fav/src/driver.rs`:
  - `fav run --streaming`: `#[streaming]` なしのパイプラインでも強制ストリーミング
  - `fav run --chunk-size N`: chunk サイズをオーバーライド

- テスト: `v191000_tests`（5件）:
  - `version_is_19_1_0`
  - `streaming_annotation_parses`（`#[streaming(chunk_size = 1000)]` が解析される）
  - `streaming_pipeline_executes`（ストリーミングパイプラインが正しい結果を返す）
  - `streaming_memory_bounded`（大量データでもメモリ使用量が chunk_size に比例する）
  - `streaming_stateful_stage`（`#[stateful]` stage が chunk 間で状態を保持する）

**完了条件（PASS=5）:**
1. `#[streaming(chunk_size = 1000)]` が解析・実行される
2. ストリーミングパイプラインが通常パイプラインと同じ結果を返す
3. chunk_size 行ずつ処理されることが確認できる（ログ等で）
4. `#[stateful]` で chunk 間の集計状態が保持される
5. 10 万行以上のデータでメモリが chunk_size × row_size 程度に収まる

---

### v19.2.0 — AOT コンパイル（Cranelift バックエンド）

**テーマ**: バイトコード VM から脱却し、ネイティブバイナリを生成する。
Cranelift を AOT バックエンドとして採用し、実行速度を大幅に改善する。

**現状と目標の比較:**

```
現状: .fav → (コンパイル) → バイトコード → (VM インタープリタ) → 実行
目標: .fav → (AOT コンパイル) → ネイティブバイナリ → 実行
```

**CLI:**

```bash
# ネイティブバイナリとしてビルド
fav build --target native src/pipeline.fav -o pipeline

# 実行（fav run 不要、直接実行）
./pipeline

# クロスコンパイル
fav build --target x86_64-unknown-linux-musl src/pipeline.fav -o pipeline-linux

# VM モード（従来通り）
fav build --target vm src/pipeline.fav -o pipeline.favc
fav run --precompiled pipeline.favc
```

**Cranelift の採用理由:**

| | Cranelift | LLVM |
|---|---|---|
| Rust との統合 | ネイティブ（wasmtime と共通） | FFI（libLLVM が大きい） |
| ビルド時間 | 速い（設計目標） | 遅い |
| 最適化品質 | 中程度（JIT 向け設計） | 高い |
| 依存サイズ | 小さい | 非常に大きい |

**IR → Cranelift IR の変換:**

```
Favnir IR                    Cranelift IR（CLIF）
--------                     -------------------
Opcode::Push(Int(n))    →    iconst.i64 n
Opcode::Add             →    iadd
Opcode::Call(fn_idx)    →    call fn_name(args...)
Opcode::Jump(target)    →    jump block_label
Opcode::JumpIf(target)  →    brnz val, block_label
```

**実装内容:**

- `fav/src/backend/cranelift.rs`（新規）:
  - `CraneliftBackend`: Favnir IR → Cranelift IR（CLIF）変換
  - `emit_native`: Cranelift `Module` → ネイティブオブジェクトファイル生成
  - `link_native`: オブジェクトファイルをリンク（`lld` または `cc` 経由）

- `fav/src/driver.rs`:
  - `cmd_build(target: BuildTarget, out: &str)` 実装
  - `BuildTarget::Native / Vm / Wasm`

- `Cargo.toml`:
  - `cranelift-codegen`, `cranelift-module`, `cranelift-object` を依存に追加

- テスト: `v192000_tests`（5件）:
  - `version_is_19_2_0`
  - `build_target_native_produces_binary`（`--target native` でバイナリが生成される）
  - `native_binary_executes`（生成バイナリが正しい出力を返す）
  - `native_vs_vm_same_output`（native と VM の出力が一致する）
  - `build_target_vm_still_works`（`--target vm` が従来通り動作する）

**完了条件（PASS=5）:**
1. `fav build --target native` でネイティブバイナリが生成される
2. 生成バイナリが `fav run` と同じ結果を返す
3. `fav build --target vm` も引き続き動作する
4. ネイティブバイナリの実行速度が VM 比で向上している（計測値を記録）
5. `x86_64-unknown-linux-musl` ターゲットのクロスコンパイルが動作する

---

### v19.3.0 — インクリメンタルコンパイル

**テーマ**: 変更されたファイルのみを再コンパイルする。
現在: 毎回全ファイルを再コンパイル（大きなプロジェクトで遅い）。

**キャッシュ構造:**

```
~/.fav/cache/
  <project-content-hash>/
    <file-content-hash>.ast   # パース済み AST
    <file-content-hash>.types # 型チェック結果
    <file-content-hash>.ir    # コンパイル済み IR
```

**依存グラフ追跡:**

```
src/
  pipeline.fav      ← use utils.{ format_date }    → utils.fav に依存
  utils.fav         ← use json                     → rune "json" に依存
  types.fav                                         → 他に依存しない

依存グラフ:
  pipeline.fav → utils.fav → rune:json
  types.fav（独立）

変更検出:
  utils.fav が変更 → utils.fav と pipeline.fav を再コンパイル
  types.fav が変更 → types.fav のみ再コンパイル
```

**ビルド時間の改善（目標）:**

```
プロジェクト規模: 50 ファイル

初回ビルド:    10.0s（全ファイルコンパイル）
2 回目以降:     0.3s（変更なし → キャッシュヒット）
1 ファイル変更: 1.2s（変更ファイル + 依存ファイルのみ再コンパイル）
```

**実装内容:**

- `fav/src/incremental/`（新規ディレクトリ）:
  - `cache.rs`: キャッシュの読み書き（AST / 型情報 / IR を `.fav/cache/` に保存）
  - `dep_graph.rs`: ファイル依存グラフの構築・更新
  - `fingerprint.rs`: ファイルのコンテンツハッシュ（SHA-256）計算

- `fav/src/driver.rs`:
  - `cmd_build` / `cmd_check` でインクリメンタルキャッシュを使用
  - `--no-cache` フラグ: キャッシュを無視して完全再ビルド
  - `--explain-cache`: キャッシュヒット / ミスの詳細を表示

- テスト: `v193000_tests`（5件）:
  - `version_is_19_3_0`
  - `cache_creates_on_first_build`（初回ビルドでキャッシュが生成される）
  - `cache_hits_on_second_build`（変更なし 2 回目でキャッシュヒット）
  - `cache_invalidates_on_change`（ファイル変更でキャッシュが無効化される）
  - `dep_graph_propagates`（A が B を use → B 変更で A も再コンパイル）

**完了条件（PASS=5）:**
1. 初回ビルドでキャッシュファイルが `~/.fav/cache/` に生成される
2. 変更なしの 2 回目ビルドがキャッシュを使用し高速化される
3. ファイル変更でそのファイルのキャッシュが無効化される
4. 依存グラフが正しく追跡され、変更が伝播する
5. `--no-cache` で完全再ビルドが強制できる

---

### v19.4.0 — 並列コンパイル

**テーマ**: ファイル単位でコンパイルを並列化する。
現在: シングルスレッドで全ファイルを順次コンパイル。

**並列化フェーズ:**

```
フェーズ 1: AST 生成（全ファイル完全並列）
  src/a.fav ──┐
  src/b.fav ──┤ rayon par_iter → [AST_a, AST_b, AST_c, ...]
  src/c.fav ──┘

フェーズ 2: 型チェック（依存グラフのトポロジカルソート後に並列）
  types.fav → （依存なし → 最初に処理）
  utils.fav → types.fav に依存 → types.fav 完了後に処理
  pipeline.fav → utils.fav に依存 → utils.fav 完了後に処理

フェーズ 3: IR 生成（型チェック済みファイルを並列）
  [IR_types, IR_utils, IR_pipeline] = 並列生成

フェーズ 4: リンク（シングルスレッド、全 IR を結合）
  final_artifact = link([IR_types, IR_utils, IR_pipeline])
```

**期待するスケールアップ:**

```
ファイル数:  1    5    10   20   50
シングル:  1.0  5.0  10.0  20.0  50.0（秒）
4コア並列: 1.0  1.5   3.0   6.0  14.0（秒）
8コア並列: 1.0  1.2   2.0   4.0   8.0（秒）
```

**実装内容:**

- `fav/src/driver.rs`:
  - `compile_parallel(files: Vec<PathBuf>) -> Vec<IR>` 実装
  - `rayon::iter::ParallelIterator` でフェーズ 1 / 3 を並列化
  - フェーズ 2: `petgraph` で依存グラフのトポロジカルソート + 層単位の並列処理

- `Cargo.toml`:
  - `rayon 1.x` を依存に追加
  - `petgraph 0.6` を依存に追加（依存グラフ管理）

- 共有状態の競合解消:
  - 型チェック環境（`Env`）をファイル単位にスコープ分割
  - グローバル型テーブルは `Arc<RwLock<TypeTable>>` で保護

- テスト: `v194000_tests`（5件）:
  - `version_is_19_4_0`
  - `parallel_compile_same_output`（並列と逐次の出力が一致）
  - `parallel_compile_faster`（10 ファイル以上で並列が逐次より速い）
  - `parallel_dep_order_respected`（依存順序が正しく守られる）
  - `parallel_compile_thread_count`（`--jobs N` でスレッド数制御）

**完了条件（PASS=5）:**
1. 並列コンパイルの出力が逐次コンパイルと一致する
2. 10 ファイル以上のプロジェクトで並列化による高速化が確認される
3. 依存グラフが正しくトポロジカルソートされ、依存順序が守られる
4. `--jobs N` でコンパイルスレッド数を制御できる
5. データ競合なしに動作する（Rust の型システムが保証）

---

### v19.5.0 — メモリレイアウト最適化（Apache Arrow 統合）

**テーマ**: `Value` enum の非効率なメモリレイアウトを Apache Arrow 形式に置き換える。
列指向ストレージで SIMD 最適化・Parquet 書き込みのゼロコピーを実現する。

**現状のメモリレイアウト問題:**

```rust
// 現在の Value enum
enum Value {
    Int(i64),        // 8 bytes + 1 byte tag = 9 bytes（ただし alignment で 16 bytes）
    Float(f64),      // 8 bytes + 1 byte tag = 16 bytes
    Str(String),     // 24 bytes (ptr + len + cap) + tag = 32 bytes
    Bool(bool),      // 1 byte + tag = 16 bytes（padding）
    List(Vec<Value>),// 24 bytes + tag = 32 bytes
    // ...
}

// Vec<Value> でのレコードリスト: 各レコードが別々のメモリ領域
// キャッシュミスが多発、SIMD 不可
```

**Arrow 形式（列指向）:**

```
通常の行指向:
  row 0: [id:1,  name:"Alice",  amount:100.0]
  row 1: [id:2,  name:"Bob",    amount:200.0]
  row 2: [id:3,  name:"Charlie",amount:300.0]

Arrow の列指向（RecordBatch）:
  id:     [1, 2, 3]           ← i64 の連続配列（SIMD フレンドリー）
  name:   ["Alice","Bob","Charlie"] ← StringArray
  amount: [100.0, 200.0, 300.0]    ← f64 の連続配列
```

**Favnir での使用:**

```fav
// stage の出力が Arrow RecordBatch として格納
#[arrow]
stage Transform(rows: List<RawRow>) -> List<OutputRow> {
  Result.ok(List.map(rows, transform_row))
  // 内部的に ArrowBatch<OutputRow> として格納
}

// Parquet 書き込みがゼロコピー
fn write_to_parquet(rows: ArrowBatch<OutputRow>, path: String) -> Result<Unit, String> !IO {
  IO.write_parquet(path, rows)  // RecordBatch → Parquet への変換がゼロコピー
}

// 通常の List<T> との互換レイヤー
fn process(rows: List<OutputRow>) -> Result<Unit, String> {
  let batch = ArrowBatch.from_list(rows)   // List → Arrow 変換
  let list  = ArrowBatch.to_list(batch)    // Arrow → List 変換
  ...
}
```

**実装内容:**

- `fav/src/backend/arrow.rs`（新規）:
  - `ArrowBatch<T>`: Apache Arrow `RecordBatch` の Favnir ラッパー
  - `Value` → Arrow 列変換（`Int → Int64Array`, `Float → Float64Array`, etc.）
  - `ArrowBatch.from_list` / `ArrowBatch.to_list` 変換関数

- `Cargo.toml`:
  - `arrow 53.x` を依存に追加（`arrow-array`, `arrow-schema`, `parquet`）

- `fav/src/backend/vm.rs`:
  - `#[arrow]` アノテーション付き stage の実行を `ArrowBatch` で処理

- `runes/stdlib/arrow.fav`:
  - `ArrowBatch.*` Favnir ラッパー（`from_list` / `to_list` / `write_parquet` / `read_parquet`）

- テスト: `v195000_tests`（5件）:
  - `version_is_19_5_0`
  - `arrow_batch_from_list`（`ArrowBatch.from_list` が正しく変換する）
  - `arrow_batch_to_list`（`ArrowBatch.to_list` が元のリストと一致する）
  - `arrow_parquet_roundtrip`（Parquet 書き込み → 読み込みでデータが一致する）
  - `arrow_stage_executes`（`#[arrow]` stage が正しい結果を返す）

**完了条件（PASS=5）:**
1. `ArrowBatch.from_list(rows)` で `List<T>` を Arrow 形式に変換できる
2. `ArrowBatch.to_list(batch)` で Arrow 形式を `List<T>` に戻せる
3. `IO.write_parquet` / `IO.read_parquet` が動作する（Parquet ラウンドトリップ）
4. `#[arrow]` アノテーション付き stage が正しい結果を返す
5. Arrow 形式でのデータ処理速度が通常の `List<Value>` より向上している（計測値を記録）

---

### v19.6.0 — WASM バイナリ最適化

**テーマ**: Playground の初期ロードを高速化し、WASM バイナリサイズを削減する。
現在: WASM サイズが大きく、Playground の初期ロードが遅い。
目標: WASM サイズ 50% 削減・初期実行 100ms 以下。

**現状の問題:**

```
現在の @favnir/wasm バイナリ:
  - サイズ: ~8MB（gzip 後 ~2MB）
  - 初期ロード: ~500ms（モバイル 3G では ~2s）
  - 全 stdlib がバンドルされている（使われない関数も含む）
```

**最適化アプローチ:**

```bash
# ビルドパイプライン（v19.6 以降）
fav build --target wasm src/compiler.fav

# ステップ 1: Rust 側のデッドコード除去
# --target wasm で使われる VM opcode のみをコンパイル

# ステップ 2: wasm-opt（Binaryen）で最適化
wasm-opt -O3 --strip-debug --vacuum favnir.wasm -o favnir.opt.wasm

# ステップ 3: wasm-snip で使われない関数を削除

# 結果目標:
# - サイズ: ~4MB（gzip 後 ~1MB）
# - 初期ロード: ~150ms（デスクトップ）
```

**WASM コンポーネントモデル対応:**

```fav
// Wasm Interface Types (WIT) に対応した Rune
// ブラウザ / WASI どちらでも動作

// Browser 向け: @favnir/wasm npm パッケージ
import { run } from "@favnir/wasm"
const result = await run(`
  fn main() -> String {
    "Hello from Favnir!"
  }
`)

// WASI 向け: wasm32-wasi ターゲット
fav build --target wasm32-wasi src/pipeline.fav -o pipeline.wasm
wasmtime pipeline.wasm
```

**実装内容:**

- `fav/src/backend/wasm.rs`:
  - 使用される opcode のみをコンパイルするデッドコード解析追加
  - `wasm-encoder` の出力に `wasm-opt` を自動適用

- `build-wasm.sh` 更新:
  - `wasm-opt` / `wasm-snip` のビルドパイプラインに統合
  - `wasm32-wasi` ターゲット追加

- `site/src/wasm/`:
  - 遅延ロード（lazy loading）: Playground が表示されるまで WASM をロードしない
  - Web Worker でのバックグラウンド実行

- テスト: `v196000_tests`（5件）:
  - `version_is_19_6_0`
  - `wasm_binary_size_reduced`（最適化後サイズが最適化前比 40% 以上削減）
  - `wasm_opt_applies`（`wasm-opt` が自動実行される）
  - `wasm_output_correct`（最適化後も正しい出力を返す）
  - `wasm_wasi_target`（`wasm32-wasi` ターゲットが動作する）

**完了条件（PASS=5）:**
1. `wasm-opt` が `fav build --target wasm` に自動統合される
2. WASM バイナリサイズが v16.0.0 比 40% 以上削減される
3. 最適化後も Playground での実行結果が正しい
4. `wasm32-wasi` ターゲットが動作する
5. Playground の初期ロードが改善される（計測値を記録）

---

### v19.7.0 — 事前コンパイルキャッシュ（`fav compile` / `fav run --precompiled`）

**テーマ**: Lambda / ECS での高速コールドスタートを実現する。
現在: Lambda 起動ごとに `fav` がソースをコンパイルする（~200ms）。
目標: 事前コンパイルしたアーティファクト（`.favc`）で起動時間を ~5ms に短縮。

**使用フロー:**

```bash
# 開発時: ソースから直接実行（従来通り）
fav run src/pipeline.fav

# デプロイ前: 事前コンパイル
fav compile src/pipeline.fav -o pipeline.favc

# Lambda での実行: コンパイル不要
fav run --precompiled pipeline.favc
# 起動時間: ~5ms（コンパイル: 0ms + VM 初期化: ~5ms）

# fav deploy との統合（自動的に .favc を生成してデプロイ）
fav deploy --precompile
```

**`.favc` ファイル形式:**

```
[ヘッダー]
magic:   "FAVC"（4 bytes）
version: u32（バイトコードフォーマットバージョン）
arch:    u8（ターゲットアーキテクチャ: 0=any_vm, 1=x86_64, 2=aarch64）

[メタデータ]
source_hash: SHA-256（元ソースのハッシュ）
compiled_at: Unix timestamp
compiler_ver: semver 文字列

[バイトコード]
opcodes: Vec<u8>（VM 命令列）
constants: Vec<Value>（定数テーブル）
fn_table: Vec<FnEntry>（関数テーブル）
```

**Lambda bootstrap の変更:**

```bash
#!/bin/sh
# bootstrap スクリプト（Lambda カスタムランタイム）

# 変更前: ソースをコンパイルして実行（~200ms）
exec fav run --legacy /var/task/pipeline.fav

# 変更後: 事前コンパイル済みアーティファクトを実行（~5ms）
exec fav run --precompiled /var/task/pipeline.favc
```

**実装内容:**

- `fav/src/backend/artifact.rs`（新規）:
  - `.favc` ファイルの読み書き
  - ヘッダー・メタデータ・バイトコードのシリアライズ（`bincode` crate）
  - バージョン互換性チェック

- `fav/src/driver.rs`:
  - `cmd_compile(src: &str, out: &str)` 実装
  - `cmd_run` に `--precompiled` フラグ追加（コンパイルフェーズをスキップ）

- `scripts/build-lambda-layer.sh` 更新:
  - `fav compile` で `.favc` を生成してデプロイパッケージに含める

- テスト: `v197000_tests`（5件）:
  - `version_is_19_7_0`
  - `compile_produces_favc`（`fav compile` で `.favc` が生成される）
  - `precompiled_runs`（`fav run --precompiled` が正しく動作する）
  - `precompiled_same_output`（通常実行と事前コンパイル実行の出力が一致）
  - `favc_version_check`（異なるバイトコードバージョンで適切なエラー）

**完了条件（PASS=5）:**
1. `fav compile src/pipeline.fav -o pipeline.favc` で `.favc` ファイルが生成される
2. `fav run --precompiled pipeline.favc` が正しく実行される
3. 通常の `fav run` と `--precompiled` の出力が一致する
4. `.favc` のバージョンミスマッチで適切なエラーが出る
5. `fav deploy --precompile` で自動的に `.favc` を生成してデプロイする

---

### v19.8.0 — プロファイリング強化（フレームグラフ）

**テーマ**: 現在の `fav profile`（stage 別実行時間のみ）を、関数レベル・行レベルの
フレームグラフ生成まで拡張する。ボトルネックを視覚的に特定できるようにする。

**現状と目標の比較:**

```bash
# 現状（v9.9.0 で実装済み）
fav run --profile src/pipeline.fav
# 出力:
# stage LoadCsv:   45ms
# stage Transform: 210ms  ← ここが遅い、でも何が？
# stage Save:      30ms

# 目標（v19.8 以降）
fav run --profile=flamegraph src/pipeline.fav
# → flamegraph.svg を生成
# Transform の中で:
#   list_map: 80ms
#   transform_row: 70ms  ← さらに内部
#     compute_score: 45ms ← これがボトルネック
#     validate_email: 25ms
```

**フレームグラフの生成:**

```bash
# フレームグラフ（SVG）を生成
fav run --profile=flamegraph src/pipeline.fav
# → flamegraph.svg（ブラウザで開いて確認）

# テキスト形式（CI 向け）
fav run --profile=text src/pipeline.fav

# JSON 形式（外部ツール向け）
fav run --profile=json src/pipeline.fav > profile.json

# 特定 stage のみ計測
fav run --profile=flamegraph --profile-stage Transform src/pipeline.fav

# n 回実行して平均を取る
fav run --profile=flamegraph --profile-runs 10 src/pipeline.fav
```

**出力例（テキスト形式）:**

```
Profile: src/pipeline.fav (1 run, 285ms total)

stage LoadCsv   (  45ms,  15.8%)
  IO.read_file  (  40ms,  14.0%)
  csv.parse     (   5ms,   1.8%)

stage Transform ( 210ms,  73.7%)
  List.map      (  80ms,  28.1%)
  transform_row (  70ms,  24.6%)
    compute_score(  45ms,  15.8%) *** HOT PATH ***
    validate_email( 25ms,   8.8%)
  List.filter   (  60ms,  21.1%)

stage Save      (  30ms,  10.5%)
  IO.write_file (  30ms,  10.5%)
```

**実装内容:**

- `fav/src/backend/vm.rs`:
  - `ProfilingMode::Flamegraph / Text / Json` enum 追加
  - 関数呼び出し・リターン時に `CallStack` を記録
  - `std::time::Instant` で各 opcode の実行時間を計測（`--profile=flamegraph` 時のみ）

- `fav/src/profiler/`（新規ディレクトリ）:
  - `collector.rs`: 実行中のコールスタック・時間情報を収集
  - `flamegraph.rs`: 折り畳みスタックフォーマット → `inferno` でフレームグラフ SVG 生成
  - `report.rs`: テキスト / JSON レポート生成

- `Cargo.toml`:
  - `inferno 0.11` を依存に追加（フレームグラフ SVG 生成）

- `fav/src/driver.rs`:
  - `--profile=flamegraph / text / json` フラグ処理
  - `--profile-stage` / `--profile-runs` フラグ処理

- テスト: `v198000_tests`（5件）:
  - `version_is_19_8_0`
  - `profile_flamegraph_generates_svg`（`--profile=flamegraph` で SVG が生成される）
  - `profile_text_output`（`--profile=text` で関数別時間が出力される）
  - `profile_json_output`（`--profile=json` で JSON が出力される）
  - `profile_hot_path_detected`（最もコストの高い関数が "HOT PATH" としてマークされる）

**完了条件（PASS=5）:**
1. `fav run --profile=flamegraph` で `flamegraph.svg` が生成される
2. フレームグラフに関数レベルの呼び出しスタックが表示される
3. `--profile=text` で stage・関数別の時間・割合が出力される
4. `--profile=json` で機械可読な形式が出力される
5. 最もコストの高い関数がテキストレポートで視覚的に強調される

---

### v20.0.0 — Production Performance マイルストーン宣言

**テーマ**: v19.x シリーズの集大成。「本番で速い言語」への転換を宣言する。

**実装内容:**

- `Cargo.toml`: バージョンを `20.0.0` に更新

- `CHANGELOG.md`: v19.1.0〜v19.8.0 の全エントリ追加

- `README.md`:
  - 「現在の状態」を v20.0.0 に更新
  - Production Performance 達成を記載（streaming / AOT / Arrow / precompiled）
  - ベンチマーク結果（10GB CSV 処理速度 / Lambda コールドスタート時間）を掲載
  - バージョン履歴表に v19.1.0〜v20.0.0 エントリ追加

- `site/content/docs/`:
  - `performance/streaming.mdx` 新規作成（`#[streaming]` ガイド）
  - `performance/native-build.mdx` 新規作成（`fav build --target native` ガイド）
  - `performance/incremental.mdx` 新規作成（インクリメンタルコンパイルガイド）
  - `performance/arrow.mdx` 新規作成（Arrow 統合ガイド）
  - `performance/precompiled.mdx` 新規作成（`.favc` 事前コンパイルガイド）
  - `performance/profiling.mdx` 新規作成（フレームグラフガイド）

- `benchmarks/`（新規ディレクトリ）:
  - `10gb_csv.fav`: 10GB CSV 処理ベンチマーク
  - `lambda_coldstart.sh`: Lambda コールドスタート計測スクリプト
  - `results.md`: ベンチマーク結果の記録

- テスト: `v200000_tests`（5件）:
  - `version_is_20_0_0`
  - `changelog_has_v19_entries`（CHANGELOG に v19.x エントリが含まれる）
  - `readme_mentions_streaming`（README に streaming が記載されている）
  - `readme_mentions_native_build`（README に native build が記載されている）
  - `benchmarks_dir_exists`（`benchmarks/` ディレクトリが存在する）

**完了条件:**

| 確認項目 | 状態 |
|---|---|
| `#[streaming]` パイプラインが動作し大規模データを定常メモリで処理できる | [ ] |
| `fav build --target native` でネイティブバイナリが生成される | [ ] |
| 2 回目以降のコンパイルがインクリメンタルで高速化される | [ ] |
| 並列コンパイルが大規模プロジェクトで高速化される | [ ] |
| Arrow 形式でのデータ交換が動作する | [ ] |
| WASM サイズが v16.0.0 比 40% 以上削減される | [ ] |
| `fav run --precompiled` が動作する（Lambda コールドスタート高速化） | [ ] |
| `--profile=flamegraph` でフレームグラフ SVG が生成される | [ ] |
| `cargo test v200000` 全テストパス（5/5） | [ ] |
| `cargo test` 全件パス（リグレッションなし） | [ ] |

---

## 依存関係

```
v19.0.0（Type System Maturity）✅
    |
    v19.1.0（Streaming 評価）        ← 最優先（大規模データ対応の基盤）
    |
    v19.2.0（AOT / Cranelift）       v19.3.0（インクリメンタルコンパイル）  ← 並列実施可能
    |                                 |
    v19.4.0（並列コンパイル）← インクリメンタルのキャッシュ基盤を活用
    |
    v19.5.0（Arrow 統合）            v19.6.0（WASM 最適化）  ← 並列実施可能
    |                                 |
    v19.7.0（事前コンパイル）← AOT（v19.2）の成果を活用
    |
    v19.8.0（フレームグラフ）
    |
    v20.0.0（マイルストーン）
```

v19.1.0（Streaming）は最優先（他と独立していて、かつ最も体感インパクトが大きい）。
v19.2.0 と v19.3.0 は独立して並列実施可能。
v19.5.0（Arrow）と v19.6.0（WASM 最適化）は独立して並列実施可能。

---

## 新規 Cargo 依存（予定）

| Crate | 用途 | 追加バージョン |
|---|---|---|
| `cranelift-codegen 0.x` | AOT ネイティブコンパイル | v19.2.0 |
| `cranelift-module 0.x` | Cranelift モジュール管理 | v19.2.0 |
| `cranelift-object 0.x` | オブジェクトファイル生成 | v19.2.0 |
| `rayon 1.x` | 並列コンパイル | v19.4.0 |
| `petgraph 0.6` | 依存グラフ管理 | v19.3.0 |
| `arrow 53.x` | Apache Arrow 統合 | v19.5.0 |
| `parquet 53.x` | Parquet 読み書き | v19.5.0 |
| `bincode 2.x` | `.favc` バイナリシリアライズ | v19.7.0 |
| `inferno 0.11` | フレームグラフ SVG 生成 | v19.8.0 |

---

## 実装ノート

- **ストリーミング評価の stage 設計**: ストリーミング対応の stage は「chunk を受け取り chunk を返す」設計のままにする。stage の実装者はストリーミングを意識しない。`#[streaming]` はパイプライン側のアノテーション。
- **`#[stateful]` stage の状態管理**: chunk 間で状態を持つ stage（`RunningAvg` 等）は初回呼び出しで状態を初期化し、以降 chunk ごとに状態を更新する。最終 chunk の後に `finalize(state)` が呼ばれる。
- **Cranelift のバージョン安定性**: `cranelift-codegen` はバージョン間で API が変わりやすい。`wasmtime` と同じバージョンを使用することで安定性を確保する。
- **インクリメンタルキャッシュの無効化戦略**: コンテンツハッシュ（SHA-256）を使う。mtime は参考情報のみ（ファイルシステムによって信頼性が低いため）。
- **並列コンパイルの `Env` 共有**: 型チェック環境は `Arc<RwLock<GlobalTypeEnv>>` で共有。各ファイルのローカル `Env` は独立したスレッドで保持。書き込みは型定義が確定した後のみ（ファン-in フェーズ）。
- **Arrow 統合の段階的移行**: 既存の `Vec<Value>` ベースのコードは引き続き動作する。`#[arrow]` アノテーションを付けた stage のみ Arrow 形式に移行。強制移行は v21.x 以降（予定）。
- **WASM 最適化の trade-off**: `wasm-opt -O3` はビルド時間を増加させる。`fav build --wasm-opt=O1 / O2 / O3` でレベルを選択可能にする。CI では O1、リリースでは O3 を推奨。
- **`.favc` の互換性保証**: `fav` のバイナリバージョンと `.favc` のバイトコードバージョンを分けて管理。マイナーバージョン変更では `.favc` を再生成することを推奨（エラーで通知）。
- **フレームグラフの overhead**: `--profile=flamegraph` を有効にすると全関数呼び出しをフックするため、実行が 2〜5x 遅くなる。本番では使用しないよう README に明記。

---

## 参照ファイル

| ファイル | 目的 |
|---|---|
| `versions/roadmap-master.md` | v17.0〜v20.0 の全体戦略 |
| `versions/roadmap-v18.1-v19.0.md` | 直前ロードマップ（形式参照） |
| `fav/src/backend/vm.rs` | VM（Streaming opcode・プロファイリング追加対象） |
| `fav/src/backend/wasm.rs` | WASM バックエンド（最適化対象） |
| `fav/src/driver.rs` | CLI（compile / build / profile コマンド追加対象） |
| `fav/src/middle/compiler.rs` | IR コンパイラ（ストリーミング対応・Cranelift 変換対象） |
| `scripts/build-lambda-layer.sh` | Lambda デプロイスクリプト（事前コンパイル統合対象） |
| `site/src/wasm/` | Playground WASM（遅延ロード・Worker 対応対象） |
| `benchmarks/` | ベンチマーク新規追加対象 |
