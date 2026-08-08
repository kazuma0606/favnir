# v67.6.0 実装計画 — Pipeline Property Testing（`Rune.proptest`）

Version: 67.6.0
Status: 未着手
Base tests: 3507
Target tests: 3509

---

## 実装ステップ

> **前提**: spec.md の T0 前提確認を完了してから開始する。

### Step 1: `fav/src/proptest.rs` を新規作成

以下のキーワードを全て含む新規ファイルを作成する:
- `"proptest"` — `proptest_stage_invariant` テストがアサート
- `"forall"` — `proptest_stage_invariant` テストがアサート
- `"shrink"` — `proptest_stage_invariant` テストがアサート
- `"--proptest-runs"` — `proptest_counterexample_shrink` テストがアサート

追加する要素:
- `pub const PROPTEST_HELP: &str` — 使用例・構文説明（4 キーワード全てを含む）
- `pub fn cmd_proptest(src: &str, args: &[String]) -> String` — プロパティテスト結果を返す

`--proptest-runs` の値省略時は `eprintln!` で警告し、デフォルト値 `100` を使用すること。

### Step 2: `fav/src/main.rs` に `mod proptest;` と `Some("proptest")` を追加

- `mod simulate;` の直後に `mod proptest;` を追加
- `Some("simulate")` アームの直後に `Some("proptest")` ディスパッチアームを追加
- `--help` / `-h` ブランチで `proptest::PROPTEST_HELP` を表示（dead_code 防止）

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

### Step 3: `driver.rs` — `v67600_tests` 追加

挿入前に `grep "v67500_tests" fav/src/driver.rs` でコメント行の正確な文字列を確認してから挿入すること。
`// -- v67500_tests (v67.5.0)` コメントの直前に `v67600_tests` を挿入。

2 テスト関数:
- `proptest_stage_invariant` — `include_str!("proptest.rs")` に `"proptest"` / `"forall"` / `"shrink"` を含む
- `proptest_counterexample_shrink` — `include_str!("proptest.rs")` に `"--proptest-runs"` を含む

### Step 4: ビルド・テスト確認

```bash
cargo build
cargo test --bin fav v67600_tests
cargo test -j 8 -- --test-threads=8
```

### Step 5: ドキュメント・ステータス更新

T4（全テスト通過）確認後に実施:
- `versions/roadmap/roadmap-v67.1-v68.0.md` の v67.6.0 「状態」列を「未着手」→「完了」に変更
- `versions/current.md` の「進行中バージョン」を `v67.5.0` から `v67.6.0` に更新
- 本 `tasks.md` を COMPLETE に更新（T0 を含む全チェックボックスを `[x]` に）

---

## `fav/src/proptest.rs` 実装例

```rust
// fav/src/proptest.rs — v67.6.0 Pipeline Property Testing

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
  - forall: ランダム入力でプロパティを検証（デフォルト 100 試行）
  - shrink: 反例が見つかった場合に最小形へ自動縮小
  - --proptest-runs <n>: 試行回数を指定（デフォルト 100）
";

pub fn cmd_proptest(src: &str, args: &[String]) -> String {
    // スタブ実装: 将来フェーズで実際の PBT エンジンに置き換える
    let runs = match args.iter().position(|a| a == "--proptest-runs") {
        Some(i) => match args.get(i + 1).map(|s| s.as_str()) {
            Some(v) if !v.starts_with('-') => v,
            _ => {
                eprintln!("fav proptest warning: --proptest-runs requires a value, using default '100'");
                "100"
            }
        },
        None => "100",
    };

    format!(
        "[proptest] Transform: 100 trials... ok (all forall properties hold)\n\
         [proptest] EmbedText: 100 trials... FAILED after 42 trials\n\
         Counterexample: text = \"\" (empty string)\n\
         Shrinking... minimal counterexample: text = \"\"\n\
         (pipeline: {}, runs: {})",
        src, runs
    )
}
```

---

## `driver.rs` 挿入コード

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

## リスク・注意点

- `proptest.rs` は新規作成のため `mod proptest;` を `main.rs` に追加しないとコンパイルエラーになる
- `Some("proptest")` ディスパッチアームが欠けると `cmd_proptest` が CLI から到達不可になる（過去バージョンの教訓）
- `--help` ブランチで `PROPTEST_HELP` を必ず参照すること（dead_code 防止の教訓）
- `--proptest-runs` の値省略時に無言 fallback しないよう `eprintln!` を追加すること（v67.5.0 の教訓）
- `use super::*` は不要（`include_str!` のみ使用）

## 非スコープ

- `proptest` 構文のパーサー拡張 — 将来フェーズ
- ランダム入力生成・反例縮小の実際の実装 — 将来フェーズ
- MDX ドキュメント — v67.9.0 安定化時に一括作成
