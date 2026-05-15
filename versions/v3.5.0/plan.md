# Favnir v3.5.0 Implementation Plan

## Overview

v3.5.0 は `gen` rune を追加する。
実装の中心は `backend/vm.rs` への VM プリミティブ追加と
`runes/gen/gen.fav` の Favnir 実装。

`gen.fav` は既存の `Random.*` プリミティブと新規 `Gen.*` プリミティブを組み合わせた
薄いラッパーとして実装する（`db.fav` が `DB.*` を包んでいるのと同じ構造）。

Total phases: 7

---

## Phase 0: Version Bump

**Goal**: バージョン文字列を `3.5.0` に更新する。

- `fav/Cargo.toml`: `version = "3.5.0"`
- `cargo build` → `fav --version` が `favnir 3.5.0` を返すことを確認

---

## Phase 1: 型登録 + namespace

**Goal**: `GenProfile` 型と `Gen` / `Random.seed` を型システムに追加する。

### 1-A: `GenProfile` 型 (`middle/checker.rs`)

```rust
// stdlib 型として登録
"GenProfile" → { total: Int  valid: Int  invalid: Int  rate: Float }
```

### 1-B: `Gen` namespace (`middle/checker.rs`, `middle/compiler.rs`)

- `checker.rs`: `"Gen"` を stdlib グローバル登録ループに追加
- `compiler.rs`: `"Gen"` を2箇所の登録ループに追加（`"DB"` / `"Env"` の直後）

### 1-C: `Random.seed` 型シグネチャ (`middle/checker.rs`)

```rust
("Random", "seed") => Some(Type::Unit)
```

エフェクト: なし（シード設定は副作用だが外部から観測不可能）

### 1-D: `Gen.*` 型シグネチャ登録 (`middle/checker.rs`)

| 関数 | シグネチャ |
|------|-----------|
| `Gen.string_val` | `Int -> String !Random` |
| `Gen.one_raw` | `String -> Map<String, String> !Random` |
| `Gen.list_raw` | `(String, Int) -> List<Map<String, String>> !Random` |
| `Gen.simulate_raw` | `(String, Int, Float) -> List<Map<String, String>> !Random` |
| `Gen.profile_raw` | `(String, List<Map<String, String>>) -> GenProfile` |

---

## Phase 2: VM プリミティブ (`backend/vm.rs`)

**Goal**: `Random.seed` と `Gen.*` VM プリミティブを実装する。

### 2-A: `Random.seed`

```rust
"Random.seed" => {
    let n = vm_int(args.into_iter().next().unwrap(), "Random.seed")?;
    // スレッドローカル RNG の seed を設定
    // rand の SeedableRng を thread-local に持つ
    RAND_SEED.with(|s| s.set(n as u64));
    Ok(VMValue::Unit)
}
```

注: 現在の `Random.int` / `Random.float` は `rand::thread_rng()` を使っている。
`Random.seed` 設定後は `SmallRng::seed_from_u64` で生成した RNG を
thread-local に持ち、`Random.int` / `Random.float` がそちらを使うよう修正する。

実装パターン:
```rust
thread_local! {
    static RAND_RNG: RefCell<Option<rand::rngs::SmallRng>> = RefCell::new(None);
}
// Random.seed → Some(SmallRng::seed_from_u64(n))
// Random.int → RAND_RNG.with(|r| { if let Some(rng) = r.borrow_mut().as_mut() { rng.gen_range(min..=max) } else { thread_rng().gen_range(...) } })
```

### 2-B: `Gen.string_val`

```rust
"Gen.string_val" => {
    let len = vm_int(args.into_iter().next().unwrap(), "Gen.string_val")? as usize;
    // RAND_RNG → alphanumeric chars
    Ok(VMValue::Str(random_alphanumeric_string(len)))
}
```

```rust
fn random_alphanumeric_string(len: usize) -> String {
    const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    // RAND_RNG があればそちらを使う; なければ thread_rng
    ...
}
```

### 2-C: `Gen.one_raw`

```rust
"Gen.one_raw" => {
    let type_name = vm_string(args.into_iter().next().unwrap(), "Gen.one_raw")?;
    // 1. type_metas から type_name のフィールド一覧を取得
    // 2. 各フィールドの型に応じてランダム値を生成
    // 3. Map<String, String> として返す
    gen_one_raw_impl(&type_name, type_metas)
}
```

