# Favnir v4.4.0 タスクリスト — Gen Rune 2.0

作成日: 2026-05-17
完了日: 2026-05-17

---

## Phase 0: バージョン更新

- [x] `fav/Cargo.toml` の version を `"4.4.0"` に変更
- [x] `fav/src/main.rs` のヘルプ文字列・バージョン表示を `4.4.0` に更新

---

## Phase 1: VM プリミティブ追加（`fav/src/backend/vm.rs`）

### 1-A: ヘルパー関数・定数定義

- [x] 日本語姓名リスト `JA_LAST_NAMES` / `JA_FIRST_NAMES` を定数として追加
- [x] ~~英語名リスト `EN_FIRST_NAMES` / `EN_LAST_NAMES` を追加~~ → JA のみ実装（英語名は未追加）
- [x] `DESCRIPTIONS` 固定文例リストを追加
- [x] `thread_local! { static HINT_ID_COUNTER: RefCell<HashMap<String, u64>> }` を追加
- [x] `gen_hint_value_for_field(type_name, field_name, ty) -> String` ヘルパー関数を実装
  - フィールド名パターン: uuid > id/*_id(連番) > email(user{n}@example.com) > *_name(日本語名) > phone > *_at/*_datetime(ISO8601) > *_date > price/amount/*_fee > age > count > url > zip > address > description/body/content > status > is_*/has_*/flag > default
  - GEN_YAML_CONFIG オーバーライドを最優先チェック
- [x] `gen_hint_one_row(type_name, type_metas) -> Result<VMValue, String>` ヘルパー関数を実装

### 1-B: `Gen.hint_one_raw` の実装

- [x] `vm_call_builtin` に `"Gen.hint_one_raw"` アームを追加（`args[0]` = type_name）
- [x] 型不明の場合は `Ok(err_vm(...))` を返す

### 1-C: `Gen.hint_list_raw` の実装

- [x] `vm_call_builtin` に `"Gen.hint_list_raw"` アームを追加（`args[0]` = type_name, `args[1]` = n）
- [x] N 回 `gen_hint_one_row` を呼び `ok_vm(VMValue::List(...))` を返す

### 1-D: `Gen.set_yaml_config_raw` の実装

- [x] `GenFieldConfig` / `GenYamlConfig` 構造体を定義
- [x] `thread_local! { static GEN_YAML_CONFIG: ... }` を追加
- [x] `vm_call_builtin` に `"Gen.set_yaml_config_raw"` アームを追加
  - `args[0]` = type_name, `args[1]` = yaml_path（直接パスを受け取る実装）
- [x] `serde_yaml::from_str` でパース → `GEN_YAML_CONFIG` に格納
- [x] 成功: `Ok(VMValue::Unit)`、失敗: Rust エラーとして伝播

### 1-E: `Gen.to_parquet_raw` の実装

- [x] `vm_call_builtin` に `"Gen.to_parquet_raw"` アームを追加
- [x] **実装 API**: `(path: String, rows: List<Map>)` — 事前生成済みリストを受け取る設計に変更
- [x] Arrow スキーマ（全フィールド Utf8）を組み立て → ArrowWriter で書き込み
- [x] 成功: `Ok(ok_vm(VMValue::Unit))`、失敗: `Ok(err_vm(...))`

### 1-F: `Gen.to_csv_raw` の実装

- [x] `vm_call_builtin` に `"Gen.to_csv_raw"` アームを追加
- [x] **実装 API**: `(path: String, rows: List<Map>)` — 事前生成済みリストを受け取る設計に変更
- [x] ヘッダー行（フィールド名ソート済み）→ 各行を csv::Writer で書き込み

### 1-G: `Gen.load_into_raw` の実装

- [x] `vm_call_builtin` に `"Gen.load_into_raw"` アームを追加
- [x] **実装 API**: `(conn: DbHandle|Int, table_name: String, rows: List<Map>)` — 事前生成済みリストを INSERT
- [x] `VMValue::DbHandle(id)` と `VMValue::Int(n)` の両方を受け付ける（後方互換）
- [x] `CREATE TABLE IF NOT EXISTS` + `INSERT` でバルクロード
- [x] 成功: `Ok(ok_vm(VMValue::Int(inserted_count)))`、失敗: `Ok(err_vm(...))`

### 1-H: `Gen.edge_cases_raw` の実装

- [x] `vm_call_builtin` に `"Gen.edge_cases_raw"` アームを追加
- [x] 4 行固定: Row1=最小値(Int→"0", Float→"0.0", Bool→"false", String→"")、Row2=最大値、Row3=空文字列、Row4=空白文字
- [x] 型不明: `Ok(err_vm(...))`

---

## Phase 2: checker.rs へのシグネチャ登録（`fav/src/middle/checker.rs`）

- [x] `("Gen", "hint_one_raw")` → `Result<Map<String, String>, String>`（エフェクト強制なし）
- [x] `("Gen", "hint_list_raw")` → `Result<List<Map<String, String>>, String>`
- [x] `("Gen", "to_parquet_raw")` → `Result<Unit, String>`
- [x] `("Gen", "to_csv_raw")` → `Result<Unit, String>`
- [x] `("Gen", "load_into_raw")` → `Result<Int, String>`
- [x] `("Gen", "edge_cases_raw")` → `Result<List<Map<String, String>>, String>`
- [x] `("Gen", "set_yaml_config_raw")` → `Unit`
- [x] 各アームが既存 `("Gen", _)` フォールバックアームより前に配置

---

## Phase 3: Favnir rune ファイル作成

### 3-A: `runes/gen/hint.fav`（新規作成）✅

