# Favnir v1.4.0 仕様書 — `fav explain --format json` + `fav bundle` + 残件

作成日: 2026-05-07

> **テーマ**: コードの意味を機械可読 JSON で出力し、最小実行 artifact を生成する。
> v1.3.0 の先送り残件（動的注入・`fav graph`・`abstract trf` ジェネリック）を合わせて解消する。
>
> **設計ドキュメント**:
> - `dev/post-v1/roadmap/fav-explain-bundle.md`
> - `dev/post-v1/roadmap/favnir-graph-explain.md`
>
> **前提**: v1.3.0 完了

---

## 1. スコープ概要

| Phase | テーマ | Done definition |
|---|---|---|
| 0 | バージョン更新 | `v1.4.0` がビルドされ HELP テキストに反映される |
| 1 | `fav explain --format json` | `explain.json` が仕様スキーマに準拠した JSON を出力する |
| 2 | 到達可能性解析 | `included`/`excluded`/`effects_required` が正確に計算される |
| 3 | `fav bundle` | 到達可能なコードだけを含む `.fvc` + `manifest.json` が生成される |
| 4 | `fav bundle --explain` + artifact explain | 一回のパスで `.fvc`+`manifest.json`+`explain.json` が生成され、`fav explain dist/app.fvc` が動く |
| 5 | `fav graph` | abstract flw の構造と束縛をグラフ表示する `fav graph` コマンドが動く |
| 6 | trf 第一級値 + 動的注入 | `fn f(save: A -> B) -> flw X -> Y` のように trf を引数として渡して `flw` を組み立てられる |
| 7 | `abstract trf` ジェネリック | `abstract trf Fetch<T>: Id -> T? !Db` の型パラメータ付き宣言が動く |
| 8 | テスト・ドキュメント | 全テスト通過、langspec.md 更新 |

---

## 2. Phase 0 — バージョン更新

- `Cargo.toml`: `version = "1.4.0"`
- `main.rs`: HELP テキスト `v1.4.0`、`fav bundle` / `fav graph` コマンドを HELP に追加

---

## 3. Phase 1 — `fav explain --format json`

### 3-1. CLI 変更

```
fav explain src/main.fav                    // 既存 text 出力（変更なし）
fav explain src/main.fav --format json      // 新規: explain.json を stdout に出力
fav explain src/main.fav --format json --focus trfs   // trf のみ
fav explain src/main.fav --format json --focus flws   // flw のみ
fav explain src/main.fav --format json --focus types  // type のみ
fav explain src/main.fav --format json --focus fns    // fn のみ
```

`--format text` は既存の `fav explain` の動作と同義（デフォルト）。
`--format json` は stdout に JSON を出力する。

### 3-2. `explain.json` スキーマ（v1.4.0）

```json
{
  "schema_version": "1.0",
  "favnir_version": "1.4.0",
  "entry": "main",
  "source": "src/main.fav",
  "fns": [
    {
      "name": "main",
      "kind": "fn",
      "params": [],
      "return_type": "Unit",
      "effects": ["Io"],
      "calls": ["ImportUsers"],
      "reachable_from_entry": true
    }
  ],
  "trfs": [
    {
      "name": "ParseCsv",
      "kind": "trf",
      "input_type": "String",
      "output_type": "List<UserRow>",
      "effects": [],
      "calls": [],
      "reachable_from_entry": true
    },
    {
      "name": "FetchUser",
      "kind": "abstract_trf",
      "input_type": "UserId",
      "output_type": "User?",
      "effects": ["Db"],
      "reachable_from_entry": false
    }
  ],
  "flws": [
    {
      "name": "ImportUsers",
      "kind": "flw",
      "input_type": "String",
      "output_type": "Int",
      "effects": ["Db"],
      "steps": ["ParseCsv", "ValidateUser", "SaveUsers"],
      "reachable_from_entry": true
    },
    {
      "name": "DataPipeline",
      "kind": "abstract_flw",
      "type_params": ["Row"],
      "slots": [
        { "name": "parse", "input_type": "String", "output_type": "List<Row>", "effects": [] },
        { "name": "save",  "input_type": "List<Row>", "output_type": "Int", "effects": ["Db"] }
      ],
      "reachable_from_entry": false
    },
    {
      "name": "UserImport",
      "kind": "flw_binding",
      "template": "DataPipeline",
      "type_args": ["UserRow"],
      "bindings": { "parse": "ParseCsv", "save": "SaveUsers" },
      "input_type": "String",
      "output_type": "Int",
      "effects": ["Db"],
      "reachable_from_entry": true
    }
  ],
  "types": [
    {
      "name": "UserRow",
      "kind": "record",
      "fields": [
        { "name": "name", "type": "String" },
        { "name": "age",  "type": "Int" }
      ],
      "invariants": ["age > 0"]
    }
  ],
  "effects_used": ["Io", "Db"],
  "emits": [],
  "runes_used": []
}
```