```rust
fn gen_one_raw_impl(
    type_name: &str,
    type_metas: &HashMap<String, TypeMeta>,
) -> Result<VMValue, String> {
    let meta = type_metas.get(type_name)
        .ok_or_else(|| format!("Gen.one_raw: unknown type '{type_name}'"))?;
    let mut map = HashMap::new();
    for (field_name, field_type) in &meta.fields {
        let val = gen_value_for_type(field_type)?;
        map.insert(field_name.clone(), val);
    }
    Ok(VMValue::Map(map))
}

fn gen_value_for_type(ty: &str) -> Result<String, String> {
    match ty {
        "Int"    => Ok(random_int(-1000, 1000).to_string()),
        "Float"  => Ok(random_float().to_string()),
        "Bool"   => Ok(if random_bool() { "true" } else { "false" }.to_string()),
        "String" => Ok(random_alphanumeric_string(8)),
        t if t.starts_with("Option<") => {
            // 50% None (empty string), 50% inner type
            if random_bool() { Ok(String::new()) }
            else {
                let inner = &t[7..t.len()-1];
                gen_value_for_type(inner)
            }
        }
        other => Ok(other.to_string()), // fallback: type name as string
    }
}
```

### 2-D: `Gen.list_raw`

```rust
"Gen.list_raw" => {
    let type_name = vm_string(...)?;
    let n = vm_int(...)? as usize;
    let rows: Result<Vec<VMValue>, _> = (0..n)
        .map(|_| gen_one_raw_impl(&type_name, type_metas))
        .collect();
    Ok(VMValue::List(rows?))
}
```

### 2-E: `Gen.simulate_raw`

```rust
"Gen.simulate_raw" => {
    let type_name = vm_string(...)?;
    let n = vm_int(...)? as usize;
    let noise = vm_float(...)? as f64;  // 0.0〜1.0
    // n 件生成し、各フィールドを noise 確率で破損させる
    let rows: Result<Vec<VMValue>, _> = (0..n)
        .map(|_| gen_simulate_one(&type_name, noise, type_metas))
        .collect();
    Ok(VMValue::List(rows?))
}
```

```rust
fn gen_simulate_one(
    type_name: &str,
    noise: f64,
    type_metas: &HashMap<String, TypeMeta>,
) -> Result<VMValue, String> {
    // gen_one_raw_impl の結果に対し、各フィールドを noise 確率で破損値に置き換える
    // 破損値: "", "NULL", "NaN", "-999999" など
}
```

### 2-F: `Gen.profile_raw`

```rust
"Gen.profile_raw" => {
    let type_name = vm_string(...)?;
    let data = vm_list_of_maps(...)?;
    // 各行を type_metas でバリデート
    // GenProfile { total, valid, invalid, rate } を VMValue::Record として返す
    let total = data.len();
    let valid = data.iter().filter(|row| is_valid_row(row, &meta)).count();
    let invalid = total - valid;
    let rate = if total > 0 { valid as f64 / total as f64 } else { 0.0 };
    Ok(record_vm([("total", VMValue::Int(total as i64)),
                  ("valid", VMValue::Int(valid as i64)),
                  ("invalid", VMValue::Int(invalid as i64)),
                  ("rate", VMValue::Float(rate))]))
}
```

---

## Phase 3: `runes/gen/gen.fav`

**Goal**: ユーザー向け rune API を Favnir で実装する。

ファイル: `<repo_root>/runes/gen/gen.fav`

```favnir
// 低レベル生成（Random.* ラッパー）
public fn int_val(min: Int, max: Int) -> Int !Random {
    Random.int(min, max)
}
public fn float_val() -> Float !Random {
    Random.float()
}
public fn bool_val() -> Bool !Random {
    Random.int(0, 1) == 1
}
public fn string_val(len: Int) -> String !Random {
    Gen.string_val(len)
}

// リストからランダム選択
public fn choice(items: List<String>) -> Option<String> !Random {
    if List.length(items) == 0 {
        Option.none()
    } else {
        bind idx <- Random.int(0, List.length(items) - 1)
        List.nth(items, idx)
    }
}

// 型名から生成
public fn one(type_name: String) -> Map<String, String> !Random {
    Gen.one_raw(type_name)
}
public fn list(type_name: String, n: Int, seed: Int)
    -> List<Map<String, String>> !Random {
    Random.seed(seed)
    Gen.list_raw(type_name, n)
}

// ノイズ混入
public fn simulate(type_name: String, n: Int, noise: Float, seed: Int)
    -> List<Map<String, String>> !Random {
    Random.seed(seed)
    Gen.simulate_raw(type_name, n, noise)
}

// プロファイリング
public fn profile(type_name: String, data: List<Map<String, String>>)
    -> GenProfile {
    Gen.profile_raw(type_name, data)
}
```

---

## Phase 4: `runes/gen/gen.test.fav`

**Goal**: seed 固定の determinism テストを含む `gen.test.fav` を作成する。

テスト 8 件:

```
test_one_returns_map_with_correct_fields
test_one_field_types_match_schema
test_list_length_matches_n
test_list_seed_deterministic        — 同 seed で同一結果
test_list_seed_different            — 異 seed で異なる結果
test_choice_returns_item_from_list
test_choice_empty_returns_none
test_simulate_introduces_noise      — noise=1.0 で全フィールドが壊れる
test_profile_valid_data             — 正常データの rate=1.0
test_profile_noise_data             — noise=1.0 データの rate が低い
```

