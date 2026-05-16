# Favnir v4.4.0 実装計画 — Gen Rune 2.0

作成日: 2026-05-17

---

## Phase 0: バージョン更新

- `fav/Cargo.toml` の version を `"4.4.0"` に変更
- `fav/src/main.rs` のヘルプ文字列・バージョン表示を `4.4.0` に更新

Cargo.toml への新クレート追加は不要（`arrow = "52"`, `parquet = "52"`, `csv = "1"`, `serde_yaml = "0.9"` は既存）。

---

## Phase 1: VM プリミティブ追加（`fav/src/backend/vm.rs`）

### 1-A: フィールド名ヒントシステム（`Gen.hint_one_raw`）

**設計方針**:

`type_metas` からフィールド名・型情報を取得し、フィールド名パターンマッチで生成戦略を決定する。
連番 ID 管理のため `thread_local! { static HINT_ID_COUNTER: RefCell<HashMap<String, u64>> }` を使う。

```rust
fn gen_hint_value_for_field(field_name: &str, field_type: &Type, rng: &mut SmallRng) -> String {
    // パターンマッチの優先順位:
    // 1. uuid / *_uuid → uuid v4
    // 2. id / *_id → 連番（HINT_ID_COUNTER で型ごとに管理）
    // 3. email / *_email → "user{n}@example.com"
    // 4. *_name / full_name / name → 日本人名（固定リストから選択）
    // 5. first_name / given_name → 名のみ
    // 6. last_name / family_name → 姓のみ
    // 7. phone / *_phone → "090-XXXX-XXXX"
    // 8. *_at (created_at等) / *_datetime → ISO 8601 日時
    // 9. *_date / birth_date → "YYYY-MM-DD"
    // 10. price / amount / *_fee / *_price → 正の実数
    // 11. age → 20〜80 の整数
    // 12. count / *_count → 1〜999 の整数
    // 13. url / *_url → "https://example.com/item/{n}"
    // 14. zip / postal_code → "XXX-XXXX"
    // 15. address → "東京都{区}XXX丁目"
    // 16. description / body / content → 固定文例から選択
    // 17. status → "active" / "inactive" / "pending"
    // 18. flag / is_* / has_* → "true" または "false"
    // 19. その他 → 既存ランダム文字列 / 数値（field_type に応じる）
}
```

名前のリスト（固定）:
```rust
const JA_LAST_NAMES: &[&str] = &["田中", "鈴木", "佐藤", "高橋", "伊藤", "渡辺", "山本", "中村", "小林", "加藤"];
const JA_FIRST_NAMES: &[&str] = &["太郎", "花子", "一郎", "京子", "健二", "恵子", "誠", "裕子", "明", "直子"];
const DESCRIPTIONS: &[&str] = &[
    "標準的な商品です。",
    "人気の高いアイテムです。",
    "新商品です。",
    "定番の品です。",
];
```

**`vm_call_builtin` への追加アーム**:

```rust
"Gen.hint_one_raw" => {
    let type_name = /* args[0] to string */;
    let meta = self.type_metas.get(&type_name).ok_or(...)?;
    let mut rng = SmallRng::from_entropy();
    let mut map = vec![];
    for (field_name, field_type) in &meta.fields {
        let val = gen_hint_value_for_field(field_name, field_type, &mut rng);
        map.push((field_name.clone(), VMValue::Str(val)));
    }
    Ok(vm_map_from_pairs(map))
}

"Gen.hint_list_raw" => {
    let type_name = /* args[0] */;
    let n = /* args[1] as i64 */;
    let mut rng = SmallRng::from_entropy();
    let mut rows = vec![];
    for _ in 0..n {
        rows.push(gen_hint_one_for_type(&type_name, &mut rng, &self.type_metas)?);
    }
    Ok(VMValue::List(rows))
}
```

### 1-B: YAML 設定読み込み（`Gen.set_yaml_config_raw`）

