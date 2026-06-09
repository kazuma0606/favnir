# Favnir v12.7.0 仕様書

Date: 2026-06-08
Theme: `fav doc --builtins` + `fav explain <code>`

---

## 概要

v12.6.0 で TLS 対応・エラー詳細化が完了した。
v12.7.0 は「AI フレンドリー強化」フェーズの第一弾。

問題: AI が Favnir コードを書く際に
「`Csv.parse_raw` が何を返すか」「`Postgres.execute_raw` の引数は何か」を
知る手段がない。vm.rs を読まない限り型シグネチャが不明。

本バージョンで組み込み Primitive の型シグネチャを
`fav doc --builtins` / `fav explain <code>` で参照可能にする。

---

## 機能 1: `fav doc --builtins`

### 目的

組み込み Primitive の型シグネチャを人間向け Markdown で出力する。

### 実行形式

```bash
fav doc --builtins           # Markdown を stdout に出力
fav doc --builtins --out docs/  # docs/builtins.md に書き出し
```

### 出力例（Markdown）

```markdown
# Favnir Built-in Primitives

## IO

### IO.println
`(value: String) -> Unit !IO`

標準出力に文字列を出力する（改行付き）。

### IO.read_file_raw
`(path: String) -> Result<String, String> !IO`

ファイルを UTF-8 文字列として読み込む。失敗時は Err を返す。

---

## Csv

### Csv.parse_raw
`(text: String, sep: String, header: Bool) -> Result<List<Record>, String> !IO`

CSV テキストを解析してレコードのリストを返す。
header=true の場合、1 行目をフィールド名として使用する。

---

## Postgres

### Postgres.execute_raw
`(sql: String, params: String) -> Result<Unit, String> !Postgres`

SQL を実行する（SELECT 以外）。params は JSON 配列文字列。
```

### 対象 Namespace

| Namespace | 主な関数 |
|---|---|
| IO | println / print / read_file_raw / write_file_raw / read_line / timestamp / argv / make_dir_raw |
| Csv | parse_raw / to_string_raw |
| Schema | to_json_array / adapt / validate |
| Json | encode_raw / decode_raw / pretty_raw |
| Gen | uuid / uuid_v7 / nano_id / one_raw / hint_one_raw / seed |
| AWS | s3_get_raw / s3_put_raw / sqs_send_raw / sqs_receive_raw |
| Postgres | execute_raw / query_raw / infer_table_raw |
| Snowflake | execute_raw / query_raw |
| Http | get_raw / post_raw / serve_raw |
| Llm | complete_raw / chat_raw / extract_raw |

---

## 機能 2: `fav doc --builtins --format json`

### 目的

AI ツールがコード生成前に「この namespace に何があるか」を
JSON パースして参照できるようにする。

### 実行形式

```bash
fav doc --builtins --format json
```

### 出力例（JSON）

```json
[
  {
    "namespace": "Csv",
    "name": "Csv.parse_raw",
    "signature": "(text: String, sep: String, header: Bool) -> Result<List<Record>, String>",
    "effects": ["!IO"],
    "returns_result": true,
    "description": "CSV テキストを解析してレコードのリストを返す。header=true の場合、1 行目をフィールド名として使用する。"
  },
  {
    "namespace": "Postgres",
    "name": "Postgres.execute_raw",
    "signature": "(sql: String, params: String) -> Result<Unit, String>",
    "effects": ["!Postgres"],
    "returns_result": true,
    "description": "SQL を実行する（SELECT 以外）。params は JSON 配列文字列。"
  }
]
```

### 仕様

- `--format json` で stdout に JSON 配列を出力
- `--format markdown` または省略でデフォルト Markdown
- `--out <file>` と組み合わせ可能
- exit 0 で必ず終了（エラーなし）

---

## 機能 3: `fav explain <code>`

### 目的

AI がエラーコードを受け取った際に
コンパイラ自身に意味・修正方法を問い合わせられるようにする。

### 実行形式

```bash
fav explain E0018
fav explain W006
```

### 出力例（E0018）

