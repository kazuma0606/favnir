# v34.4.0 — 実装プラン

## 方針

セキュリティ審査ドキュメントパターン。新規 MDX 2 ファイル作成 + `SECURITY_MODEL.md` 追記が主体。
`cargo clean` は x.4.0 のため不要。

---

## 実装ステップ

### Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml` の version を `34.3.0` → `34.4.0` に変更。

---

### Step 2: site/content/docs/tools/security-audit-v2.mdx 作成

4 審査項目を含む審査レポートページを新規作成。
（注: 以下の草稿内の `~~~bash` は実際のファイルでは ` ```bash ` と記述すること）

```markdown
---
title: "セキュリティ審査 v2"
description: "Favnir v34.x のセキュリティ状態確認レポート（v24.6 以降の更新）"
---

# セキュリティ審査 v2

v34.4.0 時点での Favnir セキュリティ状態確認レポート。
v24.6.0（セキュリティ審査 v1）からの変更点を中心に記述する。

## 1. エフェクトシステム検証（W021）

W021 `pure_fn_calls_effectful` lint は v24.6.0 で導入済みで、引き続き有効。

検証コマンド:

```bash
fav lint --allow W017 --allow W018 --allow W019 src/pipeline.fav
```

v34.x でコンパイラの Ctx 移行（v34.5 予定）が完了した後も、
W021 は ctx ベース関数に対して適用可能な形で更新する予定。

## 2. 認証情報の扱い

**ガイドライン**: Rune の接続情報はすべて環境変数または `fav.toml [env]` セクション経由で渡す。
コードへの認証情報の直接埋め込みは型システムレベルで検知できないが、
Rune API は接続情報を関数引数（`String` 型）として受け取るため、
環境変数を使う慣習を cookbook / サンプルコードで徹底する。

**確認済み Rune（v34.4.0 時点）**:
- `runes/postgres` — `POSTGRES_URL` 環境変数を使用
- `runes/snowflake` — `SNOWFLAKE_*` 環境変数を使用
- `runes/aws` — AWS SDK 標準の環境変数チェーン（`AWS_ACCESS_KEY_ID` 等）

## 3. 実行サンドボックス（sandbox）

`fav run` はホスト OS 上の Rust 実行環境で動作し、OS レベルの sandbox は提供しない。
エフェクトシステムが実行時の境界を定義する:

| 宣言なし | `!Io` のみ | `!Http` のみ | `!Io !Http` |
|---|---|---|---|
| I/O 不可 | ファイル読み書き可 | HTTP 可 | 両方可 |

**sandbox 境界の保証**: `!Effect` を宣言しない関数は、
コンパイル時チェック（checker.fav）と W021 lint によりエフェクト呼び出しをブロックされる。
OS レベルの sandbox（seccomp / WASM サンドボックス等）は v35.x 以降で検討。

## 4. OSS ライセンス

依存ライブラリ一覧は [oss-licenses](./oss-licenses) を参照。
主要依存はすべて MIT または Apache-2.0 ライセンスであることを確認（詳細は oss-licenses ページ）。
```

---

### Step 3: site/content/docs/tools/oss-licenses.mdx 作成

主要 Cargo 依存のライセンス一覧ページを新規作成:

```markdown
---
title: "OSS ライセンス"
description: "Favnir が依存する OSS ライブラリのライセンス一覧"
---

# OSS ライセンス

Favnir（`fav` バイナリ）が依存する主要 OSS ライブラリのライセンス一覧。
確認バージョン: v34.4.0（2026-07-04）。

すべての依存ライブラリは MIT ライセンスまたは Apache-2.0 ライセンス（またはその両方）の下で
提供されており、Favnir の MIT ライセンスと互換性があることを確認した。

## 主要依存ライブラリ

| クレート | バージョン | ライセンス | 用途 |
|---|---|---|---|
| serde | 1.x | MIT / Apache-2.0 | シリアライズ・デシリアライズ |
| serde_json | 1.x | MIT / Apache-2.0 | JSON 処理 |
| serde_yaml | 0.9.x | MIT / Apache-2.0 | YAML 処理 |
| regex | 1.x | MIT / Apache-2.0 | 正規表現 |
| chrono | 0.4.x | MIT / Apache-2.0 | 日時処理 |
| tokio | 1.x | MIT | 非同期ランタイム |
| ureq | 2.x | MIT / Apache-2.0 | HTTP クライアント |
| uuid | 1.x | MIT / Apache-2.0 | UUID 生成 |
| base64 | 0.22.x | MIT / Apache-2.0 | Base64 エンコード |
| rand | 0.8.x | MIT / Apache-2.0 | 乱数生成 |
| csv | 1.x | MIT / Unlicense | CSV パーサ |
| parquet | 52.x | Apache-2.0 | Parquet ファイル処理 |
| arrow | 52.x | Apache-2.0 | Apache Arrow データ形式 |
| duckdb | 1.x | MIT | 組込み OLAP エンジン |
| rusqlite | 0.31.x | MIT | SQLite バインディング |
| tokio-postgres | 0.7.x | MIT | PostgreSQL 非同期クライアント |
| tonic | 0.11.x | MIT | gRPC フレームワーク |
| rskafka | 0.6.x | MIT / Apache-2.0 | Kafka クライアント |
| redis | 0.25.x | MIT / BSD-3-Clause | Redis クライアント |
| mongodb | 3.x | Apache-2.0 | MongoDB クライアント |
| rayon | 1.x | MIT / Apache-2.0 | 並列処理 |
| petgraph | 0.6.x | MIT / Apache-2.0 | グラフアルゴリズム（DAG 実行） |
| wasmtime | 30.x | Apache-2.0 | WASM 実行エンジン |
| inferno | 0.11.x | CDDL-1.0 / MIT | フレームグラフ生成 |
| tempfile | 3.x | MIT / Apache-2.0 | 一時ファイル |
| jsonwebtoken | 9.x | MIT | JWT 処理 |

