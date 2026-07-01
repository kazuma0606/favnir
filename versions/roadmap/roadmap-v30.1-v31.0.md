# Roadmap v30.1.0 〜 v31.0.0 — Real-World Readiness

Date: 2026-07-01

## 目標

v30.0「Ecosystem Maturity」でコミュニティと Rune が揃った。
しかし「実際のプロジェクト」として .fav を使ったとき、端から端まで動くかどうかは
まだ検証されていない。

このフェーズでは **「実案件で .fav が動く」** を徹底的に検証し、
詰まるポイントをすべて修正する。

**ドッグフードの対象**: CSV → Postgres ETL（マルチファイル・型付き・テスト付き）

> **Real-World Readiness の定義（本プロジェクト固有）**
> 「`fav new --template postgres-etl my-project` で生成されたプロジェクトが、
>  `fav check` / `fav run` / `fav test` すべてで通り、
>  実データ（CSV 1000 行）を Postgres に書き込めること」

**完了条件（最終テスト）:**

```bash
# 1. 全 Rust テストが通る
cargo test

# 2. ビルド軽量化の確認
du -sh fav/target/  # debug=0 適用後

# 3. テンプレート生成 + 動作確認
fav new --template postgres-etl dogfood-proj
cd dogfood-proj
fav check
fav test
fav run src/main.fav

# 4. テンプレート一覧
fav new --list

# 5. ドッグフードパイプライン動作確認（手動）
#    実データ CSV → Postgres が動くこと
```

---

## 設計決定事項

| 項目 | 決定 |
|---|---|
| ビルド軽量化の方針 | `[profile.dev] debug = 0` + `split-debuginfo = "off"` を Cargo.toml に追加 |
| テンプレート構成 | `postgres-etl` を 4 ファイル構成に更新（types / stages / validators / main） |
| fav.toml プロジェクトモード | `src = "src"` で複数 .fav ファイルを自動検出（既存実装を検証・修正） |
| ドッグフード対象 | CSV 読み込み → バリデーション → Postgres 書き込み → 結果サマリー |
| クリーンアップタイミング | v31.0.0 マイルストーン宣言と同時に実施 |
| 破壊的変更 | なし |

---

## バージョン計画

### v30.1 — ビルド軽量化

**テーマ**: `cargo build` 後の `target/` を軽量化する。

**背景**: スプリントを重ねるごとに `target/debug/` が 40GB+ に膨らむ。
`debug = 0` でデバッグシンボルを無効化し、30〜40% 削減する。

**実装内容**:

```toml
# fav/Cargo.toml に追加
[profile.dev]
debug = 0
split-debuginfo = "off"
```

```toml
# fav/.cargo/config.toml を新規作成
[build]
# Windows: lld-link でリンク高速化（任意）
# [target.x86_64-pc-windows-msvc]
# linker = "lld-link"
```

完了条件:
- `cargo clean && cargo build` 後の `du -sh target/` がベースラインより削減されていること
- テスト全件通過

---

### v30.2 — postgres-etl テンプレート v2（4ファイル構成）

**テーマ**: マルチファイルプロジェクトの実用テンプレートを作る。

**現状**: `postgres-etl` テンプレートは `src/main.fav` + `src/pipeline.fav` の 2 ファイル構成で中身が薄い。

**目標構成**:

```
my-project/
├── fav.toml                      [project] + [postgres] 設定
├── src/
│   ├── types.fav                 型定義（Row 型 + バリデーション型）
│   ├── validators.fav            バリデーションロジック
│   ├── stages.fav                パイプラインステージ（Load/Transform/Validate/Write）
│   └── main.fav                  エントリポイント + エラーハンドリング
├── tests/
│   └── pipeline_test.fav         テストファイル
└── README.md
```

`types.fav`:
```favnir
// 入力 CSV の生データ型
type RawRow = {
    id:     String
    name:   String
    amount: String
    date:   String
}

// バリデーション済みの型
type ValidRow = {
    id:     Int
    name:   String
    amount: Float
    date:   String
}

// エラー型
type RowError = {
    row_index: Int
    field:     String
    message:   String
}
```

