# v31.3.0 実装計画 — fav explain E0001 コマンド完成

## 前提

- `fav/Cargo.toml` version = `31.2.0`
- `cargo test` — 2430 passed（0 failures）
- v31.2.0 が COMPLETE であること

---

## 実装ステップ

### Step 1: バージョンバンプ

**`fav/Cargo.toml`**
- `version = "31.2.0"` → `version = "31.3.0"`

### Step 2: driver.rs スタブ化

**`fav/src/driver.rs`**
- `v312000_tests::cargo_toml_version_is_31_2_0` をスタブ化（コメント付き）

### Step 3: get_explain_text() 拡充

`get_explain_text()` の既存アームの適切な位置に以下を追加する。

挿入位置と内容:

**E0001 の直後（`"E0007"` アームの前）に追加:**

```rust
"E0002" => Some(
"E0002: Condition type error

条件式（if / while）のガードが Bool 型ではありません。

修正例:
  誤: if 1 { ... }   ← Int は Bool でない → E0002

  正: if n > 0 { ... }   ← OK

関連: E0005（型注釈と推論型の不一致）"
),
"E0003" => Some(
"E0003: Effect not declared

stage / fn でエフェクト（!IO 等）を使っているが、シグネチャに !Effect が宣言されていません。
（checker.fav（self-hosted checker）が検出。Rust checker が検出する同種エラーは E0016。）

修正例:
  誤: stage Load: String -> String = |s| {
        bind _ <- IO.println(s)   ← !IO が宣言にない → E0003
        s
      }

  正: stage Load: String -> String !IO = |s| {
        bind _ <- IO.println(s)   ← OK
        s
      }

関連: E0016（Rust checker 相当）"
),
"E0004" => Some(
"E0004: Non-exhaustive pattern match

match 式がすべてのケースをカバーしていません。

修正例:
  誤: match opt {
        Some(v) => v   ← None が未処理 → E0004
      }

  正: match opt {
        Some(v) => v
        None    => \"default\"
      }

関連: E0006（match arm 型不一致）"
),
"E0005" => Some(
"E0005: Type annotation mismatch

型注釈と実際の式の型が一致しません。

修正例:
  誤: fn count() -> String { 42 }   ← Int を返している → E0005

  正: fn count() -> Int { 42 }   ← OK

関連: E0009（戻り型不一致）"
),
"E0006" => Some(
"E0006: Match arm type mismatch

match の各 arm が異なる型を返しています。

修正例:
  誤: match flag {
        true  => 1        ← Int
        false => \"zero\"   ← String → E0006
      }

  正: match flag {
        true  => \"one\"
        false => \"zero\"   ← 両方 String → OK
      }

関連: E0004（非網羅的パターン）"
),
```

**E0009 の直後（`"E0010"` アームの前）には既に E0010 が `get_help_text` にあるので `get_explain_text` にも追加:**

**E0010:**
```rust
"E0010" => Some(
"E0010: Interface not fully implemented

interface のすべてのメソッドを実装していません。

修正例:
  interface Runnable { fn run(self) -> Unit }

  誤: impl Runnable for MyJob { }   ← run が未実装 → E0010

  正: impl Runnable for MyJob {
        fn run(self) -> Unit = IO.println(\"running\")
      }

関連: E0014（interface not implemented）"
),
```

**E0011:**
```rust
"E0011" => Some(
"E0011: Undefined type

存在しない型名を参照しています。

修正例:
  誤: fn load() -> MyRecord { ... }   ← MyRecord が未定義 → E0011

  正: type MyRecord = { id: Int, name: String }
      fn load() -> MyRecord { ... }   ← OK

関連: E0001（未定義変数）"
),
```

**E0018 の直後（`"W001"` アームの前）に追加:**