- [x] `hint_one(type_name) -> Result<Map, String> !Random` を実装
- [x] `hint_list(type_name, n) -> Result<List<Map>, String> !Random` を実装
- [x] `set_yaml(type_name, path) -> Unit` を実装
- 注: 関数名は計画（`one_with_hints` / `list_with_hints` / `one_from_yaml`）から変更

### 3-B: `runes/gen/output.fav`（新規作成）✅

- [x] `to_csv(path, rows) -> Result<Unit, String> !Io` を実装
- [x] `to_parquet(path, rows) -> Result<Unit, String> !Io` を実装
- [x] `load_into(conn: DbHandle, table_name, rows) -> Result<Int, String> !Db` を実装
- 注: `load_into` は当初 `integration.fav` に分離予定だったが `output.fav` に集約

### 3-C: `runes/gen/integration.fav`（新規作成）

- [ ] ~~`load_into` を独立ファイルに分離~~ → `output.fav` に統合済みのためスキップ

### 3-D: `runes/gen/edge.fav`（新規作成）✅

- [x] `edge_cases(type_name) -> Result<List<Map>, String>` を実装
- [ ] ~~`first_edge` を実装~~ → 未実装（`List.first(gen.edge_cases(...))` で代替可能）

### 3-E: `runes/gen/gen.fav`（更新）✅

- [x] 既存の `use primitives...` / `use structured...` を維持
- [x] `use hint.{ hint_one, hint_list, set_yaml }` を追加
- [x] `use output.{ to_csv, to_parquet, load_into }` を追加
- [x] `use edge.{ edge_cases }` を追加

### 3-F: `runes/gen/gen.test.fav`（更新）✅

- [x] `type Order = { id: Int customer_name: String email: String amount: Float created_at: String }` を追加
- [x] 既存 19 件のテストを維持（変更なし）
- [x] `"hint_one Order email contains at-sign"` を追加
- [x] `"hint_one Order id field exists"` を追加
- [x] `"hint_one Order customer_name field exists"` を追加
- [x] `"hint_list Order returns 7 rows"` を追加
- [x] `"hint_list Order first row has email key"` を追加
- [x] `"to_csv_raw writes Order data to file"` を追加（`../fav/tmp/gen_order_test.csv`）
- [x] `"to_parquet_raw writes Order data to file"` を追加（`../fav/tmp/gen_order_test.parquet`）
- [x] `"edge_cases_raw first row x is zero"` を追加
- [x] `"edge_cases_raw List.first returns Some"` を追加

---

## Phase 4: テスト追加

### 4-A: `fav/src/backend/vm_stdlib_tests.rs` 追加（4 件）✅

- [x] `gen_hint_one_raw_returns_ok_with_fields` — Person 型 3 フィールド確認
- [x] `gen_hint_list_raw_returns_correct_count` — Item 型 8 件確認
- [x] `gen_edge_cases_raw_returns_four_rows` — Score 型 4 行確認
- [x] `gen_edge_cases_raw_unknown_type_returns_err` — 存在しない型が Err を返す

### 4-B: `fav/src/driver.rs` 統合テスト追加（8 件）✅

- [x] `gen_rune_test_file_passes` — gen.test.fav 全テスト pass
- [x] `gen_one_raw_field_count_in_favnir_source` — Product 型 3 フィールド
- [x] `gen_list_raw_count_in_favnir_source` — Event 型 8 件
- [x] `gen_profile_raw_total_in_favnir_source` — Metric 型 15 件 profile
- [x] `gen_simulate_raw_count_in_favnir_source` — Log 型 12 件 simulate
- [x] `gen_hint_one_raw_field_count_in_favnir_source` — User 型 4 フィールド
- [x] `gen_hint_list_raw_count_in_favnir_source` — Order 型 6 件
- [x] `gen_edge_cases_raw_returns_four_in_favnir_source` — Metric 型 4 行
- [x] `gen_to_csv_raw_creates_file` — tempdir に CSV 生成・ファイル存在確認
- [x] `gen_hint_email_contains_at_in_favnir_source` — email が "@" を含む
- [x] `gen_to_csv_with_hints_in_favnir_source` — hint_list → to_csv_raw が Ok
- [x] `gen_load_into_duckdb_in_favnir_source` — hint_list → load_into_raw → DuckDB INSERT

---

## Phase 5: examples 追加 ✅

- [x] `examples/gen2_demo/fav.toml` を作成（name = "gen2_demo", src = "src"）
- [x] `examples/gen2_demo/gen/sale.yaml` を作成（amount/created_at/customer_name/status 設定）
- [x] `examples/gen2_demo/src/main.fav` を作成
  - import rune "gen" + import rune "duckdb"
  - type Sale 定義
  - デモ 1: hint_one → email/customer_name 表示
  - デモ 2: hint_list 5 件 → 件数表示
  - デモ 3: hint_list 1000 件 → to_csv
  - デモ 4: hint_list 1000 件 → to_parquet
  - デモ 5: duckdb.open + load_into 100 件
  - デモ 6: edge_cases 件数表示

---

## 完了条件

- [x] `cargo build` が通る
- [x] 既存 826 件が全て pass（実績: 837 件 pass）
- [x] 新規テスト 18 件以上が pass（実績: Rust +11件、Favnir +9件 = 計 20 件以上）
- [x] `hint_one_raw("Order")` の `email` フィールドが `@` を含む
- [x] `Random.seed` 後 `hint_list_raw("Order", 5)` の `id` フィールドが "1"〜"5" の連番
- [x] `to_csv_raw(path, rows)` でファイルが生成される
- [x] `to_parquet_raw(path, rows)` でファイルが生成される
- [x] `load_into_raw(conn, "sales", rows)` が DuckDB に INSERT できる
- [x] `edge_cases_raw("Pt")` が 4 件の境界値リストを返す
- [x] `examples/gen2_demo/` が作成済み