`stages.fav`:
```favnir
import runes/postgres
import src/types
import src/validators

// CSV ファイルから生データを読み込む
stage LoadCsv: String -> List<RawRow> !IO = |path| {
    bind lines <- IO.read_lines(path)
    bind rows  <- lines
        |> List.drop(1)
        |> List.map(parse_csv_row)
        |> Result.all
    Result.ok(rows)
}

// バリデーションを適用して型安全なデータに変換
stage ValidateRows: List<RawRow> -> List<ValidRow> !IO = |rows| {
    bind results <- rows
        |> List.map(|row| validators.validate_row(row))
    bind valid   <- Result.all(results)
    Result.ok(valid)
}

// Postgres に書き込む
stage WriteToDb: List<ValidRow> -> Int !Postgres = |rows| {
    bind conn  <- Postgres.connect(env("DATABASE_URL"))
    bind count <- rows
        |> List.map(|row| Postgres.execute(conn,
            "INSERT INTO records (id, name, amount, date) VALUES ($1, $2, $3, $4)",
            [row.id, row.name, row.amount, row.date]))
        |> Result.all
    Result.ok(List.length(count))
}
```

完了条件:
- 生成プロジェクトで `fav check` がエラーなく通る
- README に `fav run` / `fav test` の手順が書かれている

---

### v30.3 — マルチファイルプロジェクト E2E 検証

**テーマ**: `fav.toml` プロジェクトモードが複数 .fav ファイルで正しく動くか検証する。

**検証対象**:

1. `fav check`（プロジェクト全体の型チェック）
2. `fav run`（プロジェクトのエントリポイント実行）
3. `fav test`（プロジェクト内全 test ブロックの実行）
4. `fav lint`（プロジェクト全体の lint）
5. `fav fmt --check`（プロジェクト全体のフォーマット確認）

各コマンドで発見したバグを修正し、Rust テストに追加する。

完了条件:
- 上記 5 コマンドが v30.2 で生成したテンプレートプロジェクトで全て通る
- 発見したバグが修正された Rust テストとして追加されている

---

### v30.4 — Rune import マルチファイル動作検証

**テーマ**: 複数 .fav ファイルから同じ Rune を import する場合の動作を検証・修正する。

**検証シナリオ**:

```
src/
  types.fav      → import runes/postgres（型参照のみ）
  stages.fav     → import runes/postgres（関数呼び出し）
  validators.fav → （Rune import なし）
  main.fav       → import runes/postgres（接続確立）
```

複数ファイルが同じ Rune を import した場合の:
- 型チェック整合性
- 実行時の Rune 初期化（二重初期化がないか）
- `fav check` のエラー表示

完了条件:
- 上記シナリオで `fav check` / `fav run` が正しく動作する
- 発見したバグが修正されテストに追加されている

---

### v30.5 — ドッグフード用サンプル実装（CSV → Postgres）

**テーマ**: 実データを使った完全なパイプラインを `examples/` に追加する。

**実装するパイプライン**:

```
examples/csv-to-postgres/
├── fav.toml
├── src/
│   ├── types.fav
│   ├── validators.fav
│   ├── stages.fav
│   └── main.fav
├── data/
│   └── sample.csv          （1000 行のサンプルデータ）
├── tests/
│   └── pipeline_test.fav
└── README.md
```

パイプラインの処理内容:
1. `LoadCsv` — CSV を読み込み `RawRow` のリストに変換
2. `ValidateRows` — 型変換・バリデーション（欠損値・範囲チェック）
3. `LogStats` — 処理件数・エラー件数をログ出力
4. `WriteToDb` — バリデーション済みデータを Postgres に書き込み
5. `ReportSummary` — 処理結果サマリーを stdout に出力

完了条件:
- `examples/csv-to-postgres/` が完全な状態で存在する
- `fav check examples/csv-to-postgres/src/main.fav` が通る
- `fav test examples/csv-to-postgres/` が通る（DB 接続なしのモック込み）
- README に「30 分で動かす」手順が書かれている

---

### v30.6 — fav test プロジェクト統合

**テーマ**: `fav test` でプロジェクト全体の全テストを一括実行できるようにする。

**現状**: `fav test <file>` で単一ファイルのテストは動く。
`fav test`（引数なし）でプロジェクト全体を走るか要検証。

