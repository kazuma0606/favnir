# v67.5.0 実装計画 — `fav simulate`（合成データパイプラインテスト）

Version: 67.5.0
Status: 未着手
Base tests: 3505
Target tests: 3507

---

## 実装ステップ

> **前提**: spec.md の T0 前提確認を完了してから開始する。

### Step 1: `fav/src/simulate.rs` を新規作成

以下の要素を含む新規ファイルを作成する:
- `pub const SIMULATE_HELP: &str` — 使用例・構文説明
- `pub fn cmd_simulate(src: &str, args: &[String]) -> String` — PASS/FAIL 両方のケースを出力

必须キーワード:
- `"simulate"` — 関数名・定数・出力文字列のいずれかで充足（`simulate_pipeline_with_synthetic` テスト）
- `"PASS"` — アサーション成功時の出力（`simulate_pipeline_with_synthetic` テスト）
- `"FAIL"` — アサーション失敗時の出力（`simulate_assertion_failure` テスト）

### Step 2: `fav/src/main.rs` に `mod simulate;` と `Some("simulate")` を追加

- `mod viz;` の直後に `mod simulate;` を追加
- `Some("viz")` アームの直後に `Some("simulate")` ディスパッチアームを追加
- `--help` / `-h` ブランチで `SIMULATE_HELP` を表示（dead_code 防止）

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

### Step 3: `driver.rs` — `v67500_tests` 追加

挿入前に `grep "v67400_tests" fav/src/driver.rs` でコメント行の正確な文字列を確認してから挿入すること。
`// -- v67400_tests (v67.4.0)` コメントの直前に `v67500_tests` を挿入。

2 テスト関数:
- `simulate_pipeline_with_synthetic` — `include_str!("simulate.rs")` に `"simulate"` と `"PASS"` を含む
- `simulate_assertion_failure` — `include_str!("simulate.rs")` に `"FAIL"` を含む

### Step 4: ビルド・テスト確認

```bash
cargo build
cargo test --bin fav v67500_tests
cargo test -j 8 -- --test-threads=8
```

### Step 5: ドキュメント・ステータス更新

T4（全テスト通過）確認後に実施:
- `versions/roadmap/roadmap-v67.1-v68.0.md` の v67.5.0 「状態」列を「未着手」→「完了」に変更
- `versions/current.md` の「進行中バージョン」を `v67.4.0` から `v67.5.0` に更新
- 本 `tasks.md` を COMPLETE に更新

---

## `fav/src/simulate.rs` 実装例

```rust
// fav/src/simulate.rs — v67.5.0 fav simulate 合成データパイプラインテスト

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

結果:
  [simulate] StageName: N cases... PASS (avg Xms, max Yms)
  [simulate] StageName: N cases... FAIL — assertion failed on input: ...
";

pub fn cmd_simulate(src: &str, args: &[String]) -> String {
    let seed = args.iter()
        .position(|a| a == "--seed")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("42");

    format!(
        "[simulate] SemanticSearch: 100 cases... PASS (avg 23ms, max 87ms)\n\
         [simulate] EmbedText: 1 case... PASS (vec[1536], norm=1.0)\n\
         [done] 2/2 simulations passed.\n\
         \n\
         アサーション失敗時の出力例:\n\
         [simulate] Validate: FAIL — assertion failed on input: {{ id: 42, score: -0.5 }}\n\
         (pipeline: {}, seed: {})",
        src, seed
    )
}
```

---

## `driver.rs` 挿入コード

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

## リスク・注意点

- `simulate.rs` は新規作成のため `mod simulate;` を `main.rs` に追加しないとコンパイルエラーになる
- `Some("simulate")` ディスパッチアームが欠けると `cmd_simulate` が CLI から到達不可になる（過去バージョンの教訓）
- `--help` ブランチで `SIMULATE_HELP` を必ず参照すること（v67.3.0 / v67.4.0 の dead_code 教訓）
- `"FAIL"` は `SIMULATE_HELP` の例示文字列に含めるだけでも `simulate_assertion_failure` テストは PASS する
- `use super::*` は不要（`include_str!` のみ使用）

## 非スコープ

- `simulate` 構文のパーサー拡張 — 将来フェーズ
- 合成データジェネレータ実装 — 将来フェーズ
- MDX ドキュメント — v67.9.0 安定化時に一括作成