```rust
// スレッドローカルな設定マップ
thread_local! {
    static GEN_YAML_CONFIG: RefCell<HashMap<String, GenYamlConfig>> = RefCell::new(HashMap::new());
}

struct GenYamlConfig {
    fields: HashMap<String, GenFieldConfig>,
}
struct GenFieldConfig {
    distribution: Option<String>,  // "uniform" / "normal" / "pareto"
    min: Option<f64>,
    max: Option<f64>,
    range: Option<String>,         // "last_90_days" 等
    locale: Option<String>,        // "ja" / "en"
    values: Vec<String>,           // choice の上書き
    weights: Vec<f64>,             // 重み付き確率
    null_rate: f64,                // 0.0〜1.0
}
```

`Gen.set_yaml_config_raw(type_name, yaml_name)`:
- `<project_root>/gen/<yaml_name>.yaml` を `serde_yaml::from_str` でパース
- `GEN_YAML_CONFIG` に格納
- `hint_one_raw` / `hint_list_raw` は `GEN_YAML_CONFIG` を参照してフィールド生成を上書き

### 1-C: Parquet ストリーム出力（`Gen.to_parquet_raw`）

```rust
"Gen.to_parquet_raw" => {
    let type_name = /* args[0] */;
    let path = /* args[1] */;
    let n = /* args[2] as i64 */;
    let seed = /* args[3] as i64 */;

    let meta = self.type_metas.get(&type_name).ok_or(...)?;
    let field_names: Vec<String> = meta.fields.iter().map(|(n,_)| n.clone()).collect();

    // Arrow スキーマ（全フィールド Utf8）
    let schema = Arc::new(Schema::new(
        field_names.iter().map(|n| Field::new(n, DataType::Utf8, false)).collect::<Vec<_>>()
    ));

    let file = File::create(&path).map_err(...)?;
    let props = WriterProperties::builder().build();
    let mut writer = ArrowWriter::try_new(file, schema.clone(), Some(props)).map_err(...)?;

    let mut rng = SmallRng::seed_from_u64(seed as u64);
    let batch_size = 1000usize;
    let mut written = 0i64;

    while written < n {
        let this_batch = ((n - written) as usize).min(batch_size);
        // バッチ生成
        let mut columns: Vec<Vec<String>> = vec![vec![]; field_names.len()];
        for _ in 0..this_batch {
            let row = gen_hint_one_for_type(&type_name, &mut rng, &self.type_metas)?;
            for (i, fname) in field_names.iter().enumerate() {
                columns[i].push(/* row の fname フィールドの値 */);
            }
        }
        // RecordBatch に変換して書き込み
        let arrays: Vec<Arc<dyn Array>> = columns.iter()
            .map(|col| Arc::new(StringArray::from(col.iter().map(|s| s.as_str()).collect::<Vec<_>>())) as Arc<dyn Array>)
            .collect();
        let batch = RecordBatch::try_new(schema.clone(), arrays).map_err(...)?;
        writer.write(&batch).map_err(...)?;
        written += this_batch as i64;
    }
    writer.close().map_err(...)?;
    Ok(ok_vm(VMValue::Int(written)))
}
```

### 1-D: CSV ストリーム出力（`Gen.to_csv_raw`）

```rust
"Gen.to_csv_raw" => {
    let type_name = /* args[0] */;
    let path = /* args[1] */;
    let n = /* args[2] as i64 */;
    let seed = /* args[3] as i64 */;

    let meta = ...;
    let field_names: Vec<String> = ...;

    let file = File::create(&path).map_err(...)?;
    let mut wtr = csv::Writer::from_writer(file);

    // ヘッダー行
    wtr.write_record(&field_names).map_err(...)?;

    let mut rng = SmallRng::seed_from_u64(seed as u64);
    for _ in 0..n {
        let row = gen_hint_one_for_type(&type_name, &mut rng, &self.type_metas)?;
        let vals: Vec<String> = field_names.iter()
            .map(|f| /* row のフィールド値 */)
            .collect();
        wtr.write_record(&vals).map_err(...)?;
    }
    wtr.flush().map_err(...)?;
    Ok(ok_vm(VMValue::Int(n)))
}
```

