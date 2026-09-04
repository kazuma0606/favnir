# Spec: v100.0.0 — Favnir SAP Platform 1.0 宣言 ★大クリーンアップ

## Background

v99.1〜v99.9 のコードフリーズが完了した。
v100.0.0 は Favnir SAP Platform 1.0 の公式宣言バージョンであり、以下を行う:

1. Cargo.toml の version を `100.0.0` に更新（宣言バージョン）
2. MILESTONE.md / README.md に SAP Platform 1.0 宣言エントリを追加
3. `mod v100000_tests`（4 テスト）を追加し 4,279 テストに到達させる
4. ★大クリーンアップ（`cargo clean` → `cargo test` 再確認）
5. 全 SAP ロードマップファイルの Status を「完了」に更新

**テストモジュール命名規則（v100.0.0 初・注意）**:
- モジュール名: `v100000_tests`（6 桁、`v` + `100` + `000`）
- 関数名: `cargo_toml_version_is_100_0_0`（アンダースコア区切り）

**宣言文**:

> 「Favnir が、SAP のプラットフォームになった。
>
>  `$delta` で差分を受け取り、Event Mesh でリアルタイムに動き、
>  `ctx.sap_env("PRD")` で本番に向き、
>  Snowflake と型安全に JOIN し、
>  `!Approval` で人間の承認を型に閉じ込め、
>  KPI が SAC に流れ、Slack が鳴り、
>  `Masked<T>` が個人情報を守り、
>  `!Audit` が証跡を刻む。
>
>  OAuth2 が認証し、Circuit Breaker が守り、SLA が測る。
>
>  これが、Favnir SAP Platform 1.0 である。
>  SAP と Favnir の 5 年間の旅が、今、完成した。」

## Goals

1. `fav/Cargo.toml` の version を `100.0.0` に更新する
2. `MILESTONE.md` に v100.0.0 — SAP Platform 1.0 宣言エントリを追加する
3. `README.md` に `## v100.0 — Favnir SAP Platform 1.0` セクションを追加する
4. `CHANGELOG.md` に `[v100.0.0]` エントリを追加する
5. `fav/src/driver.rs` に `mod v100000_tests`（4 テスト）を追加する
6. `cargo clean` → `cargo test`（4,279 tests）で大クリーンアップ完了を確認する
7. `versions/roadmap/roadmap-v99.1-v100.0.md` の Status を「完了」に更新する

## mod v100000_tests 仕様

```rust
#[cfg(test)]
mod v100000_tests {
    // use super::* は不要（外部シンボル未使用）
    #[test]
    fn cargo_toml_version_is_100_0_0() {
        let content = include_str!("../Cargo.toml");
        assert!(
            content.contains("100.0.0"),
            "Cargo.toml version should be 100.0.0 (v100.0.0)"
        );
    }
    #[test]
    fn changelog_has_v100_0_0() {
        let content = std::fs::read_to_string("../CHANGELOG.md")
            .expect("CHANGELOG.md should exist (v100.0.0)");
        assert!(
            content.contains("[v100.0.0]"),
            "CHANGELOG.md should have [v100.0.0] entry (v100.0.0)"
        );
    }
    #[test]
    fn milestone_has_sap_platform() {
        let content = std::fs::read_to_string("../MILESTONE.md")
            .expect("MILESTONE.md should exist (v100.0.0)");
        assert!(
            content.contains("SAP Platform"),
            "MILESTONE.md should mention SAP Platform (v100.0.0)"
        );
    }
    #[test]
    fn readme_mentions_sap_platform() {
        let content = std::fs::read_to_string("../README.md")
            .expect("README.md should exist (v100.0.0)");
        assert!(
            content.contains("SAP Platform"),
            "README.md should mention SAP Platform (v100.0.0)"
        );
    }
}
```

## Success Criteria

- `fav/Cargo.toml` の version が `"100.0.0"` である
- `CHANGELOG.md` に `[v100.0.0]` エントリが存在する
- `MILESTONE.md` に `SAP Platform` が含まれる
- `README.md` に `SAP Platform` が含まれる
- `mod v100000_tests` の 4 テストがすべて pass する
- 合計テスト数: 4,279（4,275 + 4）
- `cargo clean` 後に `cargo test` が 4,279 tests, 0 failures で完了する
- `versions/roadmap/roadmap-v95.1-v100.0.md` の Status が「完了」になっていること
- `fav/tmp/hello.fav` が `cargo clean` 後も存在する
- `cargo clippy --locked -- -D warnings` pass
- `./target/debug/fav fmt --check self/compiler.fav` pass
- `./target/debug/fav fmt --check self/checker.fav` pass

## Files to Modify

| ファイル | 操作 |
|---|---|
| `fav/Cargo.toml` | version を `100.0.0` に更新 |
| `MILESTONE.md` | v100.0.0 — SAP Platform 1.0 宣言エントリ追加 |
| `README.md` | `## v100.0 — Favnir SAP Platform 1.0` セクション追加 |
| `CHANGELOG.md` | `[v100.0.0]` エントリ追加 |
| `fav/src/driver.rs` | `mod v100000_tests`（4 テスト）追加 |
| `versions/current.md` | v100.0.0 に更新 |
| `versions/roadmap/roadmap-v99.1-v100.0.md` | Status を「完了」に更新 |
| `versions/roadmap/roadmap-v95.1-v100.0.md` | Status を「完了」に更新 |