**v2.0.0 移行注記**: `kind: "trf"` → `"stage"`、`kind: "flw"` / `"flw_binding"` → `"seq"`、
`"abstract_trf"` → `"abstract_stage"`、`"abstract_flw"` → `"abstract_seq"` にリネーム予定。
`"trfs"` / `"flws"` トップレベルキーも `"stages"` / `"seqs"` にリネーム予定。

### 3-3. `--focus` オプション

| 値 | 出力されるフィールド |
|---|---|
| `all`（デフォルト） | 全フィールド |
| `fns` | `fns` のみ |
| `trfs` | `trfs` のみ |
| `flws` | `flws` のみ |
| `types` | `types` のみ |

### 3-4. 実装方針

既存の `ExplainPrinter` を拡張し `render_json(program, checker_result, focus) -> String` を追加する。
serde_json（既存依存）で JSON シリアライズ。

`reachable_from_entry` はこの Phase では全て `true` 固定（Phase 2 で実装する到達可能性解析を注入）。

---

## 4. Phase 2 — 到達可能性解析

### 4-1. 共有基盤

`src/middle/reachability.rs` を新規作成:

```rust
pub struct ReachabilityResult {
    pub included:         HashSet<String>,  // エントリから到達可能な fn/trf/flw 名
    pub excluded:         HashSet<String>,  // 定義されているが未到達
    pub effects_required: Vec<String>,      // included 関数の全 effects
    pub emits:            Vec<String>,      // included 関数の全 emits
}

pub fn reachability_analysis(
    entry: &str,
    program: &IRProgram,
) -> ReachabilityResult;
```

### 4-2. アルゴリズム

BFS / DFS で IR の呼び出しグラフを走査:

1. `entry`（通常 `"main"`）を起点にキューに積む
2. 各関数の本体で参照される `IRExpr::Global(name)` / `IRExpr::Call { func }` を追跡
3. 到達した名前を `included` に追加してキューに追加
4. 定義済み全名前のうち `included` に含まれない → `excluded`
5. `included` 内の全関数の `effects` を集約 → `effects_required`

### 4-3. explain.json への反映

Phase 1 の `reachable_from_entry` フィールドを Phase 2 の解析結果で埋める。

---

## 5. Phase 3 — `fav bundle`

### 5-1. CLI

```
fav bundle src/main.fav
fav bundle src/main.fav -o dist/app.fvc
fav bundle src/main.fav -o dist/app.fvc --entry main
fav bundle src/main.fav -o dist/app.fvc --manifest
fav bundle src/main.fav --manifest --explain
```

デフォルト出力: `dist/<basename>.fvc`。

### 5-2. 動作

1. ソースをパース・型検査
2. `reachability_analysis("main", ir)` で `included` を計算
3. `included` に含まれる関数定義だけを含む `.fvc` を生成
4. `--manifest` → `manifest.json` を生成
5. `--explain` → `explain.json` を生成（Phase 4 で統合）

