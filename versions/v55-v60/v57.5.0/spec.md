# Spec — v57.5.0 — 監査ログ暗号化・署名（tamper-proof audit）

## 概要

監査ログエントリに対して決定論的署名を付与する `sign_entry()` 関数と、
署名を検証する `verify_entry()` 関数を `v57500_tests` モジュールとして実装する。
改ざんされたエントリが検出されること（tamper-proof）を Rust テストで検証する。

> **スコープ注意**: 実際の HMAC-SHA256 暗号ライブラリ（外部 crate）の使用・
> `--audit-sign` / `fav audit verify` CLI コマンドの実装・
> `[secrets]` プロバイダからの鍵取得は v57.5.0 のスコープ外。
> 本バージョンは署名・検証ロジックのデータ構造と純粋関数の確立に集中する。
> 外部 crate 追加なし（WASM ビルド影響回避）。

---

## ロードマップ参照

- `versions/roadmap/roadmap-v57.1-v58.0.md` — v57.5.0 セクション
- ベーステスト数: **3261**（v57.4.0 完了時点の実績値）
- 目標テスト数: **3263**（+2）、かつ `cargo test` failures=0

---

## スコープ外項目（後続バージョンへ延期）

| 項目 | 延期先 | 理由 |
|---|---|---|
| 実際の HMAC-SHA256（外部 crate）使用 | 未定（別途スプリント設計時に確定） | `ring` / `hmac` crate 追加は WASM ビルドに影響する可能性 |
| `--audit-sign` CLI フラグ実装 | 未定（別途スプリント設計時に確定） | CLI 層の改修は独立スプリントで対応 |
| `fav audit verify` コマンド実装 | 未定（別途スプリント設計時に確定） | CLI 層の改修は独立スプリントで対応 |
| `[secrets]` プロバイダからの鍵取得 | 未定（別途スプリント設計時に確定） | v57.2.0 実装との結合は後続で対応 |
| サイトドキュメント（`enterprise/audit.mdx`） | v57.8.0 | Enterprise Security ドキュメントまとめ対応 |

---

## 実装内容

### 1. `fav/Cargo.toml` バージョン更新

```toml
version = "57.5.0"
```

---

### 2. `fav/src/driver.rs` — `v57500_tests` 追加

`v57400_tests` の直前に挿入する。

#### 2-1: `AuditEntry` 構造体定義

```rust
// モジュール内プライベート（pub 不要）
#[derive(Debug, Clone)]
struct AuditEntry {
    id: u64,
    event: String,
    payload: String,
}
```

#### 2-2: `sign_entry` 関数

外部 crate 不要の決定論的署名（標準ライブラリのみ）:

```rust
fn sign_entry(entry: &str, key: &str) -> String
```

- `entry` と `key` の両方を組み合わせた決定論的ハッシュを 16 桁 hex で返す
- 同じ `entry` + 同じ `key` → 常に同じ署名（deterministic）
- `entry` が 1 文字でも変わると異なる署名が返る（tamper-sensitive）
- `key` が異なると異なる署名が返る（key-sensitive）
- 外部 crate 不要（stdlib の `u64` 演算のみ）

#### 2-3: `verify_entry` 関数

```rust
fn verify_entry(entry: &str, signature: &str, key: &str) -> bool
```

- `sign_entry(entry, key)` を再計算し `signature` と比較
- 一致すれば `true`（tamper-free）、不一致なら `false`（tampered）

#### 2-4: テスト関数

| テスト名 | 検証内容 |
|---|---|
| `audit_sign_entry` | 署名が非空・決定論的（同じ入力→同じ署名）であること。異なるキーでは異なる署名になること |
| `audit_verify_tamper_detected` | オリジナルエントリは検証を通過し、改ざんされたエントリは検証に失敗すること |

---

### 3. `fav/src/driver.rs` — バージョンチェックテスト更新

```
v56300_tests::cargo_toml_version_is_56_3_0  : "57.4.0" → "57.5.0"（failure メッセージも更新）
v56900_tests::cargo_toml_version_is_56_9_0  : "57.4.0" → "57.5.0"（rolling）
v57000_tests::cargo_toml_version_is_57_0_0  : "57.4.0" → "57.5.0"（rolling）
```

> `v57100_tests` / `v57200_tests` / `v57300_tests` / `v57400_tests` には `cargo_toml_version_is_*` がないため更新不要。

---

## テスト仕様

| テスト名 | 検証内容 |
|---|---|
| `audit_sign_entry` | `AuditEntry` 構造体を使い entry 文字列を生成。署名の非空性・16 桁 hex・決定論性・key-sensitivity・entry-sensitivity（1 文字変更で署名が変わる）を検証 |
| `audit_verify_tamper_detected` | オリジナル → true / 改ざん → false / 誤ったキー → false の 3 ケースを検証 |

---

## 完了条件

- `cargo build` コンパイルエラーなし
- `cargo test` 全通過（**3263 tests passed, 0 failed**、ベース 3261 + 2）
- `cargo clippy -- -D warnings` クリーン
- `v57500_tests` 2 件全 pass
- `CHANGELOG.md` に `[v57.5.0]` エントリが追加されている
- `versions/current.md` が v57.5.0 / 3263 tests を反映

---

## 備考

- `AuditEntry` / `sign_entry` / `verify_entry` はすべて `v57500_tests` 内に完結
- `toml.rs` への変更は不要（driver.rs のみ）
- 外部 crate 追加なし — `Cargo.toml` の `[dependencies]` は変更しない
- `sign_entry` の実装は stdlib の `u64` 演算（byte fold + wrapping_add/mul）で十分。実際の HMAC-SHA256 は後続バージョンで外部 crate と共に実装する
- `v57500_tests` モジュールを `v57400_tests` の直前に挿入する（正しい降順: …v57500_tests → v57400_tests → …）
- `v57400_tests` / `v57300_tests` / `v57200_tests` / `v57100_tests` には `cargo_toml_version_is_*` が存在しないため rolling 更新対象は v56300 / v56900 / v57000 の 3 件のみ