### 1-E: DuckDB 統合（`Gen.load_into_raw`）

```rust
"Gen.load_into_raw" => {
    let type_name = /* args[0] */;
    let handle_id = /* DbHandle から id を取り出す */;
    let table_name = /* args[2] */;
    let n = /* args[3] as i64 */;
    let seed = /* args[4] as i64 */;

    let meta = ...;
    let field_names: Vec<String> = ...;

    let store = duckdb_store();
    let conn = store.get(&handle_id).ok_or(...)?;

    // テーブルが存在しない場合は CREATE TABLE
    let create_sql = format!(
        "CREATE TABLE IF NOT EXISTS {} ({})",
        table_name,
        field_names.iter().map(|f| format!("{} TEXT NOT NULL", f)).collect::<Vec<_>>().join(", ")
    );
    conn.execute(&create_sql, []).map_err(...)?;

    // INSERT 文のプレースホルダ
    let placeholders = (0..field_names.len()).map(|_| "?").collect::<Vec<_>>().join(", ");
    let insert_sql = format!("INSERT INTO {} VALUES ({})", table_name, placeholders);

    let mut rng = SmallRng::seed_from_u64(seed as u64);
    let batch_size = 1000i64;
    let mut inserted = 0i64;

    while inserted < n {
        let this_batch = (n - inserted).min(batch_size);
        // duckdb の execute_batch や loop INSERT
        for _ in 0..this_batch {
            let row = gen_hint_one_for_type(&type_name, &mut rng, &self.type_metas)?;
            let vals: Vec<String> = field_names.iter().map(|f| /* 値 */).collect();
            conn.execute(&insert_sql, duckdb::params_from_iter(vals.iter())).map_err(...)?;
        }
        inserted += this_batch;
    }
    Ok(ok_vm(VMValue::Int(inserted)))
}
```

> **注意**: `duckdb_store()` は `MutexGuard` を返すため、呼び出しのスコープに注意する。
> `gen_hint_one_for_type` を呼ぶ前に guard を取得しておき、INSERT ループ内で使用する。

### 1-F: エッジケース生成（`Gen.edge_cases_raw`）

```rust
"Gen.edge_cases_raw" => {
    let type_name = /* args[0] */;
    let meta = self.type_metas.get(&type_name).ok_or(...)?;

    let int_edges = ["0", "-1", "1", "9223372036854775807", "-9223372036854775808"];
    let float_edges = ["0.0", "-1.0", "1.0", "3.4028235e38", "-3.4028235e38"];
    let string_edges = ["", "a", &"a".repeat(255), &"a".repeat(1000)];
    let bool_edges = ["true", "false"];

    // 最大バリアント数を決定
    let max_variants = meta.fields.iter().map(|(_, t)| match t {
        Type::Int => int_edges.len(),
        Type::Float => float_edges.len(),
        Type::String => string_edges.len(),
        Type::Bool => bool_edges.len(),
        _ => 1,
    }).max().unwrap_or(1);

    let mut rows = vec![];
    for variant_idx in 0..max_variants {
        let mut map = vec![];
        for (fname, ftype) in &meta.fields {
            let edges: &[&str] = match ftype {
                Type::Int => &int_edges,
                Type::Float => &float_edges,
                Type::String => &string_edges[..2], // 短いもののみ（長文字列はvariant>=2で）
                Type::Bool => &bool_edges,
                _ => &[""],
            };
            let val = edges[variant_idx % edges.len()];
            map.push((fname.clone(), VMValue::Str(val.to_string())));
        }
        rows.push(vm_map_from_pairs(map));
    }
    Ok(VMValue::List(rows))
}
```

