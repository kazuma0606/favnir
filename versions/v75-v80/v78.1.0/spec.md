# v78.1.0 仕様書 — `!Cached` エフェクト基盤

Date: 2026-08-16
Status: 計画中

---

## Background

関数の結果をキャッシュすることを宣言する `!Cached` エフェクトの型基盤を追加する。繰り返し呼ばれる参照データ取得（為替レート・マスタテーブル等）に対して TTL ベースのキャッシュ有効性検証を提供する。

```favnir
fn get_exchange_rate(currency: String) -> Result<Float, String> !Cached {
    ctx.io.fetch(f"https://api.rates.io/{currency}")
    // → TTL 内は同じ currency への呼び出しをキャッシュから返す
}
```

v78.1.0 では Rust 側の型基盤（`CacheStrategy` / `CacheConfig` / `CacheEntry` / `check_cache_valid`）を追加する。実際の `!Cached` エフェクトの VM 統合は将来バージョンで行う。

---

## Goals

1. `CacheStrategy` enum を追加する（Lru, Fifo, Lfu）
2. `CacheConfig` 構造体を追加する（ttl_secs: u64, strategy: CacheStrategy, max_entries: usize）
3. `CacheEntry` 構造体を追加する（key: String, inserted_at: i64, hits: u64）
4. `check_cache_valid(entry: &CacheEntry, now: i64, config: &CacheConfig) -> bool` を追加する
5. Rust テスト 2 件を追加し 3762 tests に到達する（現在 3760）

> **注**: ロードマップの完了条件は「3758 + 2 = 3760」だが、v77.8.0 の code-reviewer 対応で +2 追加されたため現在 3760。v78.1.0 では +2 追加して 3762 を目標とする。

---

## 型・関数仕様

### `CacheStrategy` enum

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheStrategy {
    Lru,   // Least Recently Used
    Fifo,  // First In First Out
    Lfu,   // Least Frequently Used
}
```

### `CacheConfig` 構造体

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheConfig {
    pub ttl_secs:    u64,
    pub strategy:    CacheStrategy,
    pub max_entries: usize,
}
```

| フィールド | 型 | 説明 |
|---|---|---|
| `ttl_secs` | u64 | エントリの有効期間（秒）|
| `strategy` | CacheStrategy | キャッシュ戦略 |
| `max_entries` | usize | 最大エントリ数（`check_cache_valid` ではメタデータとして保持のみ） |

### `CacheEntry` 構造体

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheEntry {
    pub key:         String,
    pub inserted_at: i64,  // Unix timestamp（秒）
    pub hits:        u64,  // キャッシュヒット数（LFU 戦略等で利用）
}
```

### `check_cache_valid`

```rust
pub fn check_cache_valid(entry: &CacheEntry, now: i64, config: &CacheConfig) -> bool
```

**動作:**
- `(now - entry.inserted_at) <= config.ttl_secs as i64` の場合 → `true`（TTL 内）
- それ以外（TTL 超過 or `now < entry.inserted_at`）→ `false`

> **設計注記**: `max_entries` / `strategy` は v78.1.0 では `check_cache_valid` に使用しない（メタデータとして保持）。キャッシュエビクション等への利用は v78.2.0 以降。

---

## テスト仕様

### `cache_entry_valid_within_ttl`

```rust
let config = CacheConfig { ttl_secs: 300, strategy: CacheStrategy::Lru, max_entries: 100 };
let entry  = CacheEntry  { key: "USD".to_string(), inserted_at: 1000, hits: 3 };
// now=1200: elapsed=200 <= ttl=300 → valid
assert!(check_cache_valid(&entry, 1200, &config));
```

### `cache_entry_expired`

```rust
let config = CacheConfig { ttl_secs: 300, strategy: CacheStrategy::Lru, max_entries: 100 };
let entry  = CacheEntry  { key: "USD".to_string(), inserted_at: 1000, hits: 3 };
// now=1400: elapsed=400 > ttl=300 → expired
assert!(!check_cache_valid(&entry, 1400, &config));
```

---

## Success Criteria

- `CacheStrategy` enum が定義されている（Debug / Clone / PartialEq / Eq 付き、3 バリアント）
- `CacheConfig` 構造体が定義されている（Debug / Clone / PartialEq / Eq 付き）
- `CacheEntry` 構造体が定義されている（Debug / Clone / PartialEq / Eq 付き）
- `check_cache_valid` が TTL を正しく判定する（`elapsed <= ttl` → true、超過 → false）
- `cache_entry_valid_within_ttl` が pass
- `cache_entry_expired` が pass
- `cargo test` が 3762 tests all pass
- `driver.rs` 内の `cargo_toml_version_is_X` 系テストの `78.0.0` バージョン文字列アサーションがすべて `78.1.0` に更新されている（セクションコメント `// --- v78.0.0: Verifiable Pipelines 宣言 ---` は変更しない）

---

## 変更ファイル

実装順序は plan.md 参照。

- `fav/src/driver.rs` — `CacheStrategy`, `CacheConfig`, `CacheEntry`, `check_cache_valid`, `v781000_tests` を追加
- `CHANGELOG.md` — v78.1.0 エントリを追加
- `versions/current.md` — 進行中バージョンを更新
- `fav/Cargo.toml` — バージョンを `78.0.0` → `78.1.0` に更新
- `fav/Cargo.lock` — 自動更新（手動編集不要）

---

## 対象外

- `!Cached` エフェクトの VM 統合: 将来バージョン
- キャッシュエビクション（`max_entries` / `strategy` の実際の利用）: v78.2.0 以降
- `fav.toml` の `[effects.cached]` セクション解析（`ttl_secs` / `strategy` / `max_entries` の TOML ロード）: 将来バージョンで行う。`CacheConfig` のフィールド名・型は `[effects.cached]` と 1:1 対応しており、将来の統合を想定した設計。
- `now < entry.inserted_at`（時計が過去にある場合）のケース: 実装上は `(now - inserted_at)` が負の i64 になるため `<= ttl` は `false` になり正しく動作するが、v78.1.0 では専用テスト 2 件の枠内に収めるため個別テストは追加しない。
- `parser.rs` / `ast.rs` / `checker.rs` への変更は一切行わない
