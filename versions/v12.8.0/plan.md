# Favnir v12.8.0 実装計画

Date: 2026-06-09

---

## Phase A — `cmd_scaffold` 基盤（driver.rs）

### A-1: `ScaffoldArgs` パラメータ構造体を定義

```rust
struct ScaffoldArgs<'a> {
    in_type:    &'a str,  // --in  (default: "String")
    out_type:   &'a str,  // --out-type (default: "String")
    effect:     Option<&'a str>, // --effect (default: Some("IO")), None = --no-effect
    out_file:   Option<&'a str>, // --out <file>
    stages:     Vec<&'a str>,    // --stages "A,B,C"
}
```

### A-2: `scaffold_stage(name, args) -> String` を実装

出力例（`--effect IO` のデフォルト）:

```
// MyStage: String -> String !IO
// TODO: implement MyStage
public stage MyStage: String -> String !IO = |input| {
    bind _result <- IO.println(input)
    input
}
```

- `--no-effect` 時はエフェクト注釈を省略
- `--in` / `--out-type` で入出力型を変更
- コメント行に型シグネチャを含める

### A-3: `scaffold_seq(name, stages: &[&str]) -> String` を実装

デフォルト stages = `["Load", "Transform", "Save"]`。

出力例:

```
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

- Load / Save ステージには `!IO` を付与
- Transform ステージにはエフェクトなし
- stage 数は `--stages` で指定

### A-4: `scaffold_postgres_etl() -> String` を実装

chain ベースの Postgres ETL 雛形を返す。
`bind _ <-` ではなく `chain _ <-` を使い Result を正しく伝播する。

### A-5: `scaffold_rune(name) -> String` を実装

```
// MyLib rune — public API
// Usage: import MyLib from "path/to/mylib"

public fn hello(name: String) -> String {
    $"Hello from MyLib, {name}!"
}
```

### A-6: `write_scaffold(content: &str, out: Option<&str>)` を実装

`out = Some(path)` → `std::fs::write(path, content)`、`None` → `print!("{}", content)`

---

## Phase B — `cmd_scaffold` コマンドハンドラ（driver.rs）

### B-1: `cmd_scaffold(sub: &str, args: &[String])` を実装

```rust
pub fn cmd_scaffold(sub: &str, args: &[String]) {
    match sub {
        "stage" => { ... }
        "seq"   => { ... }
        "postgres-etl" => { ... }
        "rune"  => { ... }
        _ => { eprintln!("unknown scaffold template: {}", sub); process::exit(1); }
    }
}
```

引数パース:
- `args.get(0)` = `<Name>`（stage / seq / rune のみ）
- `--in <Type>` / `--out-type <Type>` / `--effect <Effect>` / `--no-effect` / `--out <file>` / `--stages <A,B,C>` を解析

---

## Phase C — `fav new --template postgres-etl` 対応（driver.rs）

### C-1: `try_cmd_new` に `postgres-etl` テンプレートを追加

既存の `script` / `pipeline` / `lib` テンプレートと並べて:

```rust
"postgres-etl" => {
    create_dir_all(&format!("{}/src", dir))?;
    // fav.toml
    write(format!("{}/fav.toml", dir), POSTGRES_ETL_TOML)?;
    // src/pipeline.fav
    write(format!("{}/src/pipeline.fav", dir), scaffold_postgres_etl())?;
    // src/main.fav
    write(format!("{}/src/main.fav", dir), POSTGRES_ETL_MAIN_FAV)?;
}
```

### C-2: `POSTGRES_ETL_TOML` 定数を定義

```toml
[project]
name    = "<dir-name>"
version = "0.1.0"
edition = "2026"
src     = "src"

[postgres]
# url     = "${DATABASE_URL}"
sslmode = "require"
```

### C-3: `POSTGRES_ETL_MAIN_FAV` 定数を定義

```favnir
// entry point
public stage Main: String -> String !IO !Postgres = |_args| {
    bind args <- IO.argv()
    bind path <- List.first(args)
    EtlPipeline(path)
}
```

---

## Phase D — main.rs の変更

### D-1: `Some("scaffold")` 分岐を追加

```rust
Some("scaffold") => {
    let sub = args.get(2).map(String::as_str).unwrap_or("");
    let rest = if args.len() > 3 { &args[3..] } else { &[] };
    cmd_scaffold(sub, rest);
}
```

### D-2: `fav new` の `--template` 引数パースに `postgres-etl` を追加

既存の `try_cmd_new` の template match に追加するだけ。

---

## Phase E — テスト追加（driver.rs）

`v12800_tests` モジュール:

```rust
#[cfg(test)]
mod v12800_tests {
    fn scaffold_stage_contains_public()  { ... }  // "public stage" が含まれる
    fn scaffold_stage_has_effect()       { ... }  // "!IO" が含まれる
    fn scaffold_stage_no_effect()        { ... }  // --no-effect 時エフェクトなし
    fn scaffold_seq_has_pipe()           { ... }  // "|>" が含まれる
    fn scaffold_seq_default_stages()     { ... }  // Load / Transform / Save が含まれる
    fn scaffold_seq_custom_stages()      { ... }  // --stages "A,B,C" でカスタム
    fn scaffold_postgres_etl_uses_chain(){ ... }  // "chain" が含まれる
    fn scaffold_rune_contains_fn()       { ... }  // "public fn" が含まれる
    fn new_template_postgres_etl_creates_dir() { ... }  // ディレクトリ作成確認
    fn version_is_12_8_0()               { ... }
}
```

---

## Phase F — バージョン更新・コミット

- `fav/Cargo.toml` version → `"12.8.0"`
- `cargo test` 全通過確認
- `git commit -m "feat: v12.8.0 — fav scaffold <template>"`
- `git push`

---

## 実装上の注意

### 1. `fav check` との連携（テストケース `scaffold_*_compiles`）

spec.md のテストケースには `fav check` を通すことが要件として挙がっているが、
Rust unit test 内から `fav check` プロセスを起動するのは複雑。
代わりに以下の方針をとる:

- **生成文字列の構造テスト**（`contains_public` 等）を unit test で行う
- `fav check` E2E テストは `cargo test` とは別に `scripts/` で行う（v12.9.0 の CI 整備で対応予定）

### 2. `try_cmd_new` の既存パターンを踏襲

`fav new` のテンプレート生成は既存コードに `match "postgres-etl"` を追加するだけ。
ディレクトリ名から `name` を取り出す既存ロジックを再利用する。

### 3. `scaffold_seq` の Load/Transform/Save の `!IO` 判定

- `stages[0]`（最初）と `stages[last]`（最後）に `!IO` を付与
- 中間ステージにはエフェクトなし
- stage が 1 つの場合は `!IO` を付与

### 4. `scaffold_postgres_etl` の chain 使用

`bind _ <- IO.write_file_raw(...)` ではなく `chain _ <- IO.write_file_raw(...)` を使い、
Result エラーを正しく伝播する（spec.md テストケース `scaffold_postgres_etl_uses_chain`）。