---

## Phase 2: checker.rs へのシグネチャ登録

**変更ファイル**: `fav/src/middle/checker.rs`

既存 `("Gen", _)` アームの前に新アームを追加:

| アーム | エフェクト | 戻り値型 |
|--------|-----------|---------|
| `("Gen", "hint_one_raw")` | `!Random` | `Map<String, String>` |
| `("Gen", "hint_list_raw")` | `!Random` | `List<Map<String, String>>` |
| `("Gen", "to_parquet_raw")` | `!Io` | `Result<Int, String>` |
| `("Gen", "to_csv_raw")` | `!Io` | `Result<Int, String>` |
| `("Gen", "load_into_raw")` | `!Db` | `Result<Int, String>` |
| `("Gen", "edge_cases_raw")` | なし | `List<Map<String, String>>` |
| `("Gen", "set_yaml_config_raw")` | `!Io` | `Result<Unit, String>` |

---

## Phase 3: Favnir rune ファイル作成

### 3-A: `runes/gen/hint.fav`（新規）

```favnir
// runes/gen/hint.fav — フィールド名ヒントによるリアルデータ生成 (v4.4.0)

// one_with_hints: フィールド名を見てリアルなデータを生成
public fn one_with_hints(type_name: String) -> Map<String, String> !Random {
    Gen.hint_one_raw(type_name)
}

// list_with_hints: N行のリアルデータを生成
public fn list_with_hints(type_name: String, n: Int, seed: Int) -> List<Map<String, String>> !Random {
    Random.seed(seed)
    Gen.hint_list_raw(type_name, n)
}

// one_from_yaml: gen/*.yaml の制約を適用してデータ生成
public fn one_from_yaml(type_name: String, yaml_name: String, seed: Int) -> Map<String, String> !Io !Random {
    Random.seed(seed)
    match Gen.set_yaml_config_raw(type_name, yaml_name) {
        Ok(_)  => Gen.hint_one_raw(type_name)
        Err(_) => Gen.one_raw(type_name)
    }
}
```

### 3-B: `runes/gen/output.fav`（新規）

```favnir
// runes/gen/output.fav — 大量データの Parquet / CSV ストリーム出力 (v4.4.0)

// to_parquet: N行をメモリに乗せずに Parquet に書き込む
// 戻り値: Ok(書き込み行数) または Err(エラーメッセージ)
public fn to_parquet(type_name: String, path: String, n: Int, seed: Int) -> Result<Int, String> !Io {
    Gen.to_parquet_raw(type_name, path, n, seed)
}

// to_csv: N行を CSV に書き込む（1行目はヘッダー）
public fn to_csv(type_name: String, path: String, n: Int, seed: Int) -> Result<Int, String> !Io {
    Gen.to_csv_raw(type_name, path, n, seed)
}
```

### 3-C: `runes/gen/integration.fav`（新規）

```favnir
// runes/gen/integration.fav — DuckDB 統合 (v4.4.0)
// DuckDB rune（v4.3.0）と組み合わせて使用する

// load_into: 生成データを DuckDB テーブルに直接 INSERT
// テーブルが存在しない場合は自動 CREATE TABLE（全フィールド TEXT）
// conn は duckdb.open() で得た DbHandle
public fn load_into(type_name: String, conn: DbHandle, table_name: String, n: Int, seed: Int) -> Result<Int, DbError> !Db {
    match Gen.load_into_raw(type_name, conn, table_name, n, seed) {
        Ok(rows) => Result.ok(rows)
        Err(e)   => Result.err(DbError { code: "LOAD_ERROR" message: e })
    }
}
```

### 3-D: `runes/gen/edge.fav`（新規）

