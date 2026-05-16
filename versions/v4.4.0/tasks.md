# Favnir v4.4.0 タスクリスト — Gen Rune 2.0

作成日: 2026-05-17

---

## Phase 0: バージョン更新

- [ ] `fav/Cargo.toml` の version を `"4.4.0"` に変更
- [ ] `fav/src/main.rs` のヘルプ文字列・バージョン表示を `4.4.0` に更新

---

## Phase 1: VM プリミティブ追加（`fav/src/backend/vm.rs`）

### 1-A: ヘルパー関数・定数定義

- [ ] 日本語姓名リスト `JA_LAST_NAMES` / `JA_FIRST_NAMES` を定数として追加
- [ ] 英語名リスト `EN_FIRST_NAMES` / `EN_LAST_NAMES` を定数として追加
- [ ] `DESCRIPTIONS` 固定文例リストを追加
- [ ] `thread_local! { static HINT_ID_COUNTER: RefCell<HashMap<String, u64>> }` を追加
- [ ] `gen_hint_value_for_field(field_name, field_type, rng) -> String` ヘルパー関数を実装
  - フィールド名パターンの優先順位: uuid > id/*_id > email > name > phone > *_at/*_datetime > *_date > price/amount/*_fee > age > count > url > zip > address > description/body/content > status > is_*/has_*/flag > その他
  - `id` / `*_id`: `HINT_ID_COUNTER` を型名キーでインクリメントして返す（"1", "2", ...）
  - `email` / `*_email`: `format!("user{}@example.com", counter)` 形式
  - `*_name` / `full_name` / `name`: `JA_LAST_NAMES` + `JA_FIRST_NAMES` からランダム選択
  - `*_at` / `*_datetime`: `"2026-{:02}-{:02}T{:02}:{:02}:{:02}Z"` 形式（範囲内のランダム日時）
  - `*_date`: `"2026-{:02}-{:02}"` 形式
  - `price` / `amount` / `*_fee` / `*_price`: `100.0`〜`99999.0` の乱数
  - `age`: `20`〜`80` の整数
  - `status`: "active" / "inactive" / "pending" からランダム選択
  - `is_*` / `has_*` / `flag`: "true" または "false"
  - その他 Int: ランダム整数; Float: ランダム浮動小数点; String: ランダム文字列
- [ ] `gen_hint_one_for_type(type_name, rng, type_metas) -> Result<VMValue, VMError>` ヘルパー関数を実装
  - `type_metas.get(type_name)` → フィールドリスト → `gen_hint_value_for_field` を各フィールドに適用
  - `vm_map_from_pairs(vec)` で `VMValue::Record("Map", ...)` を組み立てて返す

### 1-B: `Gen.hint_one_raw` の実装

- [ ] `vm_call_builtin` に `"Gen.hint_one_raw"` アームを追加
- [ ] `args[0]` から type_name (String) を取り出す
- [ ] `SmallRng::from_entropy()` でランダムジェネレータを生成
- [ ] `gen_hint_one_for_type` を呼び出して結果を返す
- [ ] 型不明の場合は `VMError` を返す

### 1-C: `Gen.hint_list_raw` の実装

- [ ] `vm_call_builtin` に `"Gen.hint_list_raw"` アームを追加
- [ ] `args[0]` から type_name、`args[1]` から n (i64) を取り出す
- [ ] `SmallRng::from_entropy()` でジェネレータ生成
- [ ] N 回 `gen_hint_one_for_type` を呼び `VMValue::List` に収めて返す

### 1-D: `Gen.set_yaml_config_raw` の実装

- [ ] `GenFieldConfig` 構造体を定義（distribution, min, max, range, locale, values, weights, null_rate）
- [ ] `GenYamlConfig` 構造体を定義（fields: `HashMap<String, GenFieldConfig>`）
- [ ] `thread_local! { static GEN_YAML_CONFIG: RefCell<HashMap<String, GenYamlConfig>> }` を追加
- [ ] `vm_call_builtin` に `"Gen.set_yaml_config_raw"` アームを追加
- [ ] `args[0]` から type_name、`args[1]` から yaml_name (String) を取り出す
- [ ] `<project_root>/gen/<yaml_name>.yaml` のパスを解決する（`std::env::current_dir()` 利用）
- [ ] `std::fs::read_to_string` → `serde_yaml::from_str::<serde_yaml::Value>` でパース
- [ ] `GEN_YAML_CONFIG` にエントリを挿入
- [ ] 成功時: `Ok(ok_vm(VMValue::Unit))`、失敗時: `Ok(err_vm(VMValue::Str(e)))`
- [ ] `gen_hint_value_for_field` に `GEN_YAML_CONFIG` 参照を追加して設定を適用（distribution, values, locale 等）

