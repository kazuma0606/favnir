# v32.0.0 — 実装計画: Language Polish マイルストーン宣言

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | version `31.9.0` → `32.0.0` |
| `fav/src/driver.rs` | `cargo_toml_version_is_31_9_0` スタブ化、`v320000_tests` 追加 |
| `MILESTONE.md` | v32.0.0「Language Polish」セクションを先頭に追加 |
| `README.md` | v32.0 マイルストーン行を v31.0 行の直後に追加 |
| `CHANGELOG.md` | `[v32.0.0]` セクション追加 |
| `benchmarks/v32.0.0.json` | 新規作成 |
| `versions/current.md` | 最新安定版を v32.0.0 に更新 |

---

## 実装手順

### Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml` の `version = "31.9.0"` を `"32.0.0"` に変更。

---

### Step 2: `cargo_toml_version_is_31_9_0` スタブ化

driver.rs の `v319000_tests` 内の `cargo_toml_version_is_31_9_0` テストを空スタブに変更する。

```rust
#[test]
fn cargo_toml_version_is_31_9_0() {
    // stubbed: version has advanced to 32.0.0
}
```

**重要**: `v319000_tests` には `use super::*;` があり、これは `repl_add_history_skips_blank_lines`
テストが `ReplSession::new()` を使うために必要。スタブ化で `cargo_toml_version_is_31_9_0`
の本体（`include_str!` + `assert!`）のみを削除し、`use super::*;` は残すこと。

---

### Step 3: `v320000_tests` 追加

`v319000_tests` の閉じ括弧（`}`）の直後、かつ `// ── v31.7.0 tests` コメントの前に追加する。
挿入マーカー: `// ── v31.7.0 tests` 行の直前（`v319000_tests` の `}` の次の行）。

**重要**: `v320000_tests` は `use super::*` **なし**（`include_str!` のみ使用）。
`v31.0.0`（v310000_tests）と同じパターンを踏襲すること。

```rust
// ── v32.0.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v320000_tests {
    #[test]
    fn cargo_toml_version_is_32_0_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("32.0.0"), "Cargo.toml must contain '32.0.0'");
    }
    #[test]
    fn milestone_language_polish_declared() {
        let src = include_str!("../../MILESTONE.md");
        assert!(src.contains("Language Polish"), "MILESTONE.md must contain 'Language Polish'");
    }
    #[test]
    fn readme_mentions_v32_0() {
        let src = include_str!("../../README.md");
        assert!(src.contains("v32.0"), "README.md must contain 'v32.0'");
    }
    #[test]
    fn benchmark_v32_0_0_exists() {
        let src = include_str!("../../benchmarks/v32.0.0.json");
        assert!(src.contains("32.0.0"), "benchmarks/v32.0.0.json must contain '32.0.0'");
    }
}
```

---

### Step 4: `MILESTONE.md` 更新

先頭（`# Favnir Milestones` の直後）に以下を追記する。

```markdown
## v32.0.0 — Language Polish（2026-07-03）

> 「Favnir を初めて使うデータエンジニアが、エラーメッセージを見て
>  自力でコードを修正し、30 分以内に最初のパイプラインを動かせること」
> = Language Polish の完成を象徴する定義

v32.0.0 をもって、Favnir の **Language Polish** を正式に宣言する。

エラーメッセージが rustc スタイル（`-->` ファイル位置 + `|` ソース行 + `= ヒント:`）に刷新され、
typo 候補（Levenshtein ≤ 2）と全エラーコード URL が付与された。
`fav explain E0001` でエラーの説明・修正例がターミナルで確認できる。
REPL は `:doc` / `:load` / `:history` / `:save` コマンドとタブ補完を備え、
データ探索ツールとして実用レベルに達した。
LSP Inlay Hints により `bind` 変数の型推論結果がエディタでインライン表示される。
`fav test --watch` と `fav check --all` / `fav scaffold` が揃い、
「書いていて気持ちいい」開発体験を達成した。

### 達成コンポーネント（v31.1〜v31.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| エラーメッセージ v2 | v31.1 | rustc スタイル・E0001〜E0021 全件 hint: 付与 |
| typo 候補 + URL | v31.2 | Levenshtein ≤ 2 候補提示・全エラーコード URL |
| fav explain | v31.3 | `fav explain E0001〜E0021` 説明・修正例出力 |
| REPL 品質向上 | v31.4 | :doc / :load / :history / :save / タブ補完 |
| LSP Inlay Hints | v31.5 | bind 変数の型推論結果インライン表示 |
| fav test --watch | v31.6 | ファイル変更で自動テスト再実行 |
| fav check --all | v31.7 | プロジェクト全体クロスファイルチェック |
| fav scaffold | v31.8 | stage / seq スタブを既存プロジェクトに追記 |
| ドッグフード修正 vol.2 | v31.9 | REPL 空行スキップ / check --all 空ディレクトリ警告 |

**宣言日**: 2026-07-03
**宣言バージョン**: v32.0.0

---
```

---

### Step 5: `README.md` 更新

v31.0 行（`**v31.0（2026-07-02）で、[Real-World Readiness]...`）の直後に追加する。

```markdown
**v32.0（2026-07-03）で、[Language Polish](./MILESTONE.md) マイルストーンを宣言しました。**
エラーメッセージが rustc スタイルに刷新され、`fav explain E0001` でエラー詳細を確認できます。REPL が `:doc` / `:history` / タブ補完を備え、`fav test --watch` / `fav check --all` / `fav scaffold` が揃いました。
```

---

### Step 6: `CHANGELOG.md` 更新

先頭に追記:

```markdown
## [v32.0.0] — 2026-07-03

### Added
- Language Polish マイルストーン宣言（v31.1〜v31.9 全コンポーネント完成）
- `MILESTONE.md` に v32.0.0「Language Polish」セクション追加
- `README.md` に v32.0 マイルストーン行追加
- `cargo clean` + `cargo build` 実施（マイルストーン版クリーンアップ）
```

---

### Step 7: `benchmarks/v32.0.0.json` 作成

```json
{
  "version": "32.0.0",
  "date": "2026-07-03",
  "milestone": "Language Polish",
  "tests_passed": 2456,
  "tests_failed": 0,
  "notes": "Language Polish milestone declaration; cargo clean + rebuild"
}
```

`tests_passed` は実測後に更新する（暫定: 2452 + 4 = 2456）。

---

### Step 8: `versions/current.md` 更新

「最新安定版」欄を v32.0.0 に更新し、「進行中バージョン」を「なし（v32.0.0 完了直後）」、
「次に切る版」を v32.1.0 に変更する。

マイルストーンテーブルの `v32.0 — Language Polish` 行を `**完了**` に更新する。

---

### Step 9: cargo clean + hello.fav 復元 + cargo build

マイルストーン版の必須クリーンアップ:

```bash
cd /c/Users/yoshi/favnir/fav
cargo clean
```

その後 `fav/tmp/hello.fav` を復元（`cargo clean` で削除される）:

```
fn add(a: Int, b: Int) -> Int {
    a + b
}

fn main() -> Bool {
    add(1, 2) == 3
}
```

その後ビルド確認:

```bash
cargo test --bin fav v320000 2>&1 | tail -8   # 4/4 PASS を確認
cargo test 2>&1 | grep "test result"          # 全件 PASS を確認
```