```favnir
// runes/gen/edge.fav — エッジケース・境界値生成 (v4.4.0)

// edge_cases: 境界値のリストを生成（プロパティベーステスト向け）
// 純粋関数（エフェクトなし）
public fn edge_cases(type_name: String) -> List<Map<String, String>> {
    Gen.edge_cases_raw(type_name)
}

// first_edge: 最初の境界値のみ返す（単体テスト用）
public fn first_edge(type_name: String) -> Option<Map<String, String>> {
    List.first(Gen.edge_cases_raw(type_name))
}
```

### 3-E: `runes/gen/gen.fav`（更新）

```favnir
// runes/gen/gen.fav — Gen Rune public API (v4.4.0)
use primitives.{ int_val, float_val, bool_val, string_val, choice }
use structured.{ one, list, simulate, profile }
use hint.{ one_with_hints, list_with_hints, one_from_yaml }
use output.{ to_parquet, to_csv }
use integration.{ load_into }
use edge.{ edge_cases, first_edge }
```

### 3-F: `runes/gen/gen.test.fav`（更新）

既存 11 件を維持し、新テスト 10 件を追加（ファイル末尾に追記）。

---

## Phase 4: テスト追加

### 4-A: `fav/src/backend/vm_stdlib_tests.rs` 追加（4 件）

```
gen_hint_one_raw_email_field
  — hint_one_raw("Order") の email フィールドが "@" を含む

gen_hint_one_raw_id_sequential
  — hint_one_raw を 2 回呼ぶと id フィールドが "1", "2" と増える
  （Random.seed でリセットしてから確認）

gen_to_csv_raw_writes_file
  — to_csv_raw("Pt", "tmp/gen_test.csv", 5, 42) でファイルが生成され行数が 6（ヘッダー + 5行）

gen_edge_cases_raw_returns_multiple_rows
  — edge_cases_raw("LabeledNum") が 2 件以上返る
```

各テストは `eval(source)` 形式または Rust 直接呼び出し形式。
Parquet テスト（`to_parquet_raw`）は DuckDB テストと同様に `tmp/` ディレクトリを使用。

### 4-B: `runes/gen/gen.test.fav`（10 件追加）

テスト関数定義はファイル冒頭に追記:

```favnir
type Order = { id: Int customer_name: String email: String amount: Float created_at: String }
```

追加テスト:
```
test_one_with_hints_email_contains_at
  — Gen.hint_one_raw("Order") の email が "@" を含む

test_one_with_hints_id_is_positive
  — id フィールドが空でない

test_one_with_hints_name_not_random_ascii
  — customer_name が 1 文字以上

test_list_with_hints_count
  — Gen.hint_list_raw("Order", 7) で 7 件

test_list_with_hints_all_have_email
  — 全行に email フィールドが存在

test_to_csv_creates_file
  — Gen.to_csv_raw("Pt", "../fav/tmp/gen_test_csv.csv", 10, 42) が Ok

test_to_parquet_creates_file
  — Gen.to_parquet_raw("Pt", "../fav/tmp/gen_test_par.parquet", 10, 42) が Ok

test_edge_cases_not_empty
  — Gen.edge_cases_raw("Pt") が空でない

test_edge_cases_has_id_zero
  — 最初のエッジケースの x が "0"

test_first_edge_is_some
  — List.first(Gen.edge_cases_raw("Pt")) が Some
```

### 4-C: `fav/src/driver.rs` 統合テスト追加（4 件）

```
gen_rune_test_file_passes
  — run_fav_test_file_with_runes("runes/gen/gen.test.fav") が全 pass
    （既存テストが gen.test.fav 全体として動く確認）

gen_hint_in_favnir_source
  — exec_project_main_source_with_runes でヒント生成した email が "@" を含む

gen_to_csv_in_favnir_source
  — exec_project_main_source_with_runes で to_csv が Ok を返す

gen_load_into_duckdb_in_source
  — exec_project_main_source_with_runes(runes: ["duckdb", "gen"]) で load_into が Ok を返す
```

---

## Phase 5: examples 追加

### 5-A: `examples/gen2_demo/`

