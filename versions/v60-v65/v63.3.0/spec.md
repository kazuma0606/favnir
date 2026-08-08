# v63.3.0 Spec — キャッシュ型シグネチャ不整合検出 E0428

Version: 63.3.0
Status: 未着手
Base tests: 3410
Target tests: 3412

---

## 概要

`error_catalog.rs` に E0428 `incremental_cache_conflict` を登録する。
`cache.rs` の `IncrementalCache` に `check_type_sig` メソッドを追加し、
キャッシュ内の型シグネチャと現在のコンパイル結果を比較する。
不整合を検出した際は E0428 を `eprintln!` で警告表示し、
自動的にキャッシュエントリを無効化して再コンパイルを促す（致命的エラーではない）。

表示形式:
```
E0428: incremental cache signature mismatch
  stage `Transform` の型シグネチャがキャッシュと一致しません。
  cached:  List<Row> -> List<Row>
  current: List<Row> -> List<EnrichedRow>
  キャッシュを無効化して再コンパイルします。
```

---

## 前提確認（T0 で実施）

- `cargo test -j 8 -- --test-threads=8` でベース 3410 tests passed, 0 failed を確認
- `fav/src/cache.rs` が存在し `IncrementalCache` / `stage_hash` が実装されていることを確認
- `driver.rs` に `v63200_tests` が存在することを確認（挿入位置確認）

---

## 実装スコープ

### 1. `error_catalog.rs` — E0428 エントリ追加

既存の E0397 エントリの後に追加する:

```rust
// ── E0428: キャッシュ型シグネチャ不整合 (v63.3.0) ──────────────────────
ErrorEntry {
    code: "E0428",
    title: "incremental_cache_conflict",
    category: "cache",
    description: "The cached type signature for a stage does not match the current compilation result.",
    example: "// stage Transform: cached Row -> Row, now Row -> EnrichedRow\n// E0428: incremental cache signature mismatch",
    fix: "The cache entry has been automatically invalidated. Re-run to recompile.",
    long_description: Some("Favnir's incremental cache stores the type signature of each stage alongside its source hash. If the source hash matches but the type signature has changed (e.g., after refactoring a return type), E0428 is emitted as a warning. The cache entry is automatically invalidated and the stage will be recompiled on the next run."),
    suggestion: Some("This is a non-fatal warning. The cache has been cleared for the affected stage."),
},
```

### 2. `cache.rs` — `check_type_sig` メソッド追加

`IncrementalCache` に以下のメソッドを追加する:

```rust
/// ソースハッシュと型シグネチャの両方を検証する。
/// - ハッシュ + シグ両方一致 → true（キャッシュヒット、再コンパイル不要）
/// - ハッシュ一致・シグ不一致 → E0428 警告を eprintln!、キャッシュを自動無効化、false を返す
/// - ハッシュ不一致またはエントリなし → false（通常のキャッシュミス）
pub fn check_type_sig(&self, stage_name: &str, source_hash: &str, current_sig: &str) -> bool {
    match self.load_entry(stage_name) {
        Some(e) if e.source_hash == source_hash && e.type_sig == current_sig => true,
        Some(e) if e.source_hash == source_hash && e.type_sig != current_sig => {
            eprintln!(
                "E0428: incremental cache signature mismatch\n  stage `{}` の型シグネチャがキャッシュと一致しません。\n  cached:  {}\n  current: {}\n  キャッシュを無効化して再コンパイルします。",
                stage_name, e.type_sig, current_sig
            );
            self.invalidate(stage_name);
            false
        }
        _ => false,
    }
}
```

### 3. `driver.rs` — `v63300_tests` 追加

`v63200_tests` の直前（ファイル先頭方向）に挿入する:

```rust
// -- v63300_tests (v63.3.0) -- E0428 キャッシュ型シグネチャ不整合検出 --
#[cfg(test)]
mod v63300_tests {
    use crate::cache::{IncrementalCache, stage_hash};
    use tempfile::TempDir;

    #[test]
    fn incremental_e0428_signature_mismatch() {
        let dir = TempDir::new().unwrap();
        let cache = IncrementalCache::new(dir.path());
        let hash = stage_hash(b"fn transform(r: Row) -> Row { r }");
        // キャッシュに保存（型シグ: "Row -> Row"）
        cache.store("Transform", &hash, "Row -> Row");
        // 同じハッシュで異なる型シグ → false（E0428 警告・自動無効化）
        let result = cache.check_type_sig("Transform", &hash, "Row -> EnrichedRow");
        assert!(!result, "signature mismatch should return false");
    }

    #[test]
    fn cache_auto_invalidated() {
        let dir = TempDir::new().unwrap();
        let cache = IncrementalCache::new(dir.path());
        let hash = stage_hash(b"fn transform(r: Row) -> Row { r }");
        cache.store("Transform", &hash, "Row -> Row");
        // シグネチャ不整合 → 自動無効化
        cache.check_type_sig("Transform", &hash, "Row -> EnrichedRow");
        // 無効化後はキャッシュミス
        assert!(
            !cache.is_hit("Transform", &hash),
            "cache should be invalidated after E0428"
        );
    }
}
```

---

## 完了条件

- `cargo build` エラーなし
- `cargo test v63300` で 2 件 PASS
  - `incremental_e0428_signature_mismatch`
  - `cache_auto_invalidated`
- `cargo test -j 8 -- --test-threads=8` で 3412 tests passed, 0 failed

---

## 非スコープ

- `check_type_sig` の実際の型チェックパス統合（型チェッカーからの呼び出し）
- E0428 を `FavError` / `DiagnosticKind` 型に統合（LSP 連携）
- `fav check` CLI への E0428 組み込み
- `type_sig` 空文字列の公式セマンティクス定義
- WASM ビルド向けの追加対応（`mod cache;` 宣言側のガードは v63.1.0 で対応済み）
- `site/` MDX ドキュメント追加（v63.x 以降）

---

## 技術ノート

### ロードマップとの実装先の差異

ロードマップでは「`IncrementalCache::load` でキャッシュ型シグネチャと比較」と記述されているが、
`load_entry` は `private` メソッドであり公開 `load` メソッドは存在しない。
本バージョンでは新規公開メソッド `check_type_sig` を追加する方式で実装する。
ロードマップの「`load` 拡張」は実装方針の示唆であり、公開 API 設計は本 spec が正式定義。

### `check_type_sig` の設計方針

3 つのケースを `match` で明示的に処理する:
1. ハッシュ + シグ両方一致 → ヒット（`true`）
2. ハッシュ一致・シグ不一致 → E0428 警告 + 無効化（`false`）
3. それ以外（キャッシュなし・ハッシュ不一致）→ 通常のキャッシュミス（`false`）

シグネチャ不整合は致命的エラーではなく警告として扱う（`eprintln!`）。
無効化後は次回実行時に再コンパイルが走る。

### `&self` で `invalidate` を呼べる理由

`check_type_sig` のシグネチャは `&self`（不変参照）だが、
`invalidate` も `pub fn invalidate(&self, ...)` で `&self` として宣言されている
（ファイル削除操作 `std::fs::remove_file` は内部状態を変更しないため `&self` で実装可能）。
実装者は `&mut self` に変更する必要はない。

### `type_sig` 空文字列の扱い

`IncrementalCache::store` は `type_sig` に空文字列 `""` を受け付ける。
`check_type_sig` を `current_sig = ""` で呼んだ場合、キャッシュの `type_sig` も `""` であれば
第 1 アームでヒット判定される（誤ったキャッシュヒット）。
呼び出し元は `type_sig` に有効な型文字列を渡すこと。
空文字列のセマンティクス定義は将来バージョンのスコープとする。

### E0428 `long_description` の方針

v60.6.0 で確立した `long_description` フィールドを含めること（ロードマップ要件）。
他のエントリと同様に `Some(...)` で記述する。

### WASM ビルドへの影響

`cache.rs` はファイル IO を使用するが、`lib.rs` / `main.rs` の `mod cache;` 宣言に
`#[cfg(not(target_arch = "wasm32"))]` ガードが v63.1.0 で追加済み。
本バージョンで `check_type_sig` を `cache.rs` に追加する際、モジュール宣言側は変更不要。

### テストでの `eprintln!` 出力

`check_type_sig` が `eprintln!` を呼ぶため、テスト実行時に stderr に E0428 警告が出力される。
これは意図的な動作で、Rust テストフレームワークは stderr 出力をテスト失敗とはみなさない。