---

## Phase 5: driver.rs 統合テスト

**Goal**: Rust 側の統合テストで gen rune の動作を確認する。

`migrate_tests` モジュール（既存）に追加:

```rust
#[test]
fn gen_rune_one_produces_fields() { ... }

#[test]
fn gen_rune_list_deterministic_with_seed() { ... }

#[test]
fn gen_rune_simulate_introduces_corruption() { ... }

#[test]
fn gen_rune_profile_measures_validity() { ... }

#[test]
fn gen_rune_test_file_passes() {
    let results = run_fav_test_file_with_runes("runes/gen/gen.test.fav");
    assert!(failures.is_empty(), ...);
}
```

---

## Phase 6: `fav check --sample N`

**Goal**: `--sample N` フラグで合成データを使ったパイプライン検証を追加する。

### 6-A: `main.rs` へのフラグ追加

```rust
Some("check") => {
    // 既存フラグに追加
    "--sample" => {
        sample = Some(args.get(i+1).unwrap().parse::<usize>()?);
        i += 2;
    }
    ...
    cmd_check_with_sample(file, no_warn, sample)
}
```

### 6-B: `driver.rs` — `cmd_check_with_sample`

```rust
pub fn cmd_check_with_sample(file: Option<&str>, no_warn: bool, sample: Option<usize>) {
    // 1. 通常の check を実行（型チェック）
    // 2. sample が Some(n) なら:
    //    a. ソースを解析して入力型を特定（stage の最初の引数型 or seq の入力型）
    //    b. gen_list_for_sample(type_name, n) で合成データ生成
    //    c. パイプラインを合成データで試し実行
    //    d. 結果を報告
}
```

簡略実装: `--sample N` はまず「型チェックが通っている場合に N 件の合成 Map データで
パイプラインを試し実行する」機能として実装し、
入力型の自動推定は最初の `stage` の引数型から取る。

テスト:
- `check_sample_runs_with_synthetic_data`
- `check_sample_reports_type_errors`

---

## Phase 7: サンプル + ドキュメント

### 7-A: `fav/examples/gen_demo/`

```
examples/gen_demo/
  fav.toml
  src/main.fav     — gen.list + gen.simulate + gen.profile
```

### 7-B: ドキュメント

- `versions/v3.5.0/langspec.md`
- `versions/v3.5.0/migration-guide.md`（破壊的変更なし）
- `versions/v3.5.0/progress.md`（全 `[x]`）

---

## 依存関係グラフ

```
Phase 0 (version)
    └── Phase 1 (GenProfile 型 + Gen/Random.seed 登録)
            └── Phase 2 (VM プリミティブ)
                    ├── Phase 3 (gen.fav rune)
                    │       └── Phase 4 (gen.test.fav)
                    │               └── Phase 5 (driver.rs 統合テスト)
                    └── Phase 6 (fav check --sample N)   ← Phase 2 完了後
                            └── Phase 7 (examples + docs)
```

Phase 3 / Phase 6 は Phase 2 完了後に並行開発可能。

---

## 実装ノート

### thread-local RNG の修正

`Random.int` / `Random.float` の実装を次のように変更:

```rust
thread_local! {
    // None のとき: thread_rng を使う（既存動作）
    // Some のとき: シード固定 SmallRng を使う
    static SEEDED_RNG: RefCell<Option<rand::rngs::SmallRng>> = RefCell::new(None);
}

pub fn random_int_impl(min: i64, max: i64) -> i64 {
    SEEDED_RNG.with(|r| {
        let mut borrowed = r.borrow_mut();
        if let Some(rng) = borrowed.as_mut() {
            rng.gen_range(min..=max)
        } else {
            rand::thread_rng().gen_range(min..=max)
        }
    })
}
```

`Random.seed(n)`:
```rust
SEEDED_RNG.with(|r| {
    use rand::SeedableRng;
    *r.borrow_mut() = Some(rand::rngs::SmallRng::seed_from_u64(n as u64));
});
```

### type_metas の構造

`Schema.adapt` は既に `type_metas: &HashMap<String, TypeMeta>` を受け取っている。
`Gen.one_raw` も同じ引数を受け取る（`vm_call_builtin` に既に渡されているはず）。

`TypeMeta` のフィールドリストを使って `gen_one_raw_impl` を実装する。
`TypeMeta` の内部構造は `vm.rs` を確認して合わせる。

### `bool_val` の実装注意

`gen.fav` の `bool_val` は `Random.int(0, 1) == 1` を返す。
Favnir で `Int == Int` は `Bool` を返すので、そのまま `-> Bool !Random` になる。

### `choice` の実装注意

`List.nth` は `Option<T>` を返すため、`gen.choice` の戻り値は `Option<String>`。
呼び出し側は `Option.unwrap_or(gen.choice(items), "")` で使う。