```
examples/gen2_demo/
  fav.toml
  src/
    main.fav
  gen/
    sale.yaml
```

`main.fav` のデモ内容:
1. `gen.one_with_hints("Sale")` でリアルなデータを 1 件生成・表示
2. `gen.list_with_hints("Sale", 5, 42)` で 5 件生成・ID の連番を確認
3. `gen.to_csv("Sale", "tmp/sales.csv", 1000, 42)` で 1000 行 CSV 生成
4. `gen.to_parquet("Sale", "tmp/sales.parquet", 1000, 42)` で Parquet 生成
5. `duckdb.open(":memory:") + gen.load_into("Sale", conn, "sales", 10000, 42)` で DuckDB に INSERT
6. DuckDB で `SELECT COUNT(*) FROM sales` を実行して行数を確認
7. `gen.edge_cases("Sale")` の件数を表示

`gen/sale.yaml`:
```yaml
amount:
  distribution: pareto
  min: 100
  max: 500000
created_at:
  range: last_90_days
customer_name:
  locale: ja
status:
  values: ["active", "completed", "cancelled", "refunded"]
  weights: [0.5, 0.3, 0.15, 0.05]
```

---

## 実装順序と依存関係

```
Phase 0 (バージョン更新)
  ↓
Phase 1-A (hint_one_raw / hint_list_raw)   ← コアロジック
Phase 1-B (set_yaml_config_raw)            ← hint に依存
Phase 1-C (to_parquet_raw)                 ← hint_one_raw に依存
Phase 1-D (to_csv_raw)                     ← hint_one_raw に依存
Phase 1-E (load_into_raw)                  ← hint_one_raw + DUCKDB_CONNS に依存
Phase 1-F (edge_cases_raw)                 ← 独立
  ↓
Phase 2 (checker.rs)                        ← Phase 1 と並列可
  ↓
Phase 3 (Favnir rune ファイル)
  ↓
Phase 4 (テスト)
  ↓
Phase 5 (examples)
```

Phase 1-A を最初に実装し（最もコアで他の機能が依存）、その後 1-C/1-D/1-E/1-F を並列実施できる。

---

## リスクと対策

| リスク | 影響 | 対策 |
|--------|------|------|
| `load_into_raw` で DUCKDB_CONNS の Mutex をロックしたまま hint_one_raw を呼ぶとデッドロック | VM がハング | `duckdb_store().get(&id)` で接続を取り出す前に全行生成し、ロック取得後に INSERT のみ実行 |
| Parquet ストリーム書き込みの型不一致（Utf8 vs 数値型） | Arrow エラー | `to_parquet_raw` は全フィールドを `DataType::Utf8` で扱う（数値変換は呼び出し側の責務） |
| `gen.test.fav` で `to_parquet_raw` / `to_csv_raw` がファイルに書くと CI でパス問題 | テスト失敗 | `../fav/tmp/` ディレクトリへ書き込む（duckdb.test.fav と同じパターン）。`tmp/` は `.gitignore` 済み |
| `HINT_ID_COUNTER` がテスト間で値を引き継ぐ | テスト順序依存 | 連番テストでは `Random.seed` でカウンタをリセットする（seed 設定時に `HINT_ID_COUNTER` もリセット） |
| DuckDB `params_from_iter` が Windows で文字化け | INSERT 失敗 | `duckdb::params_from_iter` は `ToSql` 実装を使う。文字列は UTF-8 のみ使用するため問題なし |
| `serde_yaml` パースエラー時の gen フォールバック | データ品質 | `set_yaml_config_raw` が Err のとき `hint_one_raw` は `one_raw` にフォールバック（エラーを飲み込む） |
| `gen/sale.yaml` が存在しない環境での examples | 実行エラー | examples/gen2_demo は `one_from_yaml` でフォールバック動作を実証するデモにする |

---

## 完了条件チェックリスト

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