**実装内容**:
- `fav test` 引数なし時に `fav.toml` の `src` ディレクトリ以下の全 `.fav` ファイルをスキャン
- 全 `test { }` ブロックと `test_group { }` を実行
- `--filter <pattern>` フラグでテスト名フィルタリング
- 失敗時に `FAILED: <file>:<test_name>` 形式で報告

完了条件:
- `fav test` でプロジェクト全体のテストが実行される
- `fav test --filter validate` でフィルタリングが動作する
- Rust テスト 1 件追加

---

### v30.7 — fav run エラー時スタックトレース改善

**テーマ**: 実行時エラーの表示品質を上げる。

**現状の問題**:
```
runtime error: index out of bounds
```
情報が少なく、どのステージ・どの行で起きたか分からない。

**目標**:
```
runtime error: index out of bounds
  in stage ValidateRows at src/stages.fav:34:5
  called from src/main.fav:12:3 (EtlPipeline)
  |
34|   List.nth(rows, i)
  |   ^^^^^^^^^^^^^^^^^
  = ヒント: List.nth は範囲外アクセスで失敗します。List.get を使うと Option<T> で安全に取得できます。
```

**実装内容**:
- VM の実行時エラーに `stage_name` / `file` / `line` を付与
- エラー表示時にソース行を表示（span 情報を活用）
- よく起こるエラーに `= ヒント:` を追加

完了条件:
- 実行時エラーにステージ名とファイル位置が表示される
- Rust テスト 1 件追加

---

### v30.8 — fav new --list コマンド

**テーマ**: 使えるテンプレートを一覧表示できるようにする。

**実装**:
```bash
$ fav new --list
利用可能なテンプレート:

  script          シンプルなスクリプト（1ファイル）
  pipeline        基本パイプライン（seq/par）
  lib             ライブラリ（公開関数のみ）
  postgres-etl    PostgreSQL ETL（4ファイル構成）[推奨]
  etl-csv-to-db   CSV → DB ETL
  api-gateway     HTTP API ゲートウェイ
  lambda-scheduled スケジュール実行 Lambda ジョブ
  distributed-etl 分散並列 ETL パイプライン

使用例:
  fav new my-project --template postgres-etl
```

完了条件:
- `fav new --list` でテンプレート一覧が表示される
- Rust テスト 1 件追加

---

### v30.9 — ドッグフード発見修正

**テーマ**: v30.5 のドッグフードで発見した問題をすべて修正する。

v30.5 のサンプル実装・v30.3 の E2E 検証で見つかった問題の修正版。
具体的な内容は v30.5 実装時に確定する。

想定される修正候補:
- String match で `!IO` エフェクトエラーが出る場合の対処
- マルチファイルでの型推論の境界ケース
- `fav test` のテスト失敗メッセージの改善
- Rune import が深いディレクトリで解決できない場合

完了条件:
- v30.3〜v30.6 で残った既知バグが修正されている
- テスト全件通過

---

## v31.0 — Real-World Readiness マイルストーン宣言

**完了条件:**

| コンポーネント | 完了基準 |
|---|---|
| ビルド軽量化 | `debug=0` 適用・`target/` サイズ削減確認 |
| postgres-etl テンプレート v2 | 4 ファイル構成・`fav check` / `fav run` / `fav test` 全通過 |
| マルチファイル E2E | 5 コマンド（check/run/test/lint/fmt）全通過 |
| Rune import マルチファイル | 複数ファイルからの同一 Rune import が正常動作 |
| ドッグフードサンプル | `examples/csv-to-postgres/` が動作・README 完備 |
| fav test プロジェクト統合 | 引数なし `fav test` でプロジェクト全体が実行される |
| エラー表示改善 | 実行時エラーにステージ名と位置が表示される |
| fav new --list | テンプレート一覧が表示される |
| ドッグフード修正 | 発見したバグが修正済み |

**最終テスト:**

```bash
# 全 Rust テスト通過
cargo test

# テンプレート生成 + 全コマンド確認
fav new --template postgres-etl smoke-test
cd smoke-test
fav check && fav test && fav lint

# テンプレート一覧確認
fav new --list

# ビルドサイズ確認
du -sh target/
```

**★ クリーンアップ実施:**

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
- 前フェーズ: `versions/roadmap/roadmap-v29.1-v30.0.md`
- 次フェーズ: `versions/roadmap/roadmap-v31.1-v32.0.md`
- サンプル: `examples/csv-to-postgres/`
