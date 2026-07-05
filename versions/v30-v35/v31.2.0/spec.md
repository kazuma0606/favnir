# v31.2.0 仕様書 — typo 候補ユーティリティ + E0011〜E0019 hint 追加

## 概要

ロードマップ v31.2「typo 候補（Levenshtein）+ エラーコード全件 URL」に対応する。

- `levenshtein()` / `suggest_similar()` ユーティリティ関数を `driver.rs` に追加する
- `get_help_text()` に E0011/E0012/E0016/E0017/E0019 のヒントを追加する
- エラーコード URL は `format_diagnostic()` (driver.rs:93) で全コード実装済みのため変更不要

---

## 背景

ロードマップ v31.2 より:

```rust
// Levenshtein 距離で候補を検索
fn suggest_similar(name: &str, candidates: &[&str]) -> Vec<&str> {
    candidates.iter()
        .filter(|c| levenshtein(name, c) <= 2)
        .take(3)
        .collect()
}
```

適用箇所（ロードマップ）:
- E0001（未定義変数）→ スコープ内の変数名から候補
- E0007（未定義関数）→ 定義済み関数名から候補
- E0011（未定義型）→ 定義済み型名から候補

---

## 既存実装の確認事項

| 項目 | 状態 |
|---|---|
| `format_diagnostic()` URL 出力 (driver.rs:93) | **実装済み** — 全コードに `= 参照: https://favnir.dev/errors/EXXXX` |
| `get_help_text()` E0011 | **未設定** |
| `get_help_text()` E0012 | **未設定** |
| `get_help_text()` E0016 | **未設定** |
| `get_help_text()` E0017 | **未設定** |
| `get_help_text()` E0019 | **未設定** |
| `levenshtein()` 関数 | 存在しない — 新規追加対象 |
| `suggest_similar()` 関数 | 存在しない — 新規追加対象 |

---

## スコープ

### IN SCOPE

- `fav/Cargo.toml` — version `31.1.0` → `31.2.0`
- `fav/src/driver.rs` — `cargo_toml_version_is_31_1_0` をスタブ化
- `fav/src/driver.rs` — `levenshtein(s: &str, t: &str) -> usize` 関数を追加
  - Wagner-Fischer アルゴリズムによる編集距離計算
  - `#[cfg(test)]` 外（ユーティリティ関数）に配置
- `fav/src/driver.rs` — `suggest_similar<'a>(name: &str, candidates: &[&'a str]) -> Vec<&'a str>` 関数を追加
  - 距離 ≤ 2 かつ最大 3 件を返す
- `fav/src/driver.rs` — `get_help_text()` に以下を追加:
  - E0011: `"check the type name for typos; use \`fav doc --builtins\` to list built-in types"`
  - E0012: `"check that the expression type matches the expected type"`
  - E0016: `"add the missing effect to the function signature: \`fn foo() -> T !IO { ... }\`"`
  - E0017: `"remove the unused effect declaration from the function signature"`
  - E0019: `"remove the circular interface inheritance"`
- `fav/src/driver.rs` — `v312000_tests`（4 件）追加（`use super::*` あり）
- `CHANGELOG.md` — `[v31.2.0]` セクション追加
- `benchmarks/v31.2.0.json` 新規作成
- `versions/current.md` — v31.2.0 に更新

### OUT OF SCOPE

- checker.fav への Levenshtein 統合（動的な「〇〇の typo ですか？」表示）
  — checker.fav の修正が必要なため将来バージョンで実施
- E0022〜E0320 へのヒント追加 — 順次追加予定
- `fav explain` コマンド — v31.3.0 で実施
- site/ MDX 更新 — v32.0 マイルストーン宣言時に実施

---

## 実装詳細

### levenshtein() 関数

距離計算の根拠:
- `levenshtein("kitten", "sitting") = 3` — k→s(置換1) + e→i(置換1) + \0→g(挿入1)
- `levenshtein("user_id", "user_id2") = 1` — 末尾に "2" を1文字挿入
- `levenshtein("user_id", "userId") = 2` — `_`削除(1) + `i`→`I`置換(1) （境界値: ≤2 に含まれる）
- `levenshtein("user_id", "order_id") = 3` — `u→o`+`s→r`+`e→d`+`r→e`... DPで距離3 → 除外

```rust
fn levenshtein(s: &str, t: &str) -> usize {
    let s: Vec<char> = s.chars().collect();
    let t: Vec<char> = t.chars().collect();
    let m = s.len();
    let n = t.len();
    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for i in 0..=m { dp[i][0] = i; }
    for j in 0..=n { dp[0][j] = j; }
    for i in 1..=m {
        for j in 1..=n {
            dp[i][j] = if s[i-1] == t[j-1] {
                dp[i-1][j-1]
            } else {
                1 + dp[i-1][j].min(dp[i][j-1]).min(dp[i-1][j-1])
            };
        }
    }
    dp[m][n]
}
```

### suggest_similar() 関数

> 注意: 現バージョンでは距離順ソートを行わない（候補は `candidates` の入力順）。
> 将来の E0001/E0007 checker 統合時に `sort_by_key(|c| levenshtein(name, c))` を追加予定。

```rust
fn suggest_similar<'a>(name: &str, candidates: &[&'a str]) -> Vec<&'a str> {
    let mut result: Vec<&'a str> = candidates
        .iter()
        .copied()
        .filter(|c| levenshtein(name, c) <= 2)
        .collect();
    result.truncate(3);
    result
}
```

---

## テスト設計（v312000_tests — 4 件）

| # | テスト名 | 確認内容 |
|---|---------|----------|
| 1 | `cargo_toml_version_is_31_2_0` | `Cargo.toml` に `version = "31.2.0"` |
| 2 | `benchmark_v31_2_0_exists` | `benchmarks/v31.2.0.json` に `"31.2.0"` |
| 3 | `levenshtein_distance_basic` | `levenshtein("kitten", "sitting") == 3` |
| 4 | `suggest_similar_finds_close_match` | `suggest_similar("user_id", &["user_id2", "userId", "order_id"])` が `["user_id2", "userId"]` を含む |

> `v312000_tests` は `use super::*` あり（`levenshtein` / `suggest_similar` / `get_help_text` は非 pub 関数のため）。

---

## 完了条件

- `Cargo.toml` version = `"31.2.0"`
- `levenshtein()` と `suggest_similar()` が `driver.rs` に追加されている
- `get_help_text()` が E0011/E0012/E0016/E0017/E0019 に非空スライスを返す
- `cargo test v312000` — 4/4 PASS
- `cargo test` — 全件 PASS（0 failures）
- `CHANGELOG.md` に `[v31.2.0]` セクション
- `benchmarks/v31.2.0.json` 存在
- `benchmarks/v31.2.0.json` の `tests_passed` が実測値（`cargo test` 後）で記録されていること
- `versions/current.md` を v31.2.0 に更新
- `tasks.md` が COMPLETE
