# Favnir v1.5.0 仕様書 — CI/CD 統合 + 静的解析強化

作成日: 2026-05-08

> **テーマ**: explain 差分比較・fn 依存グラフ・ユーザー定義エフェクト・lint 強化により、
> チームの CI/CD ワークフローへの組み込みと静的解析の完成度を高める。
>
> **前提**: v1.4.0 完了（441 テスト通過）
>
> **先送り残件の解消**:
> - `fav explain diff`（v1.4.0 先送り）
> - `fav graph --focus fn`（v1.4.0 先送り）

---

## 1. スコープ概要

| Phase | テーマ | Done definition |
|---|---|---|
| 0 | バージョン更新 | `v1.5.0` がビルドされ HELP テキストに反映される |
| 1 | `fav explain diff` | 2つの explain.json を比較して差分を text/JSON で出力する |
| 2 | `fav graph --focus fn` | fn 呼び出し依存グラフを text/mermaid で出力する |
| 3 | ユーザー定義エフェクト | `effect Foo` 宣言構文・型検査・E052 が動く |
| 4 | `fav lint` 強化 | L005/L006/L007 の新ルールが動く |
| 5 | テスト・ドキュメント | 全テスト通過、langspec.md 更新 |

---

## 2. Phase 0 — バージョン更新

- `Cargo.toml`: `version = "1.5.0"`
- `main.rs`: HELP テキスト `v1.5.0`
- HELP テキストに `explain diff` サブコマンドを追記

---

## 3. Phase 1 — `fav explain diff`

### 3-1. CLI

```
fav explain diff old.json new.json
fav explain diff old.json new.json --format json
fav explain diff old.fav new.fav
fav explain diff old.fvc new.fvc
```

- デフォルト出力フォーマット: `text`
- `--format json`: 機械可読差分 JSON を stdout に出力
- ソース `.fav` / artifact `.fvc` を渡した場合はその場で `explain.json` 相当を生成して比較

### 3-2. text 出力フォーマット

```
--- old.json  (v1.4.0)
+++ new.json  (v1.5.0)

[fns]
+ fn newFn(Int) -> String
- fn oldFn() -> String
~ fn processUser(UserRow) -> String   return_type: Int -> String
~ fn saveAll(List<UserRow>) -> Int    effects: [] -> [Io]

[trfs]
~ trf ParseCsv: String -> List<UserRow>   (unchanged)

[types]
+ type AdminUser = { ... }
~ type UserRow   fields: +role: String

[summary]
  added:   fns=1, types=1
  removed: fns=1
  changed: fns=2
  breaking changes: oldFn removed, processUser signature changed
```

変更なしの場合: `No changes detected.`

### 3-3. JSON 出力フォーマット

```json
{
  "schema_version": "1.0",
  "from": "old.json",
  "to":   "new.json",
  "changes": {
    "fns": {
      "added":   [{ "name": "newFn", "signature": "Int -> String" }],
      "removed": [{ "name": "oldFn" }],
      "changed": [
        {
          "name": "processUser",
          "diffs": ["return_type: Int -> String", "effects: [] -> [Io]"]
        }
      ]
    },
    "trfs":   { "added": [], "removed": [], "changed": [] },
    "flws":   { "added": [], "removed": [], "changed": [] },
    "types":  { "added": [], "removed": [], "changed": [] },
    "effects_used": { "added": ["Io"], "removed": [] }
  },
  "summary": {
    "total_added":   2,
    "total_removed": 1,
    "total_changed": 2,
    "breaking_changes": ["oldFn removed", "processUser return_type changed"]
  }
}
```

### 3-4. 破壊的変更の定義

以下の変更を `breaking_changes` として分類する:

| 変更内容 | 判定 |
|---|---|
| fn/trf の削除 | 破壊的 |
| fn/trf の return_type 変更 | 破壊的 |
| fn/trf の effects 変更（追加・削除） | 破壊的 |
| fn/trf のパラメータ型変更 | 破壊的 |
| type フィールドの削除 | 破壊的 |
| fn/type/trf の追加 | 非破壊的 |
| type フィールドの追加 | 非破壊的 |

### 3-5. 実装方針

