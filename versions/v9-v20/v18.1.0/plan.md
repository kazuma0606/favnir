# v18.1.0 — 実装計画

## 方針

- エフェクト推論は `checker.rs` の拡張のみ（新 AST ノード・新 opcode 不要）
- `infer_effects` は fn ボディのネームスペース呼び出しを走査するシンプルな実装
- 推移的推論は `fn_effects_registry` への 2 パス処理で実現
- 明示エフェクト宣言は現状のまま動作（後方互換）
- テストは `fav_check_src` ヘルパー（既存）または `infer_effects` を直接呼ぶ形で実装

---

## 実装ステップ

### Step 1: `fav/src/middle/checker.rs` — `EffectSet` 型と `infer_effects` 追加

**変更内容:**

1. `EffectSet` 型エイリアスを追加:
   ```rust
   pub type EffectSet = std::collections::HashSet<Effect>;
   ```

2. ネームスペース → Effect マッピング関数:
   ```rust
   fn ns_to_effect(ns: &str) -> Option<Effect> {
       match ns {
           "Postgres" | "Db"              => Some(Effect::Db),
           "IO"                           => Some(Effect::IO),
           "S3" | "Sqs" | "Dynamo" | "Aws" => Some(Effect::AWS),
           "Kafka" | "Rskafka"            => Some(Effect::Kafka),
           "Snowflake"                    => Some(Effect::Snowflake),
           "BigQuery"                     => Some(Effect::BigQuery),
           "Http" | "Ureq"                => Some(Effect::Http),
           "Llm"                          => Some(Effect::Llm),
           _                              => None,
       }
   }
   ```

3. `collect_effects_from_expr(expr: &Expr, out: &mut EffectSet)` — 再帰的に expr を走査:
   - `Expr::Apply { func, args }` を見て
   - `func` が `Expr::FieldAccess { obj: Expr::Var(ns), .. }` なら `ns_to_effect(ns)` を出力に追加
   - すべての子 expr に再帰

4. `pub fn infer_effects_fn(fn_def: &FnDef) -> EffectSet`:
   - fn_def.body の全 stmt / expr を走査
   - 直接エフェクトを収集
   - 他の fn 呼び出しは fn 名を記録しておく（Step 2 の推移的推論で使用）

**注意点:**
- 既存の `check_fn_def` 内の effect チェックロジックとの重複を避ける
- `infer_effects_fn` は pure（副作用なし）関数として実装し、checker state を変更しない

---

### Step 2: `fav/src/middle/checker.rs` — `fn_effects_registry` と推移的推論

**変更内容:**

1. `Checker` struct に追加:
   ```rust
   pub fn_effects_registry: HashMap<String, EffectSet>,
   ```

2. `register_item_signatures` の `Item::FnDef` 処理で:
   - `infer_effects_fn(fn_def)` を呼んで直接エフェクトを計算
   - `fn_effects_registry.insert(fn_name, effects)` で登録

3. 推移的伝播（fixpoint iteration）:
   - `fn propagate_transitive_effects(registry: &mut HashMap<String, EffectSet>, call_graph: &HashMap<String, Vec<String>>)`
   - `call_graph`: fn 名 → 呼び出す fn 名のリスト
   - fixpoint に達するまで繰り返す（通常 2〜3 ラウンドで収束）

4. `check_fn_def` にて:
   - `fn.effects.is_empty()` の場合は `fn_effects_registry` から推論済みエフェクトを使用
   - エフェクト宣言ありの場合は整合性検査（E0336 / W010）

**実装上の簡略化:**
- 推移的推論は完全な fixpoint の代わりに、最大 10 ラウンドのイテレーション上限を設ける
- 相互再帰（A が B を呼び、B が A を呼ぶ）は Union で処理（循環検出不要）

---

### Step 3: `fav/src/driver.rs` — `--show-effects` と W010

**変更内容:**

1. `cmd_check` に `--show-effects` フラグを追加:
   ```rust
   pub fn cmd_check(path: &str, show_types: bool, show_effects: bool, json: bool)
   ```

2. `show_effects` が true の場合:
   - `checker.fn_effects_registry` を取得
   - 各 fn について `"fn {name}  inferred: {effects}"` を表示
   - エフェクトなしの場合は `"inferred: (none)"` を表示

3. `main.rs` で `--show-effects` フラグを解析して渡す

4. W010 警告の出力:
   - 既存 `warnings` Vec に `Warning::W010 { fn_name, excess_effect }` を追加
   - `format_warnings` で表示

---

### Step 4: `self/checker.fav` — Favnir 実装追加

**変更内容:**

```fav
fn ns_to_effect_str(ns: String) -> String {
  match ns {
    "Postgres" => "!Db"
    "IO"       => "!IO"
    "S3"       => "!AWS"
    "Snowflake" => "!Snowflake"
    "Http"     => "!Http"
    _          => ""
  }
}

fn infer_effects_from_stmts(stmts: List<Stmt>) -> List<String> {
  bind raw <- [ns_to_effect_str(ns) | stmt <- stmts, ns <- collect_ns_calls(stmt)]
  bind filtered <- [e | e <- raw, String.length(e) > 0]
  List.dedup(filtered)
}
```