### 5-3. `manifest.json` スキーマ

```json
{
  "schema_version": "1.0",
  "favnir_version": "1.4.0",
  "entry":           "main",
  "source":          "src/main.fav",
  "artifact":        "dist/app.fvc",
  "artifact_size":   4096,
  "built_at":        "2026-05-07T00:00:00Z",
  "rune": {
    "name":    "myapp",
    "version": "1.0.0"
  },
  "included":         ["main", "ImportUsers", "ParseCsv", "ValidateUser", "SaveUsers"],
  "excluded":         ["UnusedHelper"],
  "effects_required": ["Io", "Db"],
  "emits":            [],
  "runes_used":       []
}
```

### 5-4. `cmd_bundle` の実装（`driver.rs`）

```rust
pub fn cmd_bundle(
    file: &str,
    out: Option<&str>,
    entry: &str,
    manifest: bool,
    explain: bool,
);
```

---

## 6. Phase 4 — `fav bundle --explain` + artifact explain

### 6-1. 一回パス統合

`fav bundle --explain` は同一の解析パスから `.fvc` + `manifest.json` + `explain.json` を生成する。
別途 `fav explain` を呼ぶ必要がない。

### 6-2. `.fvc` フォーマットへの metadata セクション追加

`artifact.rs` の `.fvc` バイナリに explain metadata セクションを追加:

```
[FVC magic + header]
[existing function bytecode sections]
[EXPLAIN_METADATA section]
  - 4 bytes: section length
  - N bytes: explain.json の UTF-8 バイト列（gzip 非圧縮）
```

`FvcArtifact` に `explain_json: Option<String>` フィールドを追加。
書き込み時: `--explain` フラグがある場合のみセクションを書く。
読み込み時: セクションが存在すれば `explain_json` に格納。

### 6-3. `fav explain dist/app.fvc`

`fav explain` コマンドが `.fvc` ファイルを受け取った場合:

1. `FvcArtifact` を読み込む
2. `explain_json` フィールドがあれば stdout に出力
3. なければ `--format text` でスケルトン出力（名前と型のみ）

```
fav explain dist/app.fvc            // text スケルトン or stored explain
fav explain dist/app.fvc --format json  // stored JSON (metadata なければ error)
```

---

## 7. Phase 5 — `fav graph`（v1.3.0 残件）

### 7-1. CLI

```
fav graph src/main.fav
fav graph src/main.fav --format text       // デフォルト: ASCII グラフ
fav graph src/main.fav --format mermaid    // Mermaid flowchart
fav graph src/main.fav --focus flw UserImport
```

### 7-2. text 形式（デフォルト）

```
flw UserImport (DataPipeline<UserRow>)
  [parse]    <- ParseUserCsv    :  String -> List<UserRow>
  [validate] <- ValidateUser    :  UserRow -> UserRow!
  [save]     <- SaveUsers       :  List<UserRow> -> Int !Db

  String -> Int !Db
```

### 7-3. mermaid 形式

```
fav graph src/main.fav --format mermaid
```

出力例:
```
flowchart LR
  parse["parse\nString → List&lt;UserRow&gt;"] --> validate["validate\nUserRow → UserRow!"]
  validate --> save["save\nList&lt;UserRow&gt; → Int !Db"]
  style parse fill:#d4edda
  style save fill:#f8d7da
```

### 7-4. 実装方針

`src/driver.rs` に `cmd_graph(file, format, focus)` を追加。
既存の `AbstractFlwDef` / `FlwBindingDef` の情報を使って描画。
外部ライブラリ不要（テキスト/mermaid は文字列生成のみ）。

---

## 8. Phase 6 — trf 第一級値 + 動的注入（v1.3.0 残件）

### 8-1. 概要

v1.3.0 で先送りになった「関数引数から trf を注入して `flw` を組み立てる」パターンを実装する。