`driver.rs` に `cmd_explain_diff(from: &str, to: &str, format: &str)` を追加。

1. 引数の拡張子を判定して `explain_json_from_source` / `explain_json_from_artifact` / `parse_explain_json_file` を呼び分ける
2. 両方の `serde_json::Value` を比較する `diff_explain_json(from, to) -> ExplainDiff` を実装
3. `ExplainDiff` から text/JSON を生成

```rust
pub struct ExplainDiff {
    pub from_label: String,
    pub to_label: String,
    pub fn_changes:     CategoryDiff,
    pub trf_changes:    CategoryDiff,
    pub flw_changes:    CategoryDiff,
    pub type_changes:   CategoryDiff,
    pub effects_added:  Vec<String>,
    pub effects_removed: Vec<String>,
    pub breaking_changes: Vec<String>,
}

pub struct CategoryDiff {
    pub added:   Vec<serde_json::Value>,
    pub removed: Vec<serde_json::Value>,
    pub changed: Vec<ChangedEntry>,
}

pub struct ChangedEntry {
    pub name:  String,
    pub diffs: Vec<String>,
}
```

---

## 4. Phase 2 — `fav graph --focus fn`

### 4-1. CLI

```
fav graph src/main.fav --focus fn
fav graph src/main.fav --focus fn --format mermaid
fav graph src/main.fav --focus fn --entry processUser
fav graph src/main.fav --focus fn --depth 3
```

既存の `fav graph --focus flw` と `--focus fn` を並存させる（デフォルト: `flw`）。

### 4-2. text 形式

```
fn dependencies from: main
  main
  ├── importAll
  │   ├── fetchRows    !Db
  │   │   └── buildQuery
  │   └── validateRows
  │       └── checkInvariant
  └── reportSummary    !Io
      └── formatReport
```

- `!Effect` を各ノードの右に表示
- `--depth N` で表示深さを制限（0 = entry のみ）
- 循環参照は `[循環: fn名]` と表示

### 4-3. mermaid 形式

```
flowchart LR
  main --> importAll
  main --> reportSummary
  importAll --> fetchRows
  importAll --> validateRows
  fetchRows --> buildQuery
  reportSummary --> formatReport
  style fetchRows fill:#f8d7da
  style reportSummary fill:#f8d7da
```

- `!Db` / `!Io` 効果を持つ fn はノードを赤系（`#f8d7da`）で着色
- `--entry` で起点 fn を指定（デフォルト: `main`）

### 4-4. 実装方針

`driver.rs` の `render_graph_text` / `render_graph_mermaid` を拡張し、
`focus` が `"fn"` の場合に fn 呼び出しグラフを描画するパスを追加する。

fn 依存情報は `collect_fn_calls_from_ir(ir: &IRProgram) -> HashMap<String, Vec<String>>`
を `reachability.rs` に追加して取得する。

---

## 5. Phase 3 — ユーザー定義エフェクト

### 5-1. 概要

Favnir では現在 `Io`/`Db`/`File`/`Trace`/`Emit` のみが組み込みエフェクトとして有効で、
その他の名前は `Effect::Unknown(String)` として黙過されていた。

v1.5.0 では `effect Name` 宣言構文を追加し、
**宣言されていないエフェクトを型注釈で使用した場合に E052 を発生**させる。

### 5-2. 構文

```
effect Payment
effect Notification
effect Cache
```

- ファイルのトップレベルに `effect <Name>` と書く
- `Name` は PascalCase を推奨（違反は L007 で lint 警告）
- エフェクトは公開/非公開 (`public effect Foo`) を指定できる

```fav
public effect Payment
effect InternalAudit    // このファイル内限定
```

### 5-3. 型注釈での使用

宣言した effect はそのまま型注釈に使える:

```fav
public effect Payment

trf ChargeUser: UserRow -> Receipt !Payment = |user| {
    // ...
}
```

### 5-4. エラーコード

| コード | 条件 |
|---|---|
| E052 | 型注釈で未宣言のカスタムエフェクトを使用した（組み込み以外） |

**E052 の発生条件**:
組み込みエフェクト (`Io`, `Db`, `File`, `Trace`, `Emit`) 以外のエフェクト名を型注釈で使用し、
かつそのエフェクトがいずれのファイルにも `effect Name` で宣言されていない場合。