> Favnir セルフホストチェッカーは現状の実装を壊さず、新関数を追加するだけ。

---

### Step 5: `fav/src/driver.rs` — `v181000_tests` 追加

`v180000_tests` の `version_is_18_0_0` テストを削除し、新しい 5 件を追加:

#### テスト実装方針

- `infer_effects_fn` を直接呼ぶ unit test として実装
- Favnir ソースをパース → FnDef を取得 → `infer_effects_fn` → EffectSet を確認
- `fav_check_src` ヘルパーが使える場合はそちらも使用

```rust
mod v181000_tests {
    #[test]
    fn version_is_18_1_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("\"18.1.0\""), "Cargo.toml should have version 18.1.0");
    }

    #[test]
    fn effect_inference_db() {
        // Postgres.query_raw を含む fn に !Db が推論される
        let src = r#"
fn load() -> Result<Int, String> {
  bind rows <- Postgres.query_raw("SELECT 1", [])
  Result.ok(0)
}
"#;
        let effects = infer_effects_from_src(src, "load");
        assert!(effects.contains(&Effect::Db), "!Db should be inferred");
    }

    #[test]
    fn effect_inference_multi() {
        // Postgres.* と IO.* 両方 → !Db !IO
        let src = r#"
fn load_and_log() -> Result<Int, String> {
  bind rows <- Postgres.query_raw("SELECT 1", [])
  bind _    <- IO.println("done")
  Result.ok(0)
}
"#;
        let effects = infer_effects_from_src(src, "load_and_log");
        assert!(effects.contains(&Effect::Db));
        assert!(effects.contains(&Effect::IO));
    }

    #[test]
    fn effect_inference_pure() {
        // 副作用なし fn のエフェクトは空集合
        let src = r#"
fn add(a: Int, b: Int) -> Int {
  a + b
}
"#;
        let effects = infer_effects_from_src(src, "add");
        assert!(effects.is_empty(), "pure fn should have no effects");
    }

    #[test]
    fn effect_inference_transitive() {
        // !Db を持つ fn を呼ぶ fn にも !Db が推論される
        let src = r#"
fn fetch() -> Result<Int, String> {
  bind rows <- Postgres.query_raw("SELECT 1", [])
  Result.ok(0)
}
fn wrap() -> Result<Int, String> {
  fetch()
}
"#;
        let effects = infer_effects_from_src(src, "wrap");
        assert!(effects.contains(&Effect::Db), "transitive !Db should be inferred");
    }
}
```

---

### Step 6: バージョン更新

- `fav/Cargo.toml`: `18.0.0` → `18.1.0`
- `cargo build` で `Cargo.lock` 更新

---

### Step 7: ドキュメント

`site/content/docs/language/effect-inference.mdx` 新規作成:
- エフェクト推論の概要（現状との比較）
- ネームスペース → エフェクト マッピング表
- 明示宣言との共存方法
- `fav check --show-effects` の使い方
- W010 警告の説明

---

## 依存関係グラフ

```
Step 1 (EffectSet / infer_effects)
    |
Step 2 (fn_effects_registry / 推移的推論)  ← Step 1 必須
    |
Step 3 (driver.rs --show-effects / W010)   ← Step 2 必須
Step 4 (checker.fav)                        ← Step 1 と並列可
    |
Step 5 (v181000_tests)                     ← Steps 1-4 すべて完了後
    |
Step 6 (バージョン更新)
Step 7 (ドキュメント)                       ← Step 6 と並列可
```

---

## テストのヘルパー関数

```rust
// driver.rs の v181000_tests 内ヘルパー
fn infer_effects_from_src(src: &str, fn_name: &str) -> EffectSet {
    use crate::frontend::parser::parse_program;
    use crate::middle::checker::infer_effects_fn;

    let prog = parse_program(src).expect("parse failed");
    let fn_def = prog.items.iter()
        .filter_map(|item| if let Item::FnDef(f) = item { Some(f) } else { None })
        .find(|f| f.name == fn_name)
        .expect(&format!("fn {} not found", fn_name));
    infer_effects_fn(fn_def)
}
```

---

## 注意事項

- `infer_effects_fn` は既存の `check_fn_def` の副作用チェックを**置き換えない**。
  チェックは既存ロジックを残しつつ、推論結果を「宣言なしの場合のデフォルト」として使う。
- `stage` の `fn` 等価変換は既存の `compile_stage` が行うため、`infer_effects_fn` は `FnDef` に変換後の stage にも自動的に適用される。
- `checker.fav` の変更は Rust 実装と並行して追加するが、テストは Rust 側のみで行う（`checker.fav` はセルフホスト用）。
- W010 は警告（ビルド失敗にならない）。E0336 はエラー（ビルド失敗）。