### 1-E: `Gen.to_parquet_raw` の実装

- [ ] `vm_call_builtin` に `"Gen.to_parquet_raw"` アームを追加
- [ ] `args`: type_name (0), path (1), n (2), seed (3) を取り出す
- [ ] `type_metas.get(type_name)` からフィールド名リストを取得
- [ ] Arrow スキーマを組み立てる（全フィールド `DataType::Utf8`）
- [ ] `File::create(path)` → `ArrowWriter::try_new(file, schema, props)` でライターを開く
- [ ] `SmallRng::seed_from_u64(seed as u64)` でジェネレータ初期化
- [ ] バッチサイズ 1000 行のループで `gen_hint_one_for_type` を呼び `RecordBatch` を組み立てて `writer.write` する
- [ ] `writer.close()` で確定
- [ ] 成功時: `Ok(ok_vm(VMValue::Int(n)))`、失敗時: `Ok(err_vm(VMValue::Str(msg)))`
- [ ] `use arrow::array::StringArray;` / `use arrow::record_batch::RecordBatch;` / `use parquet::arrow::ArrowWriter;` 等のインポートを追加

### 1-F: `Gen.to_csv_raw` の実装

- [ ] `vm_call_builtin` に `"Gen.to_csv_raw"` アームを追加
- [ ] `args`: type_name (0), path (1), n (2), seed (3) を取り出す
- [ ] `type_metas.get(type_name)` からフィールド名リストを取得
- [ ] `File::create(path)` → `csv::Writer::from_writer(file)` でライターを開く
- [ ] ヘッダー行（フィールド名）を `wtr.write_record` で書き込む
- [ ] `SmallRng::seed_from_u64(seed as u64)` でジェネレータ初期化
- [ ] N 回ループで `gen_hint_one_for_type` → 値リスト → `wtr.write_record`
- [ ] `wtr.flush()` で確定
- [ ] 成功時: `Ok(ok_vm(VMValue::Int(n)))`、失敗時: `Ok(err_vm(VMValue::Str(msg)))`

### 1-G: `Gen.load_into_raw` の実装

- [ ] `vm_call_builtin` に `"Gen.load_into_raw"` アームを追加
- [ ] `args`: type_name (0), conn handle (1), table_name (2), n (3), seed (4) を取り出す
- [ ] `DbHandle` から handle_id (u64) を取り出す
- [ ] `type_metas.get(type_name)` からフィールド名リストを取得
- [ ] 全行をベクタに生成してからロック取得（デッドロック回避）
  ```rust
  let mut all_rows: Vec<Vec<String>> = Vec::with_capacity(n as usize);
  let mut rng = SmallRng::seed_from_u64(seed as u64);
  for _ in 0..n {
      let row = gen_hint_one_for_type(&type_name, &mut rng, &self.type_metas)?;
      all_rows.push(/* フィールド値リスト */);
  }
  ```
- [ ] `duckdb_store().get(&handle_id)` で接続を取り出す
- [ ] `CREATE TABLE IF NOT EXISTS <table_name> (<fields> TEXT NOT NULL)` を実行
- [ ] `INSERT INTO <table_name> VALUES (?, ?, ...)` 文を準備
- [ ] all_rows をバッチ 1000 件ずつ INSERT
- [ ] 成功時: `Ok(ok_vm(VMValue::Int(n)))`、失敗時: `Ok(err_vm(VMValue::Str(msg)))`

### 1-H: `Gen.edge_cases_raw` の実装

- [ ] `vm_call_builtin` に `"Gen.edge_cases_raw"` アームを追加
- [ ] `args[0]` から type_name を取り出す
- [ ] `type_metas.get(type_name)` からフィールドリストを取得
- [ ] 型ごとの境界値リストを定義（Int: 5件, Float: 5件, String: 4件, Bool: 2件）
- [ ] `max_variants = フィールド中で最大の境界値数` を計算
- [ ] `max_variants` 行の `VMValue::List` を組み立てて返す（各行は `variant_idx % edges.len()` でインデックス）

---

