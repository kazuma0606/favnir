# v73.3.0 Spec — PII 検出・マスキング Rune

Date: 2026-08-13
Status: 計画中

---

## 背景

エンタープライズデータパイプラインでは、個人情報（PII）を保護することが法的・倫理的要件である。
v73.3.0 では PII フィールドのマスキング・スキャン・GDPR 削除チェックを行う
Rust 関数群と対応スタブ Rune ファイルを実装する。

---

## 目標

1. `PiiMaskStrategy` 列挙型（Hash / Redact / Truncate）を追加
2. `mask_pii_fields(fields, strategy)` — PII フィールドをマスクして返す関数
3. `scan_pii_patterns(text)` — メールアドレス・電話番号パターンを検出する関数
4. `gdpr_erase_record(fields_to_erase)` — GDPR 削除対象フィールドのクリア関数
5. `runes/privacy/privacy.fav` + `runes/privacy/rune.toml` — スタブ Rune ファイル
6. 2 件のテスト（`privacy_rune_mask_pii_fields` / `privacy_rune_gdpr_erase`）

---

## API 例

```rust
// マスキング
let fields = vec![
    ("email".to_string(), "user@example.com".to_string()),
    ("name".to_string(),  "Alice".to_string()),
];
let masked = mask_pii_fields(&fields, PiiMaskStrategy::Hash);
// masked: [("email", "***"), ("name", "***")]

// スキャン
let hits = scan_pii_patterns("contact user@example.com or call 090-1234-5678");
// hits: ["email:user@example.com", "phone:090-1234-5678"]

// GDPR 削除
let result = gdpr_erase_record(&["email", "phone"]);
// result: Ok(2)  ← 削除したフィールド数
```

---

## 実装詳細

### `PiiMaskStrategy` 列挙型

```rust
pub enum PiiMaskStrategy {
    Hash,     // フィールド値を "***" で固定上書き（将来的に sha256 ハッシュ化を予定）
    Redact,   // フィールド値を "[REDACTED]" で置換
    Truncate, // フィールド値を最初の2文字 + "..." に短縮（空文字 → "..."、1文字 → "<1文字>..."）
}
```

### `mask_pii_fields`

```rust
pub fn mask_pii_fields(
    fields: &[(String, String)],
    strategy: PiiMaskStrategy,
) -> Vec<(String, String)>
```

- `Hash` → 値を `"***"` に置換
- `Redact` → 値を `"[REDACTED]"` に置換
- `Truncate` → 値を `&value[..2.min(value.len())]` + `"..."` に置換

### `scan_pii_patterns`

```rust
pub fn scan_pii_patterns(text: &str) -> Vec<String>
```

- メールアドレスパターン（`@` を含む単語）を検出 → `"email:<value>"` 形式で追加
- 電話番号パターン（`-` を含み、かつ数字が 7 桁以上の単語）を検出 → `"phone:<value>"` 形式で追加
- シンプルな文字列マッチング（正規表現ライブラリ不使用）

### `gdpr_erase_record`

```rust
pub fn gdpr_erase_record(fields_to_erase: &[&str]) -> Result<usize, String>
```

- `fields_to_erase` が空なら `Err("no fields specified for erasure")`
- それ以外は `Ok(fields_to_erase.len())` — 削除フィールド数を返す（スタブ）

### `runes/privacy/privacy.fav`（スタブ）

```favnir
// Rune.privacy — PII 保護 Rune（将来 VM primitive に接続）
fn mask(fields: List<String>, strategy: String) -> List<String> {
    fields
}

fn scan(text: String) -> List<String> {
    []
}

fn gdpr_erase(user_id: String, tables: List<String>) -> Int {
    0
}
```

### `runes/privacy/rune.toml`

```toml
[rune]
name = "privacy"
version = "0.1.0"
description = "PII detection and masking for Favnir pipelines"
```

---

## テスト

### `v733000_tests` モジュール

```rust
#[test]
fn privacy_rune_mask_pii_fields() {
    let fields = vec![
        ("email".to_string(), "user@example.com".to_string()),
        ("name".to_string(),  "Alice".to_string()),
    ];
    // Hash マスク
    let hashed = mask_pii_fields(&fields, PiiMaskStrategy::Hash);
    assert_eq!(hashed.len(), 2, "should return same number of fields");
    assert!(hashed.iter().all(|(_, v)| v == "***"), "all values should be masked");
    // Redact マスク
    let redacted = mask_pii_fields(&fields, PiiMaskStrategy::Redact);
    assert!(redacted.iter().all(|(_, v)| v == "[REDACTED]"), "all values should be redacted");
    // scan_pii_patterns
    let hits = scan_pii_patterns("contact user@example.com or 090-1234-5678");
    assert!(!hits.is_empty(), "should detect PII patterns");
    assert!(hits.iter().any(|h| h.contains("email")), "should detect email pattern");
}

#[test]
fn privacy_rune_gdpr_erase() {
    // 正常系
    let ok = gdpr_erase_record(&["email", "phone", "ssn"]);
    assert!(ok.is_ok(), "gdpr erase should succeed: {:?}", ok);
    assert_eq!(ok.unwrap(), 3, "should return count of erased fields");
    // エラー系（空フィールド）
    let err = gdpr_erase_record(&[]);
    assert!(err.is_err(), "empty fields should return Err");
    assert!(err.unwrap_err().contains("no fields"), "error should mention no fields");
    // rune.toml の存在確認
    let rune_toml = include_str!("../../runes/privacy/rune.toml");
    assert!(rune_toml.contains("privacy"), "rune.toml should reference privacy rune");
}
```

---

## 成功基準

- `cargo test v733000` で 2 件 pass
- `cargo test` 全体で 3652 tests pass（3650 + 2）
- `fav/Cargo.toml` のバージョンが `73.3.0`
- `PiiMaskStrategy` / `mask_pii_fields` / `scan_pii_patterns` / `gdpr_erase_record` が pub で存在する
- `runes/privacy/privacy.fav` と `runes/privacy/rune.toml` が存在する

---

## スコープ外

- `Rune.privacy` の VM primitive 接続（v73.6.0 以降）
- 正規表現ライブラリ（`regex` crate）の導入（シンプル文字列マッチングで代替）
- `main.rs` への `privacy` コマンド登録（将来バージョン）
- WASM / サイト MDX 更新（v74.x 以降）

---

## 変更ファイル

- `fav/src/driver.rs` — `PiiMaskStrategy` / `mask_pii_fields` / `scan_pii_patterns` / `gdpr_erase_record` + `v733000_tests` + バージョン更新
- `fav/Cargo.toml` — version `73.2.0` → `73.3.0`
- `runes/privacy/privacy.fav` — スタブ Rune ファイル（新規作成）
- `runes/privacy/rune.toml` — Rune 設定ファイル（新規作成）
- `CHANGELOG.md` — v73.3.0 エントリ追加
- `versions/current.md` — 進行中バージョンを v73.3.0 に更新