**注意**: v1.4.0 以前のコードとの後方互換性は `--compat` フラグで対応しない。
既存コードに未宣言エフェクトがある場合は警告（W012）として表示し、エラーには昇格しない（移行猶予）。

### 5-5. AST の変更

```rust
// ast.rs に追加
#[derive(Debug, Clone)]
pub struct EffectDef {
    pub visibility: Option<Visibility>,
    pub name: String,
    pub span: Span,
}
```

`Item` に `EffectDef(EffectDef)` バリアントを追加。

### 5-6. 語彙解析・パーサーの変更

- トークン: `TokenKind::Effect`（キーワード `"effect"`）を追加
- `parse_item` の先頭で `TokenKind::Effect` を検出して `parse_effect_def` を呼ぶ
- `parse_effect_def` → `EffectDef { visibility, name, span }` を返す

### 5-7. チェッカーの変更

- `first_pass` で `Item::EffectDef` を `effect_registry: HashSet<String>` に登録
- `check_effects(effects: &[Effect], span: Span)` を追加:
  - 各 effect が組み込みまたは `effect_registry` に存在するか確認
  - 存在しない場合: E052
- fn/trf/flw の型注釈チェック時に `check_effects` を呼ぶ

### 5-8. `fav explain` への追加

```json
{
  "custom_effects": [
    { "name": "Payment", "public": true },
    { "name": "InternalAudit", "public": false }
  ]
}
```

`ExplainJson` に `custom_effects: Vec<EffectEntry>` フィールドを追加。

---

## 6. Phase 4 — `fav lint` 強化

### 6-1. 新しいルール

| コード | 条件 | 対象 |
|---|---|---|
| L005 | 定義された `trf` / `flw` / `abstract trf` / `abstract flw` が一度も参照されていない | trf/flw 宣言 |
| L006 | `trf` 名が PascalCase でない | trf 宣言 |
| L007 | `effect` 名が PascalCase でない | effect 宣言 |

既存ルール（参考）:

| コード | 条件 |
|---|---|
| L002 | `bind` で束縛した変数が未使用 |
| L003 | fn 名が snake_case でない |
| L004 | type 名が PascalCase でない |

### 6-2. L005 の検出ロジック

`collect_trf_flw_uses(program: &Program) -> HashSet<String>` を `lint.rs` に追加:
- `Item::TrfDef` / `Item::AbstractTrfDef` / `Item::FlwDef` / `Item::AbstractFlwDef` / `Item::FlwBindingDef` の名前を「定義済みセット」に登録
- AST を走査して参照された名前（`TrfCall` / `FlwCall` / `FlwBindingDef.template` / `FlwBindingDef.bindings` 内の名前）を収集
- 未参照の定義に対して L005 を発行

**例外**:
- `public` 修飾子付きの定義は外部から参照される可能性があるため L005 対象外
- `main` から直接または間接的に到達可能な flw は L005 対象外

### 6-3. L006/L007 の検出ロジック

PascalCase 判定関数 `is_pascal_case(name: &str) -> bool` は既存の L004 ロジックを再利用。

### 6-4. CLI

```
fav lint src/main.fav
fav lint src/main.fav --warn-only
```

既存の `cmd_lint` に L005/L006/L007 を追加するだけで CLI 変更なし。

---

## 7. Phase 5 — テスト・ドキュメント

### 7-1. テスト要件

#### `fav explain diff` テスト

| テスト名 | 検証内容 |
|---|---|
| `explain_diff_no_changes` | 同じソース同士の diff で "No changes" が出る |
| `explain_diff_fn_added` | fn が追加された diff が text/JSON に反映される |
| `explain_diff_fn_removed` | fn が削除された diff が text/JSON に反映される |
| `explain_diff_fn_changed` | fn の return_type / effects 変更が diff に反映される |
| `explain_diff_breaking_changes` | 破壊的変更が `breaking_changes` に分類される |
| `explain_diff_json_valid` | `--format json` が有効な JSON を出力する |

#### `fav graph --focus fn` テスト