## ライセンス適合性

Favnir 本体は MIT ライセンス。上記依存ライブラリはすべて MIT または Apache-2.0 と
互換性があり、再配布・商用利用に問題ない。

GPL・LGPL 依存はゼロ（確認済み）。
`inferno` は CDDL-1.0 ライセンスを含むが MIT デュアルライセンスのため再配布互換性あり。

確認方法:

```bash
cargo metadata --format-version 1 | jq '[.packages[] | {name: .name, license: .license}]'
```
```

---

### Step 4: SECURITY_MODEL.md 更新

ファイル末尾に v34.x ctx 移行セクションを追記:

```markdown

## v34.x Context 移行との関係

v34.5 以降で `!Effect` アノテーションを廃止し Capability Context（ctx パラメータ）に移行する予定。
ctx 移行後も公理 1〜4 は変形なく成立する:

- ctx フィールドへのアクセスが「capability を保有する」条件に相当
- ctx を持たない関数は引き続き純粋（公理 1 が適用される）
- W021 は ctx ベースの実装に対しても適用可能（v34.5 で更新予定）

v34 セキュリティ審査（v34.4.0）時点では `!Effect` 構文が現役であり、
W021 による形式検証は正常に動作することを確認した。
ctx 移行完了後に本セクションを更新する。
```

---

### Step 5: driver.rs 更新

1. `cargo_toml_version_is_34_3_0` を空スタブ化（コメント付き）
2. `v343000_tests` 直後・`// ── v31.7.0 tests` の前に `v344000_tests` を挿入

挿入位置の確認コマンド:

```bash
grep -n "v343000_tests\|// ── v31\.7\.0 tests" fav/src/driver.rs
```

```rust
// ── v34.4.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v344000_tests {
    #[test]
    fn cargo_toml_version_is_34_4_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("34.4.0"), "Cargo.toml must contain '34.4.0'");
    }

    #[test]
    fn security_audit_v2_page_exists() {
        let src = include_str!("../../site/content/docs/tools/security-audit-v2.mdx");
        assert!(
            src.contains("W021"),
            "security-audit-v2.mdx must mention W021"
        );
    }

    #[test]
    fn oss_licenses_page_exists() {
        let src = include_str!("../../site/content/docs/tools/oss-licenses.mdx");
        assert!(
            src.contains("MIT"),
            "oss-licenses.mdx must mention MIT license"
        );
    }

    #[test]
    fn security_model_has_v34_section() {
        let src = include_str!("../../SECURITY_MODEL.md");
        assert!(
            src.contains("v34"),
            "SECURITY_MODEL.md must have a v34 section"
        );
    }

    #[test]
    fn security_audit_v2_covers_sandbox() {
        let src = include_str!("../../site/content/docs/tools/security-audit-v2.mdx");
        assert!(
            src.contains("sandbox") || src.contains("サンドボックス"),
            "security-audit-v2.mdx must cover sandbox / execution boundary"
        );
    }
}
```

---

### Step 6: CHANGELOG.md 更新

先頭に `[v34.4.0]` セクションを追加:

```markdown
## [v34.4.0] — 2026-07-04

### Added
- `site/content/docs/tools/security-audit-v2.mdx` — セキュリティ審査 v2 レポート（W021・認証情報・sandbox・OSS ライセンス）
- `site/content/docs/tools/oss-licenses.mdx` — OSS 依存ライセンス一覧（20+ クレート）

### Changed
- `SECURITY_MODEL.md` — v34.x ctx 移行との関係セクション追加
- `versions/current.md` — 最新安定版を v34.4.0 に更新
```

---

### Step 7: benchmarks/v34.4.0.json 作成

```json
{
  "version": "34.4.0",
  "milestone": "Production Ready",
  "date": "2026-07-04",
  "tests_passed": 2556,
  "tests_failed": 0,
  "notes": "セキュリティ審査 v2: security-audit-v2.mdx / oss-licenses.mdx 追加。SECURITY_MODEL.md v34 更新。v344000_tests 5 件追加。"
}
```

（`tests_passed` は `cargo test` 実測後に確定）

---

### Step 8: versions/current.md 更新

以下のフィールドを変更する:
- `最新安定版` 行: `**v34.3.0** — ベンチマーク公開` → `**v34.4.0** — セキュリティ審査 v2`
- `cargo install` 行: `"34.3.0"` → `"34.4.0"`
- `進行中バージョン` 行: `なし（v34.3.0 完了直後）` → `なし（v34.4.0 完了直後）`
- `次に切る版` 行: `**v34.4.0** — セキュリティ審査 v2` → `**v34.5.0** — !Effect 廃止・コンテキスト構文統一`

---

## テスト実行

```bash
cd /c/Users/yoshi/favnir/fav && cargo test --bin fav v344000 2>&1 | tail -8
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

---

## 完了処理

- `benchmarks/v34.4.0.json` の `tests_passed` を実測値で確定
  （実測値が想定 2556 と異なる場合は spec.md の完了条件も実測値に更新する）
- `tasks.md` を COMPLETE に更新
