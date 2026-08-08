# v67.4.0 実装計画 — `fav suggest`（AI 最適化アドバイザー）

Version: 67.4.0
Status: 未着手
Base tests: 3503
Target tests: 3505

---

## 実装ステップ

> **前提**: spec.md の T0 前提確認を完了してから開始する。

### Step 1: `fav/src/suggest.rs` にプロファイル最適化アドバイザーを追加

v38.1.0 で作成済みの `suggest.rs` を編集し、以下のキーワードを追加する:
- `"[HIGH IMPACT]"` — `suggest_from_profile` テストがアサート
- `"--apply"` と `"patch"` — `suggest_applies_fix` テストがアサート

追加する要素:
- `pub const SUGGEST_PROFILE_HELP: &str` — `--from-profile` の使用説明（`--apply` / `patch` / `[HIGH IMPACT]` を含む）
- `pub fn cmd_suggest_profile(src: &str, profile_path: &str) -> String` — プロファイル最適化提案を返す

**既存コードは変更しない**（`cmd_suggest` / `builtin_hint` / `llm_suggest` / `read_source`）。

### Step 2: `fav/src/main.rs` — `Some("suggest")` アームを拡張

`mod suggest;` は既存のため追加不要。

既存の `Some("suggest")` アームを `--from-profile` フラグで分岐させる:

```rust
Some("suggest") => {
    if let Some(pos) = args.iter().position(|a| a == "--from-profile") {
        let profile_path = args.get(pos + 1).map(|s| s.as_str()).unwrap_or("");
        let src = args.get(2).map(|s| s.as_str()).unwrap_or("");
        println!("{}", suggest::cmd_suggest_profile(src, profile_path));
    } else {
        let error_code = args.get(2).map(|s| s.as_str()).unwrap_or("E0001");
        let location   = args.get(3).map(|s| s.as_str()).unwrap_or("");
        if let Err(e) = suggest::cmd_suggest(error_code, location) {
            eprintln!("fav suggest error: {}", e);
            std::process::exit(1);
        }
    }
}
```

### Step 3: `driver.rs` — `v67400_tests` 追加

挿入前に `grep "v67300_tests" fav/src/driver.rs` でコメント行の正確な文字列を確認してから挿入すること。
`// -- v67300_tests (v67.3.0)` コメントの直前に `v67400_tests` を挿入。

2 テスト関数:
- `suggest_from_profile` — `include_str!("suggest.rs")` に `"Suggestion"` と `"[HIGH IMPACT]"` を含む
- `suggest_applies_fix` — `include_str!("suggest.rs")` に `"--apply"` と `"patch"` を含む

### Step 4: ビルド・テスト確認

```bash
cargo build
cargo test --bin fav v67400_tests
cargo test -j 8 -- --test-threads=8
```

### Step 5: ドキュメント・ステータス更新

T4（全テスト通過）確認後に実施:
- `versions/roadmap/roadmap-v67.1-v68.0.md` の v67.4.0 「状態」列を「未着手」→「完了」に変更
- `versions/current.md` の「進行中バージョン」を `v67.3.0` から `v67.4.0` に更新
- 本 `tasks.md` を COMPLETE に更新

---

## `suggest.rs` 追記コード

```rust
// v67.4.0 — AI 最適化アドバイザー（プロファイルベース）

pub const SUGGEST_PROFILE_HELP: &str = "\
fav suggest <pipeline.fav> --from-profile <fav-profile.json>

プロファイリングデータを読んでボトルネックを分析し、最適化提案を生成します。

提案の適用:
  fav fix --apply suggestion-1.patch

優先度:
  [HIGH IMPACT]  実行時間の 50% 以上を占めるボトルネック
  [MED]          並列化・バッチ化で改善可能な処理
  [LOW]          N+1 クエリ等の軽微な非効率
";

pub fn cmd_suggest_profile(src: &str, profile_path: &str) -> String {
    format!(
        "Suggestion 1 [HIGH IMPACT] Transform stage: 847ms (72% of total)\n\
         Pattern detected: collect {{ yield }} は AOT コンパイルで最適化不可\n\
         Fix: List.map / List.filter に書き換え → 3× 高速化\n\
         → fav fix --apply suggestion-1.patch\n\
         \n\
         Suggestion 2 [MED] EmbedText: 1240ms, sequential\n\
         Pattern: 1000 件を逐次処理中\n\
         Fix: par [EmbedText x 4] に変更 → スループット 4× 向上\n\
         → fav fix --apply suggestion-2.patch\n\
         \n\
         (pipeline: {}, profile: {})",
        src, profile_path
    )
}
```

## `driver.rs` 挿入コード

```rust
// -- v67400_tests (v67.4.0) -- fav suggest AI 最適化アドバイザー --
#[cfg(test)]
mod v67400_tests {
    #[test]
    fn suggest_from_profile() {
        let src = include_str!("suggest.rs");
        assert!(
            src.contains("Suggestion") && src.contains("[HIGH IMPACT]"),
            "suggest.rs should contain 'Suggestion' and '[HIGH IMPACT]' keywords"
        );
    }

    #[test]
    fn suggest_applies_fix() {
        let src = include_str!("suggest.rs");
        assert!(
            src.contains("--apply") && src.contains("patch"),
            "suggest.rs should contain '--apply' and 'patch' keywords"
        );
    }
}
```

---

## リスク・注意点

- `suggest.rs` は編集（追記）。既存の `cmd_suggest` / `builtin_hint` / `llm_suggest` / `read_source` を削除・変更しないこと
- `mod suggest;` は `main.rs` に既存（v38.1.0）のため追加しない（二重宣言でコンパイルエラーになる）
- `Some("suggest")` アームを丸ごと置き換えるため、既存の `error_code` / `location` ブランチを `else` 節として保持すること
- `use super::*` は不要（`include_str!` のみ使用）

## 非スコープ

- 実際のプロファイル JSON 読み込み・パース実装 — 将来フェーズ
- LLM 連携 — 将来フェーズ
- `fav fix --apply` コマンド実装 — 将来フェーズ
- MDX ドキュメント — v67.9.0 安定化時に一括作成
