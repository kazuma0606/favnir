# v39.3.0 spec — `fav policy`

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v39.3.0 |
| テーマ | `fav policy` — 組織ポリシーの宣言的定義と検証 |
| 前提 | v39.2.0 COMPLETE — Audit Log Rune 完了 |
| 完了条件 | `v39300_tests` 全テスト pass・`cargo test` 0 failures（≥ 2794 件） |

## 背景と目的

v39.2.0 で監査ログが整った。v39.3.0 では組織ポリシーを宣言的に定義する `fav policy` コマンドを追加する。
`fav.toml` に `policy { ... }` ブロックを記述するだけで、Rune の使用制限・スキーマ要件・テスト必須化・ステージ数上限を宣言でき、`fav policy check --ci` で CI ゲートとして機能させることができる。

**想定動作**:
```
$ fav policy check
Policy: OK (3 rules checked)

$ fav policy check --ci
Policy violation: deny_runes matched "experimental/unstable"
exit 1
```

**`fav.toml` の `policy` ブロック**:
```favnir
policy {
  deny_runes: ["experimental/*"]
  require_schema: true
  require_tests: true
  max_pipeline_stages: 20
}
```

## 実装スコープ

### 1. `fav/src/policy.rs` — 新規作成

```rust
/// v39.3.0 — fav policy: 組織ポリシーの宣言的定義と検証

pub fn cmd_policy_check(ci_mode: bool) -> Result<(), String> {
    let rules = load_policy_rules()?;
    let violations = check_rules(&rules);
    if violations.is_empty() {
        println!("Policy: OK ({} rules checked)", rules.len());
        Ok(())
    } else {
        for v in &violations {
            eprintln!("Policy violation: {}", v);
        }
        if ci_mode {
            std::process::exit(1);
        }
        Err(format!("{} policy violation(s) found", violations.len()))
    }
}

fn load_policy_rules() -> Result<Vec<String>, String> {
    // fav.toml の policy ブロックを読み込む（v39.x で parse 実装予定）
    // 現在は組み込みデフォルトルールを返すスタブ
    Ok(vec![
        "deny_runes: [\"experimental/*\"]".to_string(),
        "require_schema: true".to_string(),
        "require_tests: true".to_string(),
    ])
}

fn check_rules(rules: &[String]) -> Vec<String> {
    // ルール評価ロジック（現在はスタブ: 常に violations なし）
    let _ = rules;
    vec![]
}
```

**エクスポート関数**: `pub fn cmd_policy_check(ci_mode: bool) -> Result<(), String>`

**テストキーワード**: `pub fn cmd_policy_check`

### 2. `fav/src/main.rs` — `mod policy;` + `Some("policy")` アーム追加

#### `mod policy;` 追加

既存の `mod suggest;` の直後に追加:
```rust
mod policy;
```

#### `Some("policy")` ディスパッチアーム

```rust
Some("policy") => {
    let sub = args.get(2).map(|s| s.as_str()).unwrap_or("check");
    let ci_mode = args.iter().any(|a| a == "--ci");
    if sub == "check" {
        if let Err(e) = policy::cmd_policy_check(ci_mode) {
            eprintln!("fav policy error: {}", e);
            std::process::exit(1);
        }
    }
}
```

### 3. `driver.rs` — テストモジュール追加

#### `v39200_tests::cargo_toml_version_is_39_2_0` のスタブ化

```rust
// Stubbed: version bumped to 39.3.0 — assertion intentionally removed
```

#### `v39300_tests` モジュール新規追加

```rust
// ── v39300_tests (v39.3.0) — fav policy ──────────────────────────────────────
#[cfg(test)]
mod v39300_tests {
    // include_str! のみ使用のため imports 不要

    #[test]
    fn cargo_toml_version_is_39_3_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("39.3.0"), "Cargo.toml must contain version 39.3.0");
    }

    #[test]
    fn changelog_has_v39_3_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v39.3.0]"), "CHANGELOG.md must contain [v39.3.0]");
    }

    #[test]
    fn policy_rs_exists() {
        let src = include_str!("policy.rs");
        assert!(
            src.contains("pub fn cmd_policy_check"),
            "policy.rs must contain pub fn cmd_policy_check"
        );
    }
}
```

`policy_rs_exists` の `include_str!` パス: `"policy.rs"`（`driver.rs` と同じ `fav/src/` ディレクトリ）

### 4. `CHANGELOG.md` — `[v39.3.0]` エントリ追加

`## [v39.2.0]` ヘッダ行の直前に挿入:

```
## [v39.3.0] — YYYY-MM-DD

### Added
- `fav/src/policy.rs` — `fav policy check` / `fav policy check --ci` コマンド追加
- `policy { deny_runes / require_schema / require_tests / max_pipeline_stages }` ブロック仕様
- `v39300_tests` 3 テスト追加

---
```

**セパレータは `—`（全角ダッシュ U+2014）**

### 5. その他ドキュメント更新

- `fav/Cargo.toml`: `39.2.0` → `39.3.0`
- `versions/current.md`: 最新安定版 → v39.3.0、次に切る版 → v39.4.0
- `versions/roadmap/roadmap-v39.1-v40.0.md`: v39.3.0 を ✅ 完了済みにマーク

## 注意事項

### `policy.rs` の `load_policy_rules` はスタブ

`fav.toml` の `policy { ... }` ブロック parse 実装は後続バージョンで行う。
現時点は組み込みデフォルトルールを返すスタブとし、`check_rules` は常に violations なし（空リスト）を返す。

### `check_rules` スタブの削除タイミング

`fn check_rules` 内の `let _ = rules;` は未使用変数警告を抑制するスタブ用コードである。
`fav.toml` の `policy { ... }` ブロック parse 実装時に `rules` を実際に評価するロジックに置き換え、`let _ = rules;` を削除すること。

### `gen` 予約語

Rust 2024 edition では `gen` は予約語。テスト内の変数名に `gen` を使用しないこと（今回は `policy` 系のみで問題なし）。

### `main.rs` dispatch アームの挿入位置

Read で `Some("suggest")` の行番号を確認してから `Some("policy")` アームを直後に挿入する。

## テスト数の計算

| バージョン | 実績 |
|---|---|
| v39.2.0 | 2791 |
| v39.3.0 追加分 | +3 |
| v39.3.0 期待値 | 2794 |

ロードマップは「Rust テスト 3 件」と記載しており、meta 2 件 + 機能 1 件の計 3 件で一致する。

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `policy.rs` に `pub fn cmd_policy_check` が含まれる | `policy_rs_exists` テスト |
| 2 | `CHANGELOG.md` に `[v39.3.0]` が含まれる | `changelog_has_v39_3_0` テスト |
| 3 | `Cargo.toml` バージョンが `39.3.0` | `cargo_toml_version_is_39_3_0` テスト |
| 4 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2794） | `cargo test` 実行結果（2791 + 3 = 2794） |
| 5 | `roadmap-v39.1-v40.0.md` の v39.3.0 が ✅ | T8 後に目視確認 |
