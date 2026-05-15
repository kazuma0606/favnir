# Favnir v3.5.0 Tasks

## Phase 0: Version Bump

- [ ] `fav/Cargo.toml`: `version = "3.5.0"`
- [ ] `cargo build` 成功、`env!("CARGO_PKG_VERSION")` 伝播
- [ ] `fav --version` で `favnir 3.5.0` を確認

## Phase 1: 型登録 + namespace

- [ ] `fav/src/middle/checker.rs`: `"GenProfile"` を stdlib 型として登録
  - フィールド: `total: Int`, `valid: Int`, `invalid: Int`, `rate: Float`
- [ ] `fav/src/middle/checker.rs`: `"Gen"` namespace を stdlib グローバル登録ループに追加
- [ ] `fav/src/middle/compiler.rs`: `"Gen"` を2箇所の登録ループに追加
- [ ] `fav/src/middle/checker.rs`: `("Random", "seed")` の型シグネチャを登録 → `Unit`
- [ ] `fav/src/middle/checker.rs`: `Gen.*` 5関数の型シグネチャを登録
  - `Gen.string_val`: `Int -> String !Random`
  - `Gen.one_raw`: `String -> Map<String, String> !Random`
  - `Gen.list_raw`: `(String, Int) -> List<Map<String, String>> !Random`
  - `Gen.simulate_raw`: `(String, Int, Float) -> List<Map<String, String>> !Random`
  - `Gen.profile_raw`: `(String, List<Map<String, String>>) -> GenProfile`

## Phase 2: VM プリミティブ

### 2-A: thread-local SEEDED_RNG の追加 (`backend/vm.rs`)

- [ ] `thread_local! { static SEEDED_RNG: RefCell<Option<rand::rngs::SmallRng>> }` を追加
- [ ] `fn random_int_impl(min: i64, max: i64) -> i64` ヘルパー追加（SEEDED_RNG を優先使用）
- [ ] `fn random_float_impl() -> f64` ヘルパー追加（同上）
- [ ] 既存の `"Random.int"` ビルトイン → `random_int_impl` を使うよう修正
- [ ] 既存の `"Random.float"` ビルトイン → `random_float_impl` を使うよう修正

### 2-B: `Random.seed` (`backend/vm.rs`)

- [ ] `"Random.seed"` ビルトイン追加
  - `SEEDED_RNG` に `SmallRng::seed_from_u64(n)` をセット
  - → `VMValue::Unit`

### 2-C: `Gen.string_val` (`backend/vm.rs`)

- [ ] `fn random_alphanumeric_string(len: usize) -> String` ヘルパー追加
  - SEEDED_RNG がある場合はそれを使う; なければ `thread_rng`
  - `b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"` から選択
- [ ] `"Gen.string_val"` ビルトイン追加 → `random_alphanumeric_string(len)`

### 2-D: `Gen.one_raw` (`backend/vm.rs`)

- [ ] `fn gen_value_for_type(ty: &str) -> Result<String, String>` ヘルパー追加
  - `"Int"` → `random_int_impl(-1000, 1000).to_string()`
  - `"Float"` → `random_float_impl().to_string()`
  - `"Bool"` → `"true"` / `"false"` (random)
  - `"String"` → `random_alphanumeric_string(8)`
  - `"Option<T>"` → 50% 空文字列, 50% `gen_value_for_type` の内側型
  - その他 → `""` (fallback)
- [ ] `fn gen_one_raw_impl(type_name: &str, type_metas: &...) -> Result<VMValue, String>` 追加
  - type_metas から type_name のフィールド一覧取得 → 各フィールドに `gen_value_for_type`
  - → `VMValue::Map(HashMap<String, VMValue::Str(...)>)`
- [ ] `"Gen.one_raw"` ビルトイン追加

### 2-E: `Gen.list_raw` (`backend/vm.rs`)

- [ ] `"Gen.list_raw"` ビルトイン追加
  - args: (type_name: String, n: Int)
  - n 回 `gen_one_raw_impl` を呼んで `VMValue::List` で返す

### 2-F: `Gen.simulate_raw` (`backend/vm.rs`)

- [ ] `fn gen_simulate_one(type_name: &str, noise: f64, type_metas: &...) -> Result<VMValue, String>` 追加
  - `gen_one_raw_impl` の結果の各フィールドを `noise` 確率で破損値に置換
  - 破損値候補: `""`, `"NULL"`, `"NaN"`, `"-999999"`, `"true"` (Bool フィールドに String を入れる)
- [ ] `"Gen.simulate_raw"` ビルトイン追加
  - args: (type_name: String, n: Int, noise: Float)

### 2-G: `Gen.profile_raw` (`backend/vm.rs`)

- [ ] `fn is_valid_for_type(val: &str, ty: &str) -> bool` ヘルパー追加
  - `"Int"` → `val.parse::<i64>().is_ok()`
  - `"Float"` → `val.parse::<f64>().is_ok()`
  - `"Bool"` → `val == "true" || val == "false"`
  - `"String"` → `true`
  - `"Option<T>"` → 空文字列 or 内側型に valid
  - その他 → `true`