```fav
// trf 値を引数として受け取り、flw を返す関数
fn make_pipeline(save: UserRow -> Int !Io) -> flw String -> Int !Io {
    bind p <- DataPipeline<UserRow> {
        parse    <- ParseCsvUser
        validate <- ValidateUser
        save     <- save         // 引数から注入
    }
    p
}

// 本番・テスト用の異なる実装を注入
bind prod_pipe <- make_pipeline(SaveUserDb)
bind test_pipe <- make_pipeline(SaveUserMock)
```

### 8-2. 型システムの変更

trf 型シグネチャ `A -> B !Fx` は既に `Type::Trf(A, B, Fx)` として存在する。
関数パラメータとして宣言できるように、`TypeExpr` レベルで `A -> B !Fx` 形式のパラメータ型を受け入れる。

**パーサー**: `fn f(save: UserRow -> Int !Io)` — パラメータ型として `TypeExpr::Trf(input, output, effects)` をパースする。
（`->` を含む型式のパース。既存の型式パーサーを拡張）

### 8-3. `flw` 束縛での変数スロット

`FlwBindingDef.bindings: Vec<(String, String)>` の 2 番目要素を「グローバル名 OR ローカル変数名」に拡張:

```rust
pub enum SlotImpl {
    Global(String),  // グローバルな trf 名（既存）
    Local(String),   // ローカル変数（関数引数 or bind で束縛された値）
}

pub struct FlwBindingDef {
    // ...
    pub bindings: Vec<(String, SlotImpl)>,
}
```

### 8-4. チェッカーの変更

`check_flw_binding_def` でスロット実装が `Local` の場合、ローカル変数の型を型環境から解決する。
型照合ロジックは既存と同じ（E048: 型不一致）。

### 8-5. コンパイラ / VM の変更

#### VMValue 拡張

```rust
// vm.rs の VMValue に追加
TrfRef(String),  // trf 関数への参照（グローバル名）
```

#### IR 拡張

```rust
// ir.rs
IRExpr::TrfRef(name: String),  // グローバル trf への参照値
```

#### コンパイラの変更

`compile_flw_binding_def` でスロット実装が `SlotImpl::Local(name)` の場合:
- `IRExpr::Local(name)` から trf 参照を取得
- IR 呼び出し: `IRExpr::CallTrfRef { trf_local: name, arg: ... }`

#### VM の変更

```rust
// vm.rs
IRExpr::CallTrfRef { trf_local, arg } => {
    let trf_ref = self.load_local(trf_local)?;
    match trf_ref {
        VMValue::TrfRef(fn_name) => self.call_fn(&fn_name, vec![self.eval(arg)?]),
        _ => Err("expected trf reference"),
    }
}
```

---

## 9. Phase 7 — `abstract trf` ジェネリック型パラメータ（v1.3.0 残件）

### 9-1. 概要

```fav
abstract trf Fetch<T>:    Id   -> T?   !Db
abstract trf Transform<A, B>: A -> B
abstract trf Validate<T>: T   -> T!
```

型パラメータ付き `abstract trf` を `abstract flw` のスロット型として使う。

### 9-2. パーサーの変更

```
abstract trf Name "<" type_params ">" ":" TypeExpr "->" TypeExpr effects
```

`AbstractTrfDef` に `type_params: Vec<String>` を追加:

```rust
pub struct AbstractTrfDef {
    pub name:        String,
    pub type_params: Vec<String>,  // 追加
    pub input_ty:    TypeExpr,
    pub output_ty:   TypeExpr,
    pub effects:     Vec<Effect>,
    pub span:        Span,
}
```

### 9-3. スロット型での使用

```fav
abstract flw Pipeline<Row, Out> {
    fetch:     Id         -> Row?    !Db    // 非ジェネリック
    transform: Row        -> Out!           // Out は Pipeline の型パラメータ
    save:      List<Out>  -> Int     !Db
}
```

`abstract trf Fetch<T>` をスロット型として直接指定することも可能:

```fav
abstract flw Pipeline<Row> {
    fetch: Fetch<Row>     // Fetch<T> に Row を代入
    save:  Save<Row>
}
```

