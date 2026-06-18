# Favnir v10.3.0 Tasks

Date: 2026-06-04
Theme: Effect::Snowflake 追加（8 ファイル更新）

---

## Phase A: ast.rs + parser.rs + fmt.rs + lineage.rs

- [x] A-1: `ast.rs` — `Effect` 列挙体に `Snowflake` 追加（`Llm` の直後）
- [x] A-2: `parser.rs` — `"Snowflake"` → `Effect::Snowflake` 解析追加（`"Llm"` ブランチ直後）
- [x] A-3: `fmt.rs` — `Effect::Snowflake => Some("!Snowflake".to_string())` 追加
- [x] A-4: `lineage.rs` — `Snowflake => "!Snowflake".into()` 追加
- [x] A-5: `cargo build` 通過確認

---

## Phase B: driver.rs

- [x] B-1: 表示用変換に `Snowflake => "!Snowflake".into()` 追加（`Llm` 直後）
- [x] B-2: 短縮名変換に `ast::Effect::Snowflake => "Snowflake".into()` 追加（`Llm` 直後）

---

## Phase C: ast_lower_checker.rs + reachability.rs

- [x] C-1: `ast_lower_checker.rs` — `ast::Effect::Snowflake => "Snowflake".to_string()` 追加
- [x] C-2: `reachability.rs` — `Effect::Snowflake` ブランチ追加

---

## Phase D: checker.rs

- [x] D-1: builtin NS ホワイトリスト 1 箇所目に `"Snowflake"` 追加（〜line 1256）
- [x] D-2: builtin NS ホワイトリスト 2 箇所目に `"Snowflake"` 追加（〜line 2124）
- [x] D-3: effects ホワイトリスト 2 箇所に `"Snowflake"` 追加（〜line 4513/4525）
- [x] D-4: `require_snowflake_effect`（E0314）関数を `require_llm_effect` 直後に追加
- [x] D-5: `("Snowflake", "execute_raw")` / `("Snowflake", "query_raw")` 型シグネチャ追加

---

## Phase E: error_catalog.rs

- [x] E-1: E0314「undeclared !Snowflake effect」エントリを E0313 直後に追加

---

## Phase F: テスト追加

- [x] F-1: `driver.rs` 末尾に `v10300_tests` モジュール追加（3 件）
  - [x] F-1a: `snowflake_execute_requires_effect` — !Snowflake なしで E0314
  - [x] F-1b: `snowflake_execute_with_effect_ok` — !Snowflake ありで E0314 なし
  - [x] F-1c: `snowflake_lineage_shows_effect` — lineage に `!Snowflake` が含まれる
- [x] F-2: `cargo test v10300` — 3 件通過

---

## Phase G: 完了処理

- [x] G-1: `fav/Cargo.toml` version → `"10.3.0"`
- [x] G-2: `fav/self/cli.fav` の `run_version` → `"10.3.0"`
- [x] G-3: `fav check --legacy-check self/compiler.fav` — エラーなし
- [x] G-4: `fav check self/checker.fav` — エラーなし
- [x] G-5: `cargo test` — 全件通過（目標 1267 件）
- [x] G-6: 本ファイル完了チェック
- [x] G-7: `memory/MEMORY.md` に v10.3.0 完了を記録
- [x] G-8: commit

---

## 完了条件

| 条件 | 状態 |
|---|---|
| `stage Foo: String -> String !Snowflake = ...` が型チェックを通る | |
| `!Snowflake` 未宣言の fn で `Snowflake.execute_raw` を呼ぶと E0314 | |
| `fav explain --lineage` 出力に `!Snowflake` が表示される | |
| `cargo test v10300` — 3 件通過 | |
| `cargo test` 全件通過 | |
| `fav check self/checker.fav` エラーなし | |
