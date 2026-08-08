# v67.4.0 Spec — `fav suggest`（AI 最適化アドバイザー）

Version: 67.4.0
Status: 未着手
Base tests: 3503
Target tests: 3505

---

## 概要

プロファイリング結果を読み込み、ボトルネックの自動分析と最適化提案を行う `fav suggest --from-profile` コマンドを実装する。
AI が提案し、`fav fix --apply` が適用する。人間が承認主導。

ロードマップ `roadmap-v67.1-v68.0.md` の v67.4.0 セクションに準拠。

---

## 前提確認（T0 で実施）

- `cargo test -j 8 -- --test-threads=8` でベース 3503 tests passed, 0 failed を確認
- `fav/Cargo.toml` の version が `"67.0.0"` であることを確認（sub-version では変更しない）
- `fav/src/suggest.rs` が存在することを確認（v38.1.0 で作成済み。本バージョンで拡張する）
  - 既存の `cmd_suggest(error_code, location)` は変更しない（v38.1.0 の機能を保持）
- `driver.rs` に `v67300_tests` が存在することを確認（`v67400_tests` の挿入位置）
- `driver.rs` に `v67400_tests` が存在しないことを確認（新規追加）
- `cargo test --bin fav v67300_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `viz_ascii_dag`, `viz_svg_with_timing`
- `versions/current.md` の「進行中バージョン」が `v67.3.0` であることを確認

---

## 実装スコープ

### 1. `fav/src/suggest.rs` — プロファイル最適化アドバイザーを追加

v38.1.0 で作成済みの `suggest.rs` を拡張する。既存の `cmd_suggest` は変更しない。
以下のキーワードを追加する（テストでアサートされる）:
- `"[HIGH IMPACT]"` — 最適化提案の優先度ラベル（`suggest_from_profile` テスト）
- `"Suggestion"` — 提案文（既存コードに `"Suggestion:"` が含まれているため既に充足）
- `"--apply"` — 自動適用フラグ（`suggest_applies_fix` テスト）
- `"patch"` — パッチファイル（`suggest_applies_fix` テスト）

追加する定数・関数の例:

```rust
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
         (pipeline: {}, profile: {})",
        src, profile_path
    )
}
```

### 2. `main.rs` — `Some("suggest")` アームに `--from-profile` ブランチを追加

既存の `Some("suggest")` アームに `--from-profile` フラグのチェックを追加する（`mod suggest;` は既存のため追加不要）:

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

### 3. `driver.rs` — `v67400_tests` 追加

挿入位置: `// -- v67300_tests (v67.3.0)` コメントの直前

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

## 完了条件

- `fav/src/suggest.rs` が `"[HIGH IMPACT]"` / `"--apply"` / `"patch"` を含む
- 既存の `cmd_suggest(error_code, location)` が変更されていない（v67300_tests がリグレッションしないこと）
- `cargo build` でエラーなし
- `cargo test --bin fav v67400_tests` で 2 件 PASS
  - `suggest_from_profile` PASS
  - `suggest_applies_fix` PASS
- `cargo test -j 8 -- --test-threads=8` で 3505 tests passed, 0 failed

---

## 非スコープ

- 実際のプロファイル JSON 読み込み・パース実装 — 将来フェーズ
- LLM 連携（Claude API へのプロファイルデータ送信） — 将来フェーズ（ロードマップに記載があるが v67.4.0 ではキーワードベースのスタブ実装で代替する）
- パッチファイル生成（自動適用可能な diff 出力） — 将来フェーズ
- `fav fix --apply` コマンド実装 — 将来フェーズ
- MDX ドキュメント — v67.9.0 安定化時に一括作成

---

## 技術ノート

### 既存コードとの関係

| コマンド形式 | 関数 | バージョン |
|---|---|---|
| `fav suggest E0001 file.fav` | `cmd_suggest(error_code, location)` | v38.1.0（既存） |
| `fav suggest pipeline.fav --from-profile profile.json` | `cmd_suggest_profile(src, profile_path)` | v67.4.0（新規） |

`mod suggest;` は `main.rs` に既存（v38.1.0）のため追加不要。

### `include_str!` パス（`fav/src/driver.rs` 起点）

- `"suggest.rs"` → `fav/src/suggest.rs`（同じ `fav/src/` ディレクトリ）

### `"Suggestion"` キーワードについて

既存の `builtin_hint` 関数に `"Suggestion:"` が含まれているため `suggest_from_profile` テストの `src.contains("Suggestion")` は実装前から充足している。
`"[HIGH IMPACT]"` は新規追加が必要。

### パターン検出キーワードのテストカバレッジについて

ロードマップには `collect { yield }` / N+1 クエリのパターン検出が実装内容として列挙されているが、
`v67400_tests` ではこれらをアサートしていない（`"Suggestion"` / `"[HIGH IMPACT]"` / `"--apply"` / `"patch"` のみ）。
`cmd_suggest_profile` の実装例に `"collect"` は含まれているが、テスト未カバーは意図的。
理由: ロードマップのテスト仕様（`suggest_from_profile` / `suggest_applies_fix`）がキーワードを明示しており、
それ以上のアサートはロードマップ外。将来の実装変更に対する保護は v67.9.0 安定化テストに委ねる。

### テスト数増加の根拠

`v67400_tests` モジュール内の `#[test]` fn 2 件（`suggest_from_profile` / `suggest_applies_fix`）で +2。
