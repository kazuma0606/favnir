# Spec — v57.6.0 — コンプライアンスレポート（GDPR / SOC2 対応）

## 概要

GDPR・SOC2 フレームワークに対応したコンプライアンスレポートを生成する
`ComplianceReport` データ構造と `generate_report()` 純粋関数を `v57600_tests` モジュールとして実装する。
GDPR / SOC2 それぞれのレポートが生成されること（フレームワーク識別子・エントリ数・Markdown ヘッダ含む）を
Rust テストで検証する。

> **スコープ注意**: 実際の JSONL 監査ログファイルの読み込み・`fav compliance-report` CLI コマンド実装・
> Markdown ファイル出力（`-o report.md`）は v57.6.0 のスコープ外。
> 本バージョンはレポート生成ロジックのデータ構造と純粋関数の確立に集中する。

---

## ロードマップ参照

- `versions/roadmap/roadmap-v57.1-v58.0.md` — v57.6.0 セクション
- ベーステスト数: **3263**（v57.5.0 完了時点の実績値）
- 目標テスト数: **3265**（+2）、かつ `cargo test` failures=0

---

## スコープ外項目（後続バージョンへ延期）

| 項目 | 延期先 | 理由 |
|---|---|---|
| JSONL 監査ログファイルの実際の読み込み | 未定（別途スプリント設計時に確定） | ファイル I/O 実装を要する |
| `fav compliance-report` CLI コマンド実装 | 未定（別途スプリント設計時に確定） | CLI 層の改修は独立スプリントで対応 |
| Markdown ファイル出力（`-o report.md`） | 未定（別途スプリント設計時に確定） | ファイル書き込み実装を要する |
| GDPR / SOC2 の詳細チェック項目評価ロジック | 未定（別途スプリント設計時に確定） | 規制要件の詳細実装は別スプリントで対応 |
| サイトドキュメント（`enterprise/compliance.mdx`） | v57.8.0 | Enterprise Security ドキュメントまとめ対応 |

---

## 実装内容

### 1. `fav/Cargo.toml` バージョン更新

```toml
version = "57.6.0"
```

---

### 2. `fav/src/driver.rs` — `v57600_tests` 追加

`v57500_tests` の直前に挿入する。

#### 2-1: `ComplianceFramework` 列挙型

```rust
// モジュール内プライベート
#[derive(Debug, PartialEq)]
enum ComplianceFramework {
    Gdpr,
    Soc2,
}
```

#### 2-2: `ComplianceReport` 構造体

```rust
// モジュール内プライベート
#[derive(Debug)]
struct ComplianceReport {
    framework: ComplianceFramework,
    entry_count: usize,
    sections: Vec<String>,    // Markdown セクション見出しのリスト
}
```

#### 2-3: `generate_report` 関数

```rust
fn generate_report(framework: ComplianceFramework, entries: &[&str]) -> ComplianceReport
```

- `framework` に応じて適切な `sections` を生成
  - `Gdpr`: `["## Data Access Log", "## Deletion Records"]`
  - `Soc2`: `["## Access Control", "## Audit Trail"]`
- `entry_count` は `entries.len()`
- 外部 I/O なし（純粋関数）

#### 2-4: テスト関数

| テスト名 | 検証内容 |
|---|---|
| `compliance_report_gdpr_generates` | `ComplianceFramework::Gdpr` で生成されたレポートの `framework`・`entry_count`・`sections` の内容（GDPR 固有見出し）を検証 |
| `compliance_report_soc2_generates` | `ComplianceFramework::Soc2` で生成されたレポートの `framework`・`entry_count`・`sections` の内容（SOC2 固有見出し）を検証 |

---

### 3. `fav/src/driver.rs` — バージョンチェックテスト更新

```
v56300_tests::cargo_toml_version_is_56_3_0  : "57.5.0" → "57.6.0"（failure メッセージも更新）
v56900_tests::cargo_toml_version_is_56_9_0  : "57.5.0" → "57.6.0"（rolling）
v57000_tests::cargo_toml_version_is_57_0_0  : "57.5.0" → "57.6.0"（rolling）
```

> `v57100_tests` 〜 `v57500_tests` には `cargo_toml_version_is_*` がないため更新不要。

---

## テスト仕様

| テスト名 | 検証内容 |
|---|---|
| `compliance_report_gdpr_generates` | framework が `Gdpr`・entry_count が正しい・sections に `"## Data Access Log"` と `"## Deletion Records"` が含まれることを検証 |
| `compliance_report_soc2_generates` | framework が `Soc2`・entry_count が正しい・sections に `"## Access Control"` と `"## Audit Trail"` が含まれることを検証 |

---

## 完了条件

- `cargo build` コンパイルエラーなし
- `cargo test` 全通過（**3265 tests passed, 0 failed**、ベース 3263 + 2）
- `cargo clippy -- -D warnings` クリーン
- `v57600_tests` 2 件全 pass
- `CHANGELOG.md` に `[v57.6.0]` エントリが追加されている
- `versions/current.md` が v57.6.0 / 3265 tests を反映

---

## 備考

- `ComplianceFramework` / `ComplianceReport` / `generate_report` はすべて `v57600_tests` 内に完結
- `toml.rs` への変更は不要（driver.rs のみ）
- 外部 crate 追加なし
- `ComplianceFramework::PartialEq` を derive して framework の一致確認を assert で行う
- `v57600_tests` モジュールを `v57500_tests` の直前に挿入する（正しい降順: …v57600_tests → v57500_tests → …）
- `v57500_tests` 〜 `v57100_tests` には `cargo_toml_version_is_*` が存在しないため rolling 更新対象は v56300 / v56900 / v57000 の 3 件のみ
