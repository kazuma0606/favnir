# v74.6.0 仕様書 — `fav audit` 拡張（依存関係セキュリティ機能追加）

Date: 2026-08-14

---

## Background

Favnir の `fav audit` は既存のソースコードレベル監査（W008 ambient effect レポート等）を提供している。
本バージョンでは `fav audit --deps` サブフラグとして **Cargo 依存関係のセキュリティスキャン基盤** を追加する。

実際の `cargo audit` CLI 呼び出しや RustSec データベースへのネットワークアクセスはスコープ外とし、
`driver.rs` にデータ構造・レポートフォーマット・バージョン置換関数を実装することで、
将来の統合（`process::Command` 経由の cargo audit 呼び出し等）の土台を作る。

---

## Goals

1. `DepVulnerability` 構造体（name / version / cve / severity / fix_version）を定義する
2. `format_audit_deps_report(vulns: &[DepVulnerability]) -> String` — 脆弱性一覧レポートを生成する
3. `apply_audit_fix(cargo_toml: &str, name: &str, fix_version: &str) -> String` — Cargo.toml 文字列中の依存バージョンを置換する
4. `v746000_tests` モジュール（2 件）を追加する
   - `audit_detects_vulnerable_dep`
   - `audit_fix_updates_cargo_toml`

---

## API / コマンド例

```bash
# 既存: ソースコード監査（変更なし）
$ fav audit pipeline.fav

# 新規追加: Cargo 依存関係のセキュリティスキャン
$ fav audit --deps
Auditing 47 Cargo dependencies...

CRITICAL  libduckdb-sys 1.2.2  CVE-2026-XXXX  Update to 1.3.0
HIGH      tokio 1.38.0         CVE-2026-YYYY  Update to 1.38.1
OK        45 dependencies clean

$ fav audit --deps --fix
Updated: libduckdb-sys 1.2.2 → 1.3.0
Updated: tokio 1.38.0 → 1.38.1
```

### `DepVulnerability` 構造体

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct DepVulnerability {
    pub name: String,         // クレート名（例: "tokio"）
    pub version: String,      // 脆弱なバージョン（例: "1.38.0"）
    pub cve: String,          // CVE 識別子（例: "CVE-2026-1234"）
    /// 有効値: "CRITICAL" | "HIGH" | "MEDIUM" | "LOW"
    pub severity: String,
    pub fix_version: String,  // 修正済みバージョン（例: "1.38.1"）
}
```

### `format_audit_deps_report`

```rust
/// 脆弱性一覧をテキスト形式でフォーマットする
/// 出力形式: "severity  name version  cve  Update to fix_version"（固定幅整形なし）
/// 例: "HIGH  tokio 1.38.0  CVE-2026-1234  Update to 1.38.1"
/// 空スライスは "OK  0 vulnerabilities found" を返す
pub fn format_audit_deps_report(vulns: &[DepVulnerability]) -> String
```

### `apply_audit_fix`

```rust
/// Cargo.toml 文字列中の `name = "version"` を `name = "fix_version"` に置換する
/// 対象パターン: `name = "old_version"` → `name = "fix_version"`（単純文字列形式のみ）
/// インラインテーブル形式（`name = { version = "..." }`）は対象外（将来対応）
/// マッチしない場合は元の文字列をそのまま返す
pub fn apply_audit_fix(cargo_toml: &str, name: &str, fix_version: &str) -> String
```

---

## Success Criteria

1. `audit_detects_vulnerable_dep` テストが pass する
   - `DepVulnerability` を構築し各フィールドを assert
   - `format_audit_deps_report` の出力に severity / name / cve が含まれることを assert
   - 空スライスで "OK" を含む文字列を返すことを assert
2. `audit_fix_updates_cargo_toml` テストが pass する
   - `apply_audit_fix` で Cargo.toml 文字列中のバージョンが置換されることを assert
   - 対象クレートが存在しない場合に元の文字列が返ることを assert
3. `cargo test` で 3682 tests pass（0 failures）

---

## スコープ外（明示的除外）

- `cargo audit` CLI の実際の呼び出し（`process::Command` 統合は後続バージョン）
- RustSec データベースへのネットワークアクセス
- `Cargo.lock` の解析・パース
- `fav audit --deps` の main.rs CLI エントリポイント（後続バージョン）
- `--fix` フラグの実ファイル書き込み（後続バージョン）

---

## Error Codes

新規エラーコードなし

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `DepVulnerability` / `format_audit_deps_report` / `apply_audit_fix` + `v746000_tests` 追加 |
| `fav/Cargo.toml` | `version = "74.6.0"` に更新 |
| `CHANGELOG.md` | v74.6.0 エントリを先頭に追加 |
| `versions/current.md` | 進行中バージョン・次に切る版を更新 |
