# Favnir v12.8.0 仕様書

Date: 2026-06-09
Theme: `fav scaffold <template>` — 正しい雛形生成

---

## 概要

v12.7.0 で Primitive リファレンスが整備された。
v12.8.0 は「AI フレンドリー強化」フェーズの第二弾。

問題: AI が Favnir を書く際に「それっぽいが文法的に誤った」コードから始めると、
最初のコンパイルエラーから修正するコストが高い。
正しい雛形があればそこに肉付けするだけで済む。

本バージョンで `fav scaffold <template>` コマンドを追加し、
AI・人間が正しい構文から始められるスキャフォールディングを提供する。

---

## 現状確認

`fav new <name> --template <script|pipeline|lib>` はプロジェクト全体を生成する（v10.0.0）。
v12.8.0 で追加する `fav scaffold` はプロジェクトではなく
**コードスニペット（stdout 出力 or 単一ファイル）** を生成する。

---

## 機能 1: `fav scaffold stage <Name>`

### 目的

stage の正しい雛形を生成する。
AI が「型シグネチャ / エフェクト / 本体」のパターンを間違えないようにする。

### 実行形式

```bash
fav scaffold stage MyStage                    # stdout に出力
fav scaffold stage MyStage --out src/my.fav   # ファイルに書き出し
fav scaffold stage MyStage --effect IO        # !IO 付き
fav scaffold stage MyStage --in String --out-type Int  # 型指定
```

### 出力例

```favnir
// MyStage: String -> String !IO
// TODO: implement MyStage
public stage MyStage: String -> String !IO = |input| {
    bind _result <- IO.println(input)
    input
}
```

### 引数

| フラグ | デフォルト | 説明 |
|---|---|---|
| `--in <Type>` | `String` | 入力型 |
| `--out-type <Type>` | `String` | 出力型 |
| `--effect <Effect>` | `IO` | エフェクト（なしにするには `--no-effect`）|
| `--no-effect` | — | エフェクトなし |
| `--out <file>` | stdout | 出力先ファイル |

---

## 機能 2: `fav scaffold seq <Name>`

### 目的

seq パイプラインの正しい雛形を生成する。

### 実行形式

```bash
fav scaffold seq MyPipeline
fav scaffold seq MyPipeline --stages "Load,Transform,Save"
```

### 出力例（デフォルト 3 stage）

```favnir
// MyPipeline: String -> String
// 3-stage sequential pipeline
public stage Load: String -> String !IO = |input| {
    // TODO: load data
    input
}

public stage Transform: String -> String = |data| {
    // TODO: transform data
    data
}

public stage Save: String -> String !IO = |data| {
    // TODO: save results
    data
}

public seq MyPipeline = Load |> Transform |> Save
```

### 引数

| フラグ | デフォルト | 説明 |
|---|---|---|
| `--stages <A,B,C>` | `Load,Transform,Save` | カンマ区切り stage 名 |
| `--out <file>` | stdout | 出力先ファイル |

---

## 機能 3: `fav scaffold postgres-etl`

### 目的

Postgres ETL の完全な正しいパターンを生成する。
fav2py E2E デモで判明した「bind _ で Result を捨てる」「seq が fail-fast でない」
という問題を最初から回避した、ベストプラクティスのテンプレートを提供する。

### 実行形式

```bash
fav scaffold postgres-etl
fav scaffold postgres-etl --out src/pipeline.fav
```

### 出力例

```favnir
// Postgres ETL pipeline — best practice template (v12.8.0)
// Uses chain for error propagation and seq for fail-fast execution.

public stage LoadCsv: String -> String !IO = |path| {
    match IO.read_file_raw(path) {
        Ok(text) => text
        Err(e)   => e
    }
}

public stage InsertRows: String -> String !IO !Postgres = |csv_text| {
    match Csv.parse_raw(csv_text, ",", true) {
        Ok(rows) => {
            bind json <- Schema.to_json_array(rows, "Row")
            bind sql  <- $"INSERT INTO my_table (data) VALUES ('{json}')"
            match Postgres.execute_raw(sql, "[]") {
                Ok(_)  => $"inserted {List.length(rows)} rows"
                Err(e) => e
            }
        }
        Err(e) => e
    }
}

public stage SaveResult: String -> String !IO = |result| {
    chain _ <- IO.write_file_raw("/tmp/result.txt", result)
    result
}

public seq EtlPipeline = LoadCsv |> InsertRows |> SaveResult
```

---

## 機能 4: `fav scaffold rune <Name>`

### 目的

Rune（ライブラリ）の雛形を生成する。

### 実行形式

```bash
fav scaffold rune MyLib
```

### 出力例

```favnir
// MyLib rune — public API
// Usage: import MyLib from "path/to/mylib"

public fn hello(name: String) -> String {
    $"Hello from MyLib, {name}!"
}
```

---

## 機能 5: `fav new --template postgres-etl <dir>`

既存の `fav new` に `postgres-etl` テンプレートを追加する。

### 実行形式

```bash
fav new my-etl --template postgres-etl
```

### 生成ファイル

```
my-etl/
  fav.toml          # [project] + [postgres] sslmode = "require"
  src/
    pipeline.fav    # postgres-etl scaffold と同内容
    main.fav        # fav run エントリポイント
```

### fav.toml の内容

```toml
[project]
name    = "my-etl"
version = "0.1.0"
edition = "2026"
src     = "src"

[postgres]
# url     = "${DATABASE_URL}"
sslmode = "require"
```

---

## テストケース

| テスト名 | 内容 |
|---|---|
| `scaffold_stage_compiles` | `scaffold stage` 出力が `fav check` を通ること |
| `scaffold_stage_contains_public` | 出力に `public stage` が含まれること |
| `scaffold_seq_compiles` | `scaffold seq` 出力が `fav check` を通ること |
| `scaffold_seq_has_pipe` | 出力に `\|>` が含まれること |
| `scaffold_postgres_etl_compiles` | `postgres-etl` 出力が `fav check` を通ること |
| `scaffold_postgres_etl_uses_chain` | 出力に `chain` が含まれること（Result 廃棄なし） |
| `scaffold_rune_compiles` | `scaffold rune` 出力が `fav check` を通ること |
| `new_template_postgres_etl_creates_dir` | `fav new --template postgres-etl` でディレクトリが作成されること |
| `version_is_12_8_0` | `CARGO_PKG_VERSION == "12.8.0"` |

---

## 完了条件

- [ ] `fav scaffold stage <Name>` でコンパイル可能な stage 雛形が出る
- [ ] `fav scaffold seq <Name>` でコンパイル可能な seq パイプライン雛形が出る
- [ ] `fav scaffold postgres-etl` で chain ベースの ETL 雛形が出る
- [ ] `fav scaffold rune <Name>` で rune 雛形が出る
- [ ] `fav new --template postgres-etl` でプロジェクトが作成される
- [ ] 全テストケース通過
- [ ] `cargo test` 全通過

---

## 非目標

- インタラクティブなプロンプト（`fav scaffold` は非対話型）
- 既存ファイルへの部分挿入（`--out` で上書き or stdout のみ）
- Terraform テンプレートの生成（v12.9.0 以降）
- `fav scaffold --list` でテンプレート一覧表示（シンプル実装で十分）