- [ ] `fn is_valid_row(row: &HashMap<String, String>, meta: &TypeMeta) -> bool` 追加
- [ ] `"Gen.profile_raw"` ビルトイン追加
  - args: (type_name: String, data: List<Map<String, String>>)
  - → `VMValue::Record("GenProfile", { total, valid, invalid, rate })`

### テスト (`backend/vm_stdlib_tests.rs`)

- [ ] Test: `random_seed_makes_deterministic`
- [ ] Test: `gen_string_val_returns_correct_length`
- [ ] Test: `gen_one_raw_returns_map_with_fields`
- [ ] Test: `gen_list_raw_returns_n_rows`
- [ ] Test: `gen_list_raw_seed_deterministic`
- [ ] Test: `gen_simulate_raw_introduces_noise`
- [ ] Test: `gen_profile_raw_all_valid`
- [ ] Test: `gen_profile_raw_with_corrupt_data`

## Phase 3: `runes/gen/gen.fav`

- [ ] `runes/gen/gen.fav` 作成（8 public 関数、全て Favnir 実装）
  - `public fn int_val(min: Int, max: Int) -> Int !Random`
  - `public fn float_val() -> Float !Random`
  - `public fn bool_val() -> Bool !Random`
  - `public fn string_val(len: Int) -> String !Random`
  - `public fn choice(items: List<String>) -> Option<String> !Random`
  - `public fn one(type_name: String) -> Map<String, String> !Random`
  - `public fn list(type_name: String, n: Int, seed: Int) -> List<Map<String, String>> !Random`
  - `public fn simulate(type_name: String, n: Int, noise: Float, seed: Int) -> List<Map<String, String>> !Random`
  - `public fn profile(type_name: String, data: List<Map<String, String>>) -> GenProfile`
- [ ] `fav check runes/gen/gen.fav` でエラーなし確認

## Phase 4: `runes/gen/gen.test.fav`

- [ ] `runes/gen/gen.test.fav` 作成（10 テスト）
  - `test_one_returns_map_with_correct_fields`
  - `test_one_field_types_match_schema`（Favnir 内で定義した型に対して）
  - `test_list_length_matches_n`
  - `test_list_seed_deterministic` — `seed=42` で2回呼ぶと同じ結果
  - `test_list_seed_different` — `seed=1` と `seed=2` で異なる結果
  - `test_choice_returns_item_from_list`
  - `test_choice_empty_returns_none`
  - `test_simulate_noise_1_corrupts_all` — `noise=1.0` で全フィールドが型不適合
  - `test_profile_clean_data_rate_1` — 正常データの rate ≈ 1.0
  - `test_profile_corrupt_data_rate_low` — noise=1.0 データの rate < 0.5

## Phase 5: driver.rs 統合テスト

- [ ] `fav/src/driver.rs` の `migrate_tests` モジュールに統合テスト追加
  - `gen_rune_one_produces_map`
  - `gen_rune_list_deterministic_with_seed`
  - `gen_rune_simulate_introduces_corruption`
  - `gen_rune_profile_measures_validity`
  - `gen_rune_test_file_passes` — `runes/gen/gen.test.fav` の全テスト実行
- [ ] 既存の全テストが通ること確認 (`cargo test`)

## Phase 6: `fav check --sample N`

- [ ] `fav/src/main.rs`: `check` アームに `--sample <n>` フラグ追加
  - `sample: Option<usize>` を `cmd_check` に渡す
- [ ] `fav/src/driver.rs`: `pub fn cmd_check(file, no_warn, sample: Option<usize>)` に拡張
  - `sample.is_some()` のとき:
    1. 通常の型チェックを実行
    2. チェック成功の場合のみ synthetic data テストへ進む
    3. ソースを解析して最初の `stage` の入力型名を取得（`collect_pipeline_input_type`）
    4. `gen_list_raw_for_sample(type_name, n)` で n 件生成
    5. パイプラインを生成データで試し実行
    6. pass/fail を出力
- [ ] `fav/src/driver.rs`: `fn collect_pipeline_input_type(program: &ast::Program) -> Option<String>` 追加
  - 最初の `stage` 定義の入力引数型を返す
  - 見つからなければ `None`
- [ ] `fav/src/main.rs`: HELP テキストに `--sample N` オプションを追記
- [ ] Test: `check_sample_runs_with_synthetic_data`
- [ ] Test: `check_sample_skips_when_type_not_found`

## Phase 7: サンプル + ドキュメント

### サンプル

- [ ] `fav/examples/gen_demo/fav.toml` 作成
- [ ] `fav/examples/gen_demo/src/main.fav` 作成
  - `gen.list("User", 5, 42)` で生成 + `gen.simulate` + `gen.profile` を組み合わせ

### ドキュメント

- [ ] `versions/v3.5.0/langspec.md` 作成
- [ ] `versions/v3.5.0/migration-guide.md` 作成（破壊的変更なし）
- [ ] `versions/v3.5.0/progress.md` を全 Phase `[x]` に更新
