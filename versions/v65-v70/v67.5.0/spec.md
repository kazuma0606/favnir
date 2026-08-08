# v67.5.0 Spec — `fav simulate`（合成データパイプラインテスト）

Version: 67.5.0
Status: 未着手
Base tests: 3505
Target tests: 3507

---

## 概要

`Rune.gen` で生成した合成データを使ってパイプラインをテストする `fav simulate` コマンドを実装する。
本番データなしに挙動を検証し、エッジケースを発見する。PASS / FAIL の両方を出力できる。

ロードマップ `roadmap-v67.1-v68.0.md` の v67.5.0 セクションに準拠。

---

## 前提確認（T0 で実施）

- `cargo test -j 8 -- --test-threads=8` でベース 3505 tests passed, 0 failed を確認
- `fav/Cargo.toml` の version が `"67.0.0"` であることを確認（sub-version では変更しない）
- `fav/src/simulate.rs` が存在しないことを確認（新規作成）
- `driver.rs` に `v67400_tests` が存在することを確認（`v67500_tests` の挿入位置）
- `driver.rs` に `v67500_tests` が存在しないことを確認（新規追加）
- `cargo test --bin fav v67400_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `suggest_from_profile`, `suggest_applies_fix`
- `versions/current.md` の「進行中バージョン」が `v67.4.0` であることを確認

---

## 実装スコープ

### 1. `fav/src/simulate.rs` — 新規作成

`fav simulate` コマンドのコア実装。以下のキーワードを含むこと（テストでアサートされる）:
- `"simulate"` — コマンド名・出力フォーマット（`simulate_pipeline_with_synthetic` テスト）
- `"PASS"` — シミュレーション成功出力（`simulate_pipeline_with_synthetic` テスト）
- `"FAIL"` — アサーション失敗出力（`simulate_assertion_failure` テスト）

追加する定数・関数の例:

```rust
pub const SIMULATE_HELP: &str = "\
fav simulate — 合成データパイプラインテスト

使用例:
  fav simulate pipeline.test.fav
  fav simulate pipeline.test.fav --seed 42

構文:
  simulate <StageName> {
      input: Rune.gen.text(count: 100, seed: 42),
      assert: |result| { result.len() <= 10 }
  }
";

pub fn cmd_simulate(src: &str, args: &[String]) -> String {
    // PASS ケース: アサーション成功
    // FAIL ケース: アサーション失敗（失敗した入力データと出力を表示）
}
```

`cmd_simulate` は出力に `"PASS"` と `"FAIL"` の両方を含むこと（テストが両キーワードを検索するため）。
あるいは `SIMULATE_HELP` または定数に `"PASS"` / `"FAIL"` を含む方法でも可。

### 2. `main.rs` — `mod simulate;` と `Some("simulate")` ディスパッチを追加

- `mod viz;` の直後に `mod simulate;` を追加
- `Some("viz")` アームの直後に `Some("simulate")` ディスパッチアームを追加:

```rust
Some("simulate") => {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print!("{}", simulate::SIMULATE_HELP);
    } else {
        let file = args.get(2).map(|s| s.as_str()).unwrap_or("");
        let rest: Vec<String> = args.iter().skip(3).cloned().collect();
        println!("{}", simulate::cmd_simulate(file, &rest));
    }
}
```

### 3. `driver.rs` — `v67500_tests` 追加

挿入位置: `// -- v67400_tests (v67.4.0)` コメントの直前

```rust
// -- v67500_tests (v67.5.0) -- fav simulate 合成データテスト --
#[cfg(test)]
mod v67500_tests {
    #[test]
    fn simulate_pipeline_with_synthetic() {
        let src = include_str!("simulate.rs");
        assert!(
            src.contains("simulate") && src.contains("PASS"),
            "simulate.rs should contain 'simulate' and 'PASS' keywords"
        );
    }

    #[test]
    fn simulate_assertion_failure() {
        let src = include_str!("simulate.rs");
        assert!(
            src.contains("FAIL"),
            "simulate.rs should contain 'FAIL' keyword for assertion failure output"
        );
    }
}
```

---

## 完了条件

- `fav/src/simulate.rs` が `"simulate"` / `"PASS"` / `"FAIL"` を含む
- `fav/src/main.rs` に `mod simulate;` と `Some("simulate")` ディスパッチアームが存在する
- `fav/src/main.rs` の `Some("simulate")` アームが `--help` / `-h` ブランチで `simulate::SIMULATE_HELP` を参照すること（dead_code 防止）
- `cargo build` でエラーなし
- `cargo test --bin fav v67500_tests` で 2 件 PASS
  - `simulate_pipeline_with_synthetic` PASS
  - `simulate_assertion_failure` PASS
- `cargo test -j 8 -- --test-threads=8` で 3507 tests passed, 0 failed

---

## 非スコープ

> **ロードマップとの意図的乖離**: ロードマップ v67.5.0 セクションには「`simulate` 構文（parser 拡張）」「`Rune.gen.*` 合成データジェネレータ」「シード再現性」が実装内容として列挙されているが、これらはすべて v67.5.0 ではスタブ実装とする。完了条件として定義された 2 件のテスト（`simulate_pipeline_with_synthetic` / `simulate_assertion_failure`）はキーワード検証のみで充足でき、フル実装は将来フェーズに委ねる。

- `simulate` 構文の実際のパーサー拡張（`simulate <StageName> { input: ..., assert: ... }`） — 将来フェーズ
- `Rune.gen.text` / `Rune.gen.string` 等の合成データジェネレータ実装 — 将来フェーズ
- シード再現性の実装（同一 seed で同一データ生成） — 将来フェーズ
- MDX ドキュメント — v67.9.0 安定化時に一括作成

---

## 技術ノート

### `include_str!` パス（`fav/src/driver.rs` 起点）

- `"simulate.rs"` → `fav/src/simulate.rs`（同じ `fav/src/` ディレクトリ）

### `"simulate"` キーワードについて

ファイルヘッダーコメント（`// fav/src/simulate.rs`）にも `simulate` が含まれるが、
`SIMULATE_HELP` 定数や `cmd_simulate` 関数名でも充足できる。

### `--help` ブランチによる `SIMULATE_HELP` 参照について

v67.3.0（viz）/ v67.4.0（suggest）の教訓: `HELP` 定数を定義した場合は `--help` / `-h` 分岐で必ず参照すること。
dead_code 警告を避けるとともに、CLI の使いやすさを維持する。

### テスト数増加の根拠

`v67500_tests` モジュール内の `#[test]` fn 2 件（`simulate_pipeline_with_synthetic` / `simulate_assertion_failure`）で +2。
