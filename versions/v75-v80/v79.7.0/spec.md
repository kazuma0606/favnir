# v79.7.0 仕様書 — OSS 公開強化・コミュニティ整備

Date: 2026-08-16
Status: PLANNING

---

## Background

v79.6.0 でドッグフーディング強化を完了した。
v79.7.0 では OSS コントリビュートフローを強化し、コミュニティ基盤を整備する。

具体的には:
- `CONTRIBUTING.md` を v3 対応に更新（Execution Effects 追加手順・`fav verify` の使い方）
- `COMMUNITY.md` を新規作成（RFC プロセス・ディスカッション場所）

> **Note**: テスト数はベース 3799（v79.6.0 完了後の実測値）。完了後は 3801。

---

## Goals

- `CONTRIBUTING.md` に v3 機能追加手順（Execution Effects / `fav verify` / invariant 追加手順）を追記する
- `COMMUNITY.md` を新規作成し、RFC プロセスとディスカッション場所を記述する
- Rust テスト 2 件でファイル内容を検証する

> **スコープ外**: `.github/CODEOWNERS` 更新・Rune validate ガイド（`validate_rune_score`）はロードマップの `**実装内容:**` に含まれないため本バージョンでは対象外とする。

---

## `CONTRIBUTING.md` 追記内容

既存の CONTRIBUTING.md 末尾に以下のセクションを追加:

```markdown
## Execution Effects の追加手順（v3 対応）

新しいエフェクト（`!MyEffect`）を追加する場合は以下の手順に従ってください:

1. `fav/src/ast.rs` に `Effect::MyEffect` バリアントを追加
2. `fav/src/middle/checker.rs` の `ns_to_effect` / `builtin_ret_ty` を更新
3. `fav/src/backend/cranelift_aot.rs` のマッチアームを更新
4. `fav/pipelines/health-check.fav` を使ってヘルスチェックを実行

## PipelineInvariant（invariant）の追加手順

`contract` ブロックに新しい不変条件（invariant）を追加する場合:

1. `infra/e2e-demo/` の `contract.fav` に `invariant:` 節を追記する
2. `fav verify <contract.fav>` で静的検証を確認する

## fav verify の使い方

```bash
fav verify <pipeline.fav>
```

`fav verify` はパイプラインの不変条件（`contract` ブロック）を静的検証します。
CI での使用を推奨します。
```

---

## `COMMUNITY.md` 内容

```markdown
# Favnir コミュニティ

## RFC プロセス

新機能の提案は RFC（Request for Comments）プロセスを経て承認されます。

1. `versions/roadmap/` に RFC 草稿を作成する
2. GitHub Issues でディスカッションを開始する
3. コアチームのレビュー後、ロードマップに組み込む

## ディスカッション場所

- **GitHub Issues**: バグ報告・機能要望
- **GitHub Discussions**: 設計議論・RFC

## 行動規範

すべての参加者は [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) に従ってください。
```

---

## テストモジュール仕様

```rust
// --- v79.7.0: OSS 公開強化・コミュニティ整備 ---
#[cfg(test)]
mod v797000_tests {
    const CONTRIBUTING: &str = include_str!("../../CONTRIBUTING.md");
    const COMMUNITY:    &str = include_str!("../../COMMUNITY.md");

    #[test]
    fn oss_contributing_v2_exists() {
        assert!(CONTRIBUTING.contains("Execution Effects"), "CONTRIBUTING.md must mention Execution Effects");
        assert!(CONTRIBUTING.contains("fav verify"), "CONTRIBUTING.md must mention fav verify");
        assert!(CONTRIBUTING.contains("invariant"), "CONTRIBUTING.md must mention invariant");
    }

    #[test]
    fn oss_community_md_exists() {
        assert!(COMMUNITY.contains("RFC"), "COMMUNITY.md must describe RFC process");
        assert!(COMMUNITY.contains("GitHub"), "COMMUNITY.md must mention GitHub");
    }
}
```

注意: `use super::*` 不要（`include_str!` + `assert!` のみ）。`const CONTRIBUTING` / `const COMMUNITY` パターンを採用。

---

## CHANGELOG エントリ形式

```
## [v79.7.0] — 2026-08-16 — OSS 公開強化・コミュニティ整備

### Added
- `CONTRIBUTING.md`: v3 対応更新（Execution Effects 追加手順・fav verify の使い方）
- `COMMUNITY.md`: 新規作成（RFC プロセス・ディスカッション場所）

### Tests
- `oss_contributing_v2_exists`: CONTRIBUTING.md に Execution Effects / fav verify が含まれることを検証
- `oss_community_md_exists`: COMMUNITY.md に RFC / GitHub が含まれることを検証
```

---

## Success Criteria

- `cargo test v797000` で 2 件が pass
- `cargo test` で 3801 tests pass（0 failures）
- `CONTRIBUTING.md` に `Execution Effects` / `fav verify` / `invariant` が存在する
- `COMMUNITY.md` に `RFC` / `GitHub` が存在する

---

## Files to modify

| ファイル | 変更内容 |
|---|---|
| `CONTRIBUTING.md` | v3 対応セクション追記（Execution Effects / fav verify）|
| `COMMUNITY.md` | 新規作成（RFC プロセス・ディスカッション場所）|
| `fav/src/driver.rs` | `v797000_tests` モジュール追加（末尾）|
| `fav/Cargo.toml` | `version = "79.7.0"` に更新 |
| `fav/Cargo.lock` | version バンプに追随して自動更新 |
| `CHANGELOG.md` | v79.7.0 エントリを先頭に追加 |
| `versions/current.md` | 進行中バージョンを更新 |