## Phase 2: checker.rs へのシグネチャ登録（`fav/src/middle/checker.rs`）

- [ ] `("Gen", "hint_one_raw")` アームを追加（`require_random_effect` + `Map<String, String>` を返す）
- [ ] `("Gen", "hint_list_raw")` アームを追加（`require_random_effect` + `List<Map<String, String>>`）
- [ ] `("Gen", "to_parquet_raw")` アームを追加（`require_io_effect` + `Result<Int, String>`）
- [ ] `("Gen", "to_csv_raw")` アームを追加（`require_io_effect` + `Result<Int, String>`）
- [ ] `("Gen", "load_into_raw")` アームを追加（`require_db_effect` + `Result<Int, String>`）
- [ ] `("Gen", "edge_cases_raw")` アームを追加（エフェクトなし + `List<Map<String, String>>`）
- [ ] `("Gen", "set_yaml_config_raw")` アームを追加（`require_io_effect` + `Result<Unit, String>`）
- [ ] 各アームが既存 `("Gen", _)` フォールバックアームより**前に**配置されていることを確認

---

## Phase 3: Favnir rune ファイル作成

### 3-A: `runes/gen/hint.fav`（新規作成）

- [ ] `one_with_hints(type_name: String) -> Map<String, String> !Random` を実装
  - `Gen.hint_one_raw(type_name)` を呼ぶ
- [ ] `list_with_hints(type_name: String, n: Int, seed: Int) -> List<Map<String, String>> !Random` を実装
  - `Random.seed(seed)` → `Gen.hint_list_raw(type_name, n)`
- [ ] `one_from_yaml(type_name: String, yaml_name: String, seed: Int) -> Map<String, String> !Io !Random` を実装
  - `Random.seed(seed)` → `Gen.set_yaml_config_raw(type_name, yaml_name)` → `Gen.hint_one_raw(type_name)` or fallback

### 3-B: `runes/gen/output.fav`（新規作成）

- [ ] `to_parquet(type_name: String, path: String, n: Int, seed: Int) -> Result<Int, String> !Io` を実装
- [ ] `to_csv(type_name: String, path: String, n: Int, seed: Int) -> Result<Int, String> !Io` を実装

### 3-C: `runes/gen/integration.fav`（新規作成）

- [ ] `load_into(type_name: String, conn: DbHandle, table_name: String, n: Int, seed: Int) -> Result<Int, DbError> !Db` を実装
  - `Gen.load_into_raw(...)` を呼び `Err(e) => Result.err(DbError { code: "LOAD_ERROR" message: e })` に変換

### 3-D: `runes/gen/edge.fav`（新規作成）

- [ ] `edge_cases(type_name: String) -> List<Map<String, String>>` を実装（エフェクトなし）
- [ ] `first_edge(type_name: String) -> Option<Map<String, String>>` を実装

### 3-E: `runes/gen/gen.fav`（更新）

- [ ] 既存の `use primitives...` / `use structured...` を維持
- [ ] `use hint.{ one_with_hints, list_with_hints, one_from_yaml }` を追加
- [ ] `use output.{ to_parquet, to_csv }` を追加
- [ ] `use integration.{ load_into }` を追加
- [ ] `use edge.{ edge_cases, first_edge }` を追加

### 3-F: `runes/gen/gen.test.fav`（更新）

- [ ] ファイル冒頭に `type Order = { id: Int customer_name: String email: String amount: Float created_at: String }` を追加
- [ ] 既存 11 件のテストを維持（変更なし）
- [ ] `test_one_with_hints_email_contains_at` を追加
  - `Gen.hint_one_raw("Order")` → `email` フィールドが "@" を含む（`String.contains`で確認）
- [ ] `test_one_with_hints_id_is_positive` を追加
  - id フィールドが空でない（`!= ""`）
- [ ] `test_one_with_hints_name_not_empty` を追加
  - customer_name フィールドが空でない
- [ ] `test_list_with_hints_count` を追加
  - `Gen.hint_list_raw("Order", 7)` で 7 件返る
- [ ] `test_list_with_hints_all_have_email` を追加
  - 全行に email フィールドが存在する（Map.has_key）
- [ ] `test_to_csv_creates_file` を追加
  - `Gen.to_csv_raw("Pt", "../fav/tmp/gen_test_csv.csv", 10, 42)` が `Ok`