```rust
"E0019" => Some(
"E0019: Circular interface inheritance

interface の継承が循環しています。

修正例:
  誤: interface A : B { }
      interface B : A { }   ← 循環 → E0019

  正: interface Base { fn id(self) -> Int }
      interface A : Base { fn name(self) -> String }   ← OK"
),
"E0020" => Some(
"E0020: Capability not available

ctx に渡された型が必要な capability interface を実装していません。

修正例:
  誤: fn load(ctx: LoadCtx) -> Result<Rows, String> {
        ctx.db.execute(\"DELETE ...\")   ← LoadCtx.db は DbRead のみ → E0020
      }

  正: fn delete(ctx: WriteCtx) -> Result<Unit, String> {
        ctx.db.execute(\"DELETE ...\")   ← WriteCtx.db は DbWrite → OK
      }

関連: E0021（wrong context type）"
),
"E0021" => Some(
"E0021: Wrong context type

ctx に必要なフィールドが存在しません。

修正例:
  誤: fn run(ctx: LoadCtx) {
        ctx.storage.put(...)   ← LoadCtx に storage フィールドがない → E0021
      }

  正: fn run(ctx: WriteCtx) {
        ctx.storage.put(...)   ← WriteCtx に storage フィールドがある → OK
      }

関連: E0020（capability not available）"
),
```

### Step 4: cmd_explain_code() の unknown 時改善

現在:
```rust
None => {
    eprintln!("unknown error code: {}", code);
    process::exit(1);
}
```

変更後:
```rust
None => {
    eprintln!("error: unknown error code `{}`", code);
    eprintln!();
    eprintln!("Available codes (E0001-E0021):");
    for c in &["E0001","E0002","E0003","E0004","E0005","E0006","E0007",
                "E0008","E0009","E0010","E0011","E0012","E0013","E0014",
                "E0015","E0016","E0017","E0018","E0019","E0020","E0021"] {
        eprintln!("  {}", c);
    }
    eprintln!();
    eprintln!("  use `fav explain <code>` to see details");
    std::process::exit(1);
}
```

### Step 5: v313000_tests 追加

v312000_tests の直前に追加:

```rust
// ── v31.3.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v313000_tests {
    use super::*;
    #[test]
    fn cargo_toml_version_is_31_3_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("version = \"31.3.0\""), "Cargo.toml must contain version = \"31.3.0\"");
    }
    #[test]
    fn benchmark_v31_3_0_exists() {
        let src = include_str!("../../benchmarks/v31.3.0.json");
        assert!(src.contains("31.3.0"), "benchmarks/v31.3.0.json must contain '31.3.0'");
    }
    #[test]
    fn get_explain_text_e0002_through_e0021() {
        for code in &["E0002","E0003","E0004","E0005","E0006","E0010","E0011","E0019","E0020","E0021"] {
            assert!(
                get_explain_text(code).is_some(),
                "get_explain_text({}) must return Some(...)", code
            );
        }
    }
}
```

### Step 6: CHANGELOG.md 追記

```markdown
## [v31.3.0] — 2026-07-02

### Added
- `driver.rs::get_explain_text()` — E0002/E0003/E0004/E0005/E0006/E0010/E0011/E0019/E0020/E0021 の説明テキストを追加
- `benchmarks/v31.3.0.json` 追加

### Changed
- `driver.rs::cmd_explain_code()` — unknown コード時に利用可能コード一覧（E0001〜E0021）を表示
- `Cargo.toml` version: `31.2.0` → `31.3.0`
```

### Step 7: benchmarks/v31.3.0.json 作成

```json
{
  "version": "31.3.0",
  "date": "2026-07-02",
  "milestone": "Real-World Readiness",
  "tests_passed": 2433,
  "tests_failed": 0,
  "notes": "fav explain E0001-E0021 complete + unknown code list"
}
```

> `tests_passed` は `cargo test` 実行後に実測値で更新する（+3 件 = 2433 想定）。
> **T12 で必ず実測値に書き換えること。** 上記の 2433 は暫定値。

### Step 8: versions/current.md 更新

- 「最新安定版」欄を v31.3.0 に更新
- 「次に切る版」を `v31.4.0 — TBD` に更新

---

## ファイル変更一覧

| ファイル | 種別 | 変更内容 |
|---|---|---|
| `fav/Cargo.toml` | 更新 | version `31.2.0` → `31.3.0` |
| `fav/src/driver.rs` | 更新 | v312000 スタブ化 + get_explain_text 拡充 + cmd_explain_code 改善 + v313000_tests |
| `CHANGELOG.md` | 更新 | [v31.3.0] セクション追加 |
| `benchmarks/v31.3.0.json` | 新規 | ベンチマーク結果 |
| `versions/current.md` | 更新 | v31.3.0 に更新 |

---

## 完了判定

- `cargo test v313000` — 3/3 PASS
- `cargo test` — 全件 PASS（0 failures）
