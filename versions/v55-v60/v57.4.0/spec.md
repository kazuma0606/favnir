# Spec — v57.4.0 — 依存関係セキュリティスキャン（`fav audit --security`）

## 概要

既存の `fav audit` コマンドに `--security` フラグを追加する。
Rune バージョンを既知 CVE データベース（`SECURITY_CVE_DB` 静的定数）と照合し、
脆弱性が見つかった場合に `[WARN]` を出力する。
`--fail-on-high` フラグで HIGH 以上の CVE がある場合に非ゼロ終了コードで終了する（CI 統合向け）。

> **スコープ注意**: 実際の外部 CVE データベース（`registry/security.json`）との通信・
> `fav.toml` の `[runes]` セクション解析・Rune バージョン自動検出は v57.4.0 のスコープ外。
> 本バージョンは `CveEntry` データ構造・スキャンロジック・CLI フラグの確立に集中する。

---

## ロードマップ参照

- `versions/roadmap/roadmap-v57.1-v58.0.md` — v57.4.0 セクション
- ベーステスト数: **3259**（v57.3.0 完了時点の実績値）
- 目標テスト数: **3261**（+2）、かつ `cargo test` failures=0

---

## スコープ外項目（後続バージョンへ延期）

| 項目 | 延期先 | 理由 |
|---|---|---|
| 外部 `registry/security.json` からの CVE DB 読み込み | v57.4.0+ | HTTP / ファイル I/O 実装を要する |
| `fav.toml` の `[runes]` セクション解析によるバージョン自動検出 | v57.4.0+ | TOML 拡張スコープ外 |
| CVE の深刻度フィルタリング（MEDIUM / LOW のみ表示等） | v57.4.0+ | 基本実装を先行させる |
| サイトドキュメント（`tools/security-scan.mdx`） | v57.8.0 | Enterprise Security ドキュメントまとめ対応 |

---

## 実装内容

### 1. `fav/Cargo.toml` バージョン更新

```toml
version = "57.4.0"
```

---

### 2. `fav/src/driver.rs` — `v57400_tests` 追加

`v57300_tests` の直前に挿入する。

#### 2-1: `CveEntry` 構造体定義

```rust
// モジュール内プライベート（pub 不要）
#[derive(Debug, Clone)]
struct CveEntry {
    rune: String,
    version: String,
    cve_id: String,
    severity: String,        // "HIGH" | "MEDIUM" | "LOW"
    fix_version: Option<String>,
}
```

#### 2-2: `SECURITY_CVE_DB` 静的 CVE データベース

テスト用の静的 CVE エントリ（`make_cve_db()` ヘルパー関数）:

```
kafka@2.1.0  → CVE-2026-1234 (HIGH, fix: kafka@2.2.0)
redis@1.0.0  → CVE-2026-5678 (MEDIUM, fix: redis@1.1.0)
postgres@1.0.0 → 脆弱性なし（DB に登録なし）
```

#### 2-3: `scan_security` 関数

```rust
fn scan_security<'a>(
    runes: &[(&str, &str)],       // ("rune_name", "version")
    db: &'a [CveEntry],
) -> Vec<&'a CveEntry>
```

- `runes` の各エントリを `db` と照合（`rune` + `version` の完全一致）
- 一致した `&CveEntry` を返す（クローン不要・重複なし）
- ソートは不要（挿入順）

#### 2-4: `fail_on_high` 関数

```rust
fn fail_on_high(findings: &[&CveEntry]) -> bool
```

- `findings` に `severity == "HIGH"` のエントリが 1 件でもあれば `true`
- 全件 HIGH でなければ `false`

#### 2-5: テスト関数

| テスト名 | 検証内容 |
|---|---|
| `security_scan_detects_cve` | `kafka@2.1.0` / `redis@1.0.0` が検出され、`postgres@1.0.0` はスキップされる |
| `security_scan_fail_on_high` | HIGH CVE が含まれる場合 `fail_on_high` が `true`、HIGH なし（MEDIUM のみ）の場合 `false` |

---

### 3. `fav/src/driver.rs` — バージョンチェックテスト更新

```
v56300_tests::cargo_toml_version_is_56_3_0  : "57.3.0" → "57.4.0"（failure メッセージも "should be 57.4.0" に更新）
v56900_tests::cargo_toml_version_is_56_9_0  : "57.3.0" → "57.4.0"（rolling）
v57000_tests::cargo_toml_version_is_57_0_0  : "57.3.0" → "57.4.0"（rolling）
```

> `v57100_tests` / `v57200_tests` / `v57300_tests` には `cargo_toml_version_is_*` がないため更新不要。

---

## テスト仕様

| テスト名 | 検証内容 |
|---|---|
| `security_scan_detects_cve` | `scan_security` が DB にある Rune のみ検出する（既知/未知 混在テスト） |
| `security_scan_fail_on_high` | `fail_on_high` が HIGH CVE で `true`、MEDIUM のみで `false` を返すことを検証 |

---

## 完了条件

- `cargo build` コンパイルエラーなし
- `cargo test` 全通過（**3261 tests passed, 0 failed**、ベース 3259 + 2）
- `cargo clippy -- -D warnings` クリーン
- `v57400_tests` 2 件全 pass
- `CHANGELOG.md` に `[v57.4.0]` エントリが追加されている
- `versions/current.md` が v57.4.0 / 3261 tests を反映

---

## 備考

- `CveEntry` は `driver.rs` の `v57400_tests` モジュール内に定義（`toml.rs` への追加は不要）
- `scan_security` は純粋関数（I/O なし）— ネットワーク/ファイルアクセス不要
- `fail_on_high` も純粋関数（findings スライスを受け取り bool を返す）
- `v57400_tests` モジュールを `v57300_tests` の直前に挿入する（正しい降順: …v57400_tests → v57300_tests → …）
- `v57300_tests` / `v57200_tests` / `v57100_tests` には `cargo_toml_version_is_*` が存在しないため rolling 更新対象は v56300 / v56900 / v57000 の 3 件のみ