| テスト名 | 検証内容 |
|---|---|
| `graph_fn_text_shows_calls` | text 形式で fn 呼び出し依存が表示される |
| `graph_fn_mermaid_valid` | mermaid 形式が有効な flowchart を出力する |
| `graph_fn_depth_limit` | `--depth 1` で直接呼び出しのみが表示される |
| `graph_fn_cycle_safe` | 循環参照があってもパニックしない |

#### ユーザー定義エフェクトテスト

| テスト名 | 検証内容 |
|---|---|
| `effect_def_parses` | `effect Payment` がパースできる |
| `effect_def_registered` | チェッカーが `effect_registry` に登録する |
| `effect_custom_in_trf_ok` | 宣言済みエフェクトを trf 注釈で使える |
| `effect_unknown_e052` | 未宣言エフェクトで E052 が発生する |
| `effect_builtin_no_error` | 組み込みエフェクト (`Io`, `Db` など) は宣言不要 |
| `explain_json_custom_effects` | `custom_effects` フィールドが explain JSON に含まれる |

#### `fav lint` 強化テスト

| テスト名 | 検証内容 |
|---|---|
| `lint_l005_unused_trf` | 未参照の private trf に L005 が出る |
| `lint_l005_public_trf_ignored` | public trf に L005 が出ない |
| `lint_l005_unused_flw` | 未参照の private flw に L005 が出る |
| `lint_l006_trf_not_pascal` | trf 名が非 PascalCase で L006 が出る |
| `lint_l006_trf_pascal_ok` | trf 名が PascalCase で L006 が出ない |
| `lint_l007_effect_not_pascal` | effect 名が非 PascalCase で L007 が出る |

### 7-2. example ファイル

- `examples/custom_effects.fav` — `effect Payment` / `effect Notification` の宣言と使用例
- `examples/diff_demo/` — diff 比較用 v1/v2 のサンプル（`old.fav` / `new.fav`）

### 7-3. ドキュメント更新

- `versions/v1.5.0/langspec.md` を新規作成
  - `fav explain diff` コマンドと出力スキーマ
  - `fav graph --focus fn` コマンド
  - `effect Name` 宣言構文
  - E052 / W012 エラーコード
  - L005/L006/L007 lint ルール
- `README.md` に v1.5.0 セクション追加

---

## 8. 完了条件（Done Definition）

- [ ] `fav explain diff old.json new.json` が差分を text で出力する
- [ ] `fav explain diff old.fav new.fav --format json` が差分 JSON を出力する
- [ ] 破壊的変更が `breaking_changes` に正確に分類される
- [ ] `fav graph src/main.fav --focus fn` が fn 呼び出し依存ツリーを出力する
- [ ] `fav graph src/main.fav --focus fn --format mermaid` が mermaid 形式を出力する
- [ ] `effect Payment` がパースされチェッカーに登録される
- [ ] 未宣言エフェクトを使った型注釈で E052 が発生する
- [ ] 組み込みエフェクト（`Io`, `Db`, `File`, `Trace`, `Emit`）は宣言不要
- [ ] 未参照の private trf/flw に L005 が発生する
- [ ] trf 名の非 PascalCase に L006 が発生する
- [ ] effect 名の非 PascalCase に L007 が発生する
- [ ] v1.4.0 の全テスト（441）が引き続き通る
- [ ] `cargo build` で警告ゼロ
- [ ] `Cargo.toml` バージョンが `"1.5.0"`

---

## 9. 先送り一覧（v1.5.0 では対応しない）

| 制約 | バージョン |
|---|---|
| artifact の explain metadata 圧縮（gzip） | v2.0.0 |
| `PartialFlw` を型引数に取る関数 | v2.0.0 |
| `abstract flw` 継承 | v2.0.0 以降 |
| `abstract seq` / `abstract stage` / JSON キー renaming | v2.0.0 |
| Veltra との直接統合 | v2.0.0 以降 |
| `fav explain result`（Lineage Tracking） | v2.0.0 以降 |
| エフェクトの `use` による再エクスポート | v2.0.0 |
| エフェクト階層（`effect Foo extends Bar`） | v2.0.0 以降 |
| `fav lint` カスタムルールプラグイン | v2.0.0 以降 |
