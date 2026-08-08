# v67.6.0 Spec — Pipeline Property Testing（`Rune.proptest`）

Version: 67.6.0
Status: 未着手
Base tests: 3507
Target tests: 3509

---

## 概要

プロパティベーステスト（PBT）でパイプラインの不変条件を検証する `fav proptest` コマンドを実装する。
ランダム入力でエッジケースを自動探索し、最小反例を自動縮小（shrink）する。

ロードマップ `roadmap-v67.1-v68.0.md` の v67.6.0 セクションに準拠。

---

## 前提確認（T0 で実施）

- `cargo test -j 8 -- --test-threads=8` でベース 3507 tests passed, 0 failed を確認
- `fav/Cargo.toml` の version が `"67.0.0"` であることを確認（sub-version では変更しない）
- `fav/src/proptest.rs` が存在しないことを確認（新規作成）
- `driver.rs` に `v67500_tests` が存在することを確認（`v67600_tests` の挿入位置）
- `driver.rs` に `v67600_tests` が存在しないことを確認（新規追加）
- `cargo test --bin fav v67500_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `simulate_pipeline_with_synthetic`, `simulate_assertion_failure`
- `versions/current.md` の「進行中バージョン」が `v67.5.0` であることを確認

---

## 実装スコープ

### 1. `fav/src/proptest.rs` — 新規作成

`fav proptest` コマンドのコア実装。以下のキーワードを含むこと（テストでアサートされる）:
- `"proptest"` — コマンド名・定数・出力文字列（`proptest_stage_invariant` テスト）
- `"forall"` — プロパティ定義構文（`proptest_stage_invariant` テスト）
- `"shrink"` — 反例縮小機能（`proptest_stage_invariant` テスト）
- `"--proptest-runs"` — 試行回数フラグ（`proptest_counterexample_shrink` テスト）

追加する定数・関数:

```rust
pub const PROPTEST_HELP: &str = "\
fav proptest — パイプラインプロパティテスト

使用例:
  fav proptest pipeline.test.fav
  fav proptest pipeline.test.fav --proptest-runs 200

構文（将来実装予定）:
  proptest stage <StageName> {
      forall x: Int where x > 0 { Transform(x) > 0 }
  }

機能:
  - forall: ランダム入力でプロパティを検証
  - shrink: 反例を最小形に自動縮小
  - --proptest-runs <n>: 試行回数（デフォルト 100）
";

pub fn cmd_proptest(src: &str, args: &[String]) -> String {
    // ...
}
```

### 2. `main.rs` — `mod proptest;` と `Some("proptest")` ディスパッチを追加

- `mod simulate;` の直後に `mod proptest;` を追加
- `Some("simulate")` アームの直後に `Some("proptest")` ディスパッチアームを追加:

```rust
Some("proptest") => {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print!("{}", proptest::PROPTEST_HELP);
    } else {
        let file = args.get(2).map(|s| s.as_str()).unwrap_or("");
        let rest: Vec<String> = args.iter().skip(3).cloned().collect();
        println!("{}", proptest::cmd_proptest(file, &rest));
    }
}
```

### 3. `driver.rs` — `v67600_tests` 追加

挿入位置: `// -- v67500_tests (v67.5.0)` コメントの直前

```rust
// -- v67600_tests (v67.6.0) -- Pipeline Property Testing --
#[cfg(test)]
mod v67600_tests {
    #[test]
    fn proptest_stage_invariant() {
        let src = include_str!("proptest.rs");
        assert!(
            src.contains("proptest") && src.contains("forall") && src.contains("shrink"),
            "proptest.rs should contain 'proptest', 'forall', and 'shrink' keywords"
        );
    }

    #[test]
    fn proptest_counterexample_shrink() {
        let src = include_str!("proptest.rs");
        assert!(
            src.contains("--proptest-runs"),
            "proptest.rs should contain '--proptest-runs' keyword"
        );
    }
}
```

---

## 完了条件

- `fav/src/proptest.rs` が `"proptest"` / `"forall"` / `"shrink"` / `"--proptest-runs"` を含む
- `fav/src/proptest.rs` の `cmd_proptest` が `--proptest-runs` 値省略時に `eprintln!` 警告を出しデフォルト `100` を使用すること
- `fav/src/main.rs` に `mod proptest;` と `Some("proptest")` ディスパッチアームが存在する
- `fav/src/main.rs` の `Some("proptest")` アームが `--help` / `-h` ブランチで `proptest::PROPTEST_HELP` を参照すること（dead_code 防止）
- `cargo build` でエラーなし
- `cargo test --bin fav v67600_tests` で 2 件 PASS
  - `proptest_stage_invariant` PASS
  - `proptest_counterexample_shrink` PASS
- `cargo test -j 8 -- --test-threads=8` で 3509 tests passed, 0 failed

---

## 非スコープ

> **ロードマップとの意図的乖離**: ロードマップ v67.6.0 セクションには「`proptest` 構文（parser 拡張）」「ランダム入力生成」「反例縮小（shrink）の実際の実装」「型別ジェネレータ」が記載されているが、v67.6.0 ではキーワードを含むスタブ実装で代替する。

- `proptest` 構文の実際のパーサー拡張 — 将来フェーズ
- ランダム入力生成（`forall x: T` でランダムサンプリング） — 将来フェーズ
- 反例縮小（shrink）の実際のアルゴリズム実装 — 将来フェーズ
- 型別ジェネレータ（`Int`, `Float`, `String`, `List<T>`, `Record`） — 将来フェーズ
- MDX ドキュメント — v67.9.0 安定化時に一括作成

---

## 技術ノート

### `include_str!` パス（`fav/src/driver.rs` 起点）

- `"proptest.rs"` → `fav/src/proptest.rs`（同じ `fav/src/` ディレクトリ）

### `"proptest"` キーワードの充足

`PROPTEST_HELP` 定数名・`cmd_proptest` 関数名・出力文字列のいずれかで充足できる。
ファイルヘッダーコメント（`// fav/src/proptest.rs`）にも `proptest` が含まれる。

### `--proptest-runs` の処理

v67.5.0（simulate）の教訓: `--proptest-runs` の値が省略された場合に無言 fallback しないよう、
省略時に `eprintln!` で警告を出し、デフォルト値（`100`）を使用する。

### テスト数増加の根拠

`v67600_tests` モジュール内の `#[test]` fn 2 件（`proptest_stage_invariant` / `proptest_counterexample_shrink`）で +2。