```
E0018: Variable already bound

Favnir では変数は一度だけ束縛できます（イミュータブル）。
同一スコープで bind x を 2 回書くことはできません。

修正例:
  誤: bind x <- step1()
      bind x <- step2(x)   ← E0018

  正: bind x  <- step1()
      bind x2 <- step2(x)  ← OK

関連: W006（Result を bind _ で捨てる）
```

### 出力例（W006）

```
W006: Discarding Result value

bind _ <- Postgres.execute_raw(...) のように、
エフェクトが返す Result を bind _ で捨てても警告が出ます。
失敗がサイレントに通過し、パイプラインが誤動作します。

修正例:
  誤: bind _ <- Postgres.execute_raw(sql, params)

  正: chain _ <- Postgres.execute_raw(sql, params)
  または:
      match Postgres.execute_raw(sql, params) {
        Ok(_)  => ...
        Err(e) => ...
      }

関連: E0018（変数の再束縛）
```

### 対象コード

E0001〜E0018（コンパイルエラー）、W001〜W007（警告）。
未知のコードは `unknown error code: XXXX` と出力して exit 1。

---

## 実装方針

### `BuiltinPrimitive` 構造体（driver.rs）

```rust
#[derive(serde::Serialize)]
struct BuiltinPrimitive {
    namespace:      &'static str,
    name:           &'static str,   // "Csv.parse_raw"
    signature:      &'static str,   // "(text: String, ...) -> Result<...>"
    effects:        Vec<&'static str>, // ["!IO"]
    returns_result: bool,
    description:    &'static str,
}
```

静的テーブルとして定義（vm.rs の `@doc` アノテーション方式は次バージョン以降）。

### `cmd_doc_builtins(format: &str, out: Option<&str>)` 関数

- `format == "json"` → `serde_json::to_string_pretty` → stdout / out ファイル
- `format == "markdown"` (デフォルト) → namespace ごとにグループ化して Markdown 出力

### `cmd_explain_code(code: &str)` 関数

- 静的マップ `HashMap<&str, &str>` でコード → 説明文を保持
- `EXPLAIN_MAP.get(code)` が `None` なら exit 1

### main.rs の変更

```
// fav doc --builtins [--format json] [--out path]
Some("doc") => {
    if args.iter().any(|a| a == "--builtins") {
        // cmd_doc_builtins(format, out)
    } else {
        // 既存の cmd_doc(path, out_dir)
    }
}

// fav explain <code>  (既存の --lineage フロー以外)
Some("explain") => {
    // 既存の --lineage 分岐の後に追加
    // cmd_explain_code(code)
}
```

---

## テストケース

| テスト名 | 内容 |
|---|---|
| `doc_builtins_json_is_array` | `--format json` の出力が JSON 配列 |
| `doc_builtins_csv_parse_raw` | JSON に `Csv.parse_raw` エントリが存在する |
| `doc_builtins_postgres_returns_result` | `Postgres.execute_raw` の `returns_result` が `true` |
| `doc_builtins_markdown_has_namespace_header` | Markdown に `## IO` / `## Csv` ヘッダが含まれる |
| `explain_e0018_output` | `E0018` の説明に "already bound" が含まれる |
| `explain_w006_output` | `W006` の説明に "Result" が含まれる |
| `explain_unknown_code_exits_1` | 未知コードで exit 1 |
| `version_is_12_7_0` | `CARGO_PKG_VERSION == "12.7.0"` |

---

## 完了条件

- [ ] `fav doc --builtins` で Markdown が出力される（全 Namespace 網羅）
- [ ] `fav doc --builtins --format json` で JSON 配列が出力される
- [ ] `fav explain E0018` で人間が読める説明が出る
- [ ] `fav explain W006` で修正方法が出る
- [ ] 未知コードで exit 1
- [ ] 全テストケース通過
- [ ] `cargo test` 全通過

---

## 非目標

- vm.rs への `@doc` アノテーション付与（手動で静的テーブルを定義する）
- サイト (`site/content/docs/primitives/`) へのドキュメント自動組み込み（次バージョン以降）
- `fav doc --builtins --namespace Csv` での絞り込みフィルタ
- エラーコードの多言語対応