- [ ] `test_to_parquet_creates_file` を追加
  - `Gen.to_parquet_raw("Pt", "../fav/tmp/gen_test_par.parquet", 10, 42)` が `Ok`
- [ ] `test_edge_cases_not_empty` を追加
  - `List.length(Gen.edge_cases_raw("Pt")) > 0` が true
- [ ] `test_edge_cases_first_x_is_zero` を追加
  - `List.first(Gen.edge_cases_raw("Pt"))` の x フィールドが "0"
- [ ] `test_first_edge_is_some` を追加
  - `List.first(Gen.edge_cases_raw("Pt"))` が Some

---

## Phase 4: テスト追加

### 4-A: `fav/src/backend/vm_stdlib_tests.rs` 追加（4 件）

- [ ] `gen_hint_one_raw_email_field` — `eval(source)` でヒント生成、email フィールドが "@" を含む
  ```rust
  // テスト用 source:
  // type Order = { id: Int customer_name: String email: String amount: Float created_at: String }
  // fn main() -> Bool { Map.has_key(Gen.hint_one_raw("Order"), "email") }
  ```
- [ ] `gen_hint_one_raw_id_sequential` — 2 回呼んで id が "1", "2" と増えることを確認
  - `Random.seed` を挟んでリセット後、再度 "1" になることも確認
- [ ] `gen_to_csv_raw_writes_file` — `to_csv_raw` 後にファイルが存在し `fs::read_to_string` で行数を確認
  - テスト終了後に `std::fs::remove_file("tmp/gen_stdlib_test.csv")` でクリーンアップ
- [ ] `gen_edge_cases_raw_returns_multiple_rows` — `edge_cases_raw("Pt")` の結果が 2 件以上

### 4-B: `fav/src/driver.rs` 統合テスト追加（4 件）

- [ ] `gen_rune_test_file_passes` — `run_fav_test_file_with_runes("runes/gen/gen.test.fav")` が全テスト pass
- [ ] `gen_hint_in_favnir_source` — `exec_project_main_source_with_runes(runes: ["gen"])` で email が "@" を含む
- [ ] `gen_to_csv_in_favnir_source` — `exec_project_main_source_with_runes` で `to_csv` が Ok を返す
- [ ] `gen_load_into_duckdb_in_source` — `exec_project_main_source_with_runes(runes: ["duckdb", "gen"])` で `load_into` が Ok を返す

---

## Phase 5: examples 追加

- [ ] `examples/gen2_demo/fav.toml` を作成（name = "gen2_demo"）
- [ ] `examples/gen2_demo/gen/sale.yaml` を作成
  - amount: distribution=pareto, min=100, max=500000
  - created_at: range=last_90_days
  - customer_name: locale=ja
  - status: values=["active","completed","cancelled","refunded"], weights=[0.5,0.3,0.15,0.05]
- [ ] `examples/gen2_demo/src/main.fav` を作成
  - `import rune "gen"` + `import rune "duckdb"`
  - `type Sale = { id: Int customer_name: String email: String amount: Float created_at: String status: String }`
  - デモ 1: `gen.one_with_hints("Sale")` で 1 件生成・表示
  - デモ 2: `gen.list_with_hints("Sale", 5, 42)` で 5 件生成・ID 連番確認
  - デモ 3: `gen.to_csv("Sale", "tmp/sales.csv", 1000, 42)` で 1000 行 CSV 生成
  - デモ 4: `gen.to_parquet("Sale", "tmp/sales.parquet", 1000, 42)` で Parquet 生成
  - デモ 5: DuckDB でロード → COUNT(*) → 結果表示
  - デモ 6: `gen.edge_cases("Sale")` の件数表示

---

## 完了条件

- [ ] `cargo build` が通る
- [ ] 既存 826 件が全て pass
- [ ] 新規テスト 18 件以上が pass
- [ ] `gen.one_with_hints("Order")` の `email` フィールドが `@` を含む
- [ ] `gen.list_with_hints("Order", 5, 42)` の `id` フィールドが "1"〜"5" の連番
- [ ] `gen.to_csv(...)` でファイルが生成される
- [ ] `gen.to_parquet(...)` でファイルが生成される
- [ ] `gen.load_into("Sale", conn, "sales", 10000, 42)` が DuckDB に INSERT できる
- [ ] `gen.edge_cases("Order")` が 2 件以上の境界値リストを返す
- [ ] `examples/gen2_demo/` が `fav run` で動く