### 9-4. チェッカーの変更

`abstract_trf_registry` にジェネリック `AbstractTrfDef` を登録。
`check_flw_binding_def` でスロット型が `AbstractTrf<T>` の場合、型引数を代入して照合。

---

## 10. Phase 8 — テスト・ドキュメント

### テスト要件

#### explain JSON テスト

| テスト名 | 検証内容 |
|---|---|
| `explain_json_valid_schema` | `--format json` が有効な JSON を出力する |
| `explain_json_has_all_sections` | `fns`/`trfs`/`flws`/`types` が全て含まれる |
| `explain_json_focus_trfs` | `--focus trfs` で `trfs` のみが出力される |
| `explain_json_reachable_flag` | `reachable_from_entry` が正確に設定される |
| `explain_json_kinds` | trf/abstract_trf/flw/flw_binding/abstract_flw の kind が正確 |

#### 到達可能性テスト

| テスト名 | 検証内容 |
|---|---|
| `reachability_simple` | main → fn A → fn B の到達性が正確 |
| `reachability_excluded` | 未使用 fn が `excluded` に入る |
| `reachability_effects_required` | included 関数の effects が集約される |

#### bundle テスト

| テスト名 | 検証内容 |
|---|---|
| `bundle_produces_smaller_artifact` | bundle 後の .fvc が unreachable 関数を含まない |
| `bundle_manifest_json` | `--manifest` で `manifest.json` が生成される |
| `bundle_included_excludes_dead_code` | `included` が到達可能な関数のみを含む |

#### fav graph テスト

| テスト名 | 検証内容 |
|---|---|
| `graph_text_shows_flw_structure` | text 形式で flw の構造が表示される |
| `graph_mermaid_valid_syntax` | mermaid 形式が有効な flowchart 構文を出力する |

#### 動的注入テスト

| テスト名 | 検証内容 |
|---|---|
| `dynamic_injection_type_ok` | `fn f(save: A -> B)` の型検査が通る |
| `dynamic_injection_type_e048` | 型不一致の引数で E048 が出る |
| `dynamic_injection_exec_ok` | 注入した trf が実行時に正しく呼ばれる |

#### `abstract trf` ジェネリックテスト

| テスト名 | 検証内容 |
|---|---|
| `abstract_trf_generic_parse` | `abstract trf Fetch<T>: Id -> T? !Db` がパースできる |
| `abstract_trf_generic_binding_ok` | ジェネリック abstract trf を slot 型として使った束縛が通る |

#### example ファイル

- `examples/bundle_demo.fav` — `fav bundle` の動作確認用（到達不能コードを含む）
- `examples/dynamic_inject.fav` — 動的注入パターンの実例

### ドキュメント更新

- `versions/v1.4.0/langspec.md` を新規作成
  - `fav explain --format json` スキーマ仕様
  - `fav bundle` / `--manifest` / `--explain` コマンド
  - `fav graph` コマンド
  - trf 第一級値の構文・制約
  - `abstract trf` ジェネリック構文
- `README.md` に v1.4.0 セクション追加

---

## 11. 完了条件（Done Definition）

- [x] `fav explain main.fav --format json` が仕様スキーマに準拠した JSON を出力する
- [x] `fav bundle main.fav -o dist/app.fvc --explain` が `.fvc` + `manifest.json` + `explain.json` を生成する
- [x] `included` / `excluded` が正確に到達可能性を反映する
- [x] `effects_required` が到達可能な関数の effects を網羅する
- [x] `fav explain dist/app.fvc` が artifact から explain を出力する
- [x] `fav graph` が abstract flw の構造を text/mermaid で出力する
- [x] `fn f(save: A -> B) -> flw X -> Y { ... }` が型検査・実行できる
- [x] `abstract trf Fetch<T>: Id -> T? !Db` が定義でき、スロット型として使える
- [x] v1.3.0 の全テストが引き続き通る
