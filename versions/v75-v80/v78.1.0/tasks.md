# v78.1.0 タスクリスト — `!Cached` エフェクト基盤

Date: 2026-08-16
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `78.0.0` であることを確認
- [x] `cargo test` が全 pass（3760 tests）であることを確認
- [x] `fav/tmp/hello.fav` が存在することを確認

---

## T1: driver.rs — 型・関数追加

- [x] `fav/src/driver.rs` の末尾に `// --- v78.1.0: !Cached エフェクト基盤 ---` コメントを追加する
- [x] `CacheStrategy` enum を追加する（`#[derive(Debug, Clone, PartialEq, Eq)]`、Lru / Fifo / Lfu）
- [x] `CacheConfig` 構造体を追加する（ttl_secs: u64, strategy: CacheStrategy, max_entries: usize、`#[derive(Debug, Clone, PartialEq, Eq)]`）
- [x] `CacheEntry` 構造体を追加する（key: String, inserted_at: i64, hits: u64、`#[derive(Debug, Clone, PartialEq, Eq)]`）
- [x] `check_cache_valid(entry: &CacheEntry, now: i64, config: &CacheConfig) -> bool` を追加する
  - `(now - entry.inserted_at) <= config.ttl_secs as i64` → `true`
  - それ以外 → `false`
- [x] `cargo test` で既存 3760 tests が pass することを確認する

---

## T2: CHANGELOG.md 更新（テスト追加より先）

- [x] `CHANGELOG.md` の先頭に v78.1.0 エントリを追加する（形式: `## [v78.1.0] — 2026-08-16 — !Cached エフェクト基盤`）
- [x] Added セクション（型 3 件・関数 1 件）と Tests セクション（2 件）を含める

---

## T3: driver.rs — テストモジュール追加

- [x] `fav/src/driver.rs` に `v781000_tests` モジュールを追加する（`use super::*` 必須）
- [x] `cache_entry_valid_within_ttl` テストを実装する
  - `CacheConfig { ttl_secs: 300, strategy: CacheStrategy::Lru, max_entries: 100 }`
  - `CacheEntry { key: "USD", inserted_at: 1000, hits: 3 }`
  - `check_cache_valid(&entry, 1200, &config)` → `true`（elapsed=200 <= ttl=300）
- [x] `cache_entry_expired` テストを実装する
  - 同じ config / entry
  - `check_cache_valid(&entry, 1400, &config)` → `false`（elapsed=400 > ttl=300）
- [x] `cargo test v781000` で 2 件が pass することを確認する

---

## T4: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml` の `version` を `"78.0.0"` → `"78.1.0"` に変更する
- [x] `driver.rs` 内の `78.0.0` バージョン文字列アサーションを `78.1.0` に一括更新（`replace_all: true`）
- [x] **replace_all 後に** `grep "v78.0.0" fav/src/driver.rs` を実行し、以下を確認する:
  - `// --- v78.0.0: Verifiable Pipelines 宣言 ---` が残っていること
  - v76/v77 の `// NOTE:` コメント内の `78.0.0` 記述が書き換わっていないこと（書き換わっていたら手動で `78.0.0` に戻す）

---

## T5: versions/current.md 更新

- [x] `## 進行中バージョン` 欄を `**v78.1.0**（!Cached エフェクト基盤）` に更新する
- [x] `## 次に切る版` 欄を `**v78.2.0**（キャッシュ戦略型）` に更新する

---

## T6: 最終確認

- [x] `cargo test` が全 pass であることを確認する（3762 tests）
- [x] `cargo test v781000` で 2 件が pass することを確認する
- [x] `fav/Cargo.toml` のバージョンが `78.1.0` であることを確認する
- [x] `CHANGELOG.md` の先頭が `[v78.1.0]` であることを確認する
- [x] `versions/current.md` の「進行中バージョン」が v78.1.0 であることを確認する

---

## 完了チェックリスト

- [x] 全タスク（T0〜T6）が完了している
- [x] `cache_entry_valid_within_ttl` が pass
- [x] `cache_entry_expired` が pass
- [x] テスト総数: 3763（+3: `cache_entry_valid_within_ttl` / `cache_entry_expired` / `cache_entry_at_ttl_boundary`）
- [x] site/ MDX 追加: 本バージョンでは対象外（型基盤のみ）
- [x] `changelog_has_v78_1_0` テストの追加: 対象外（x.0.0 宣言バージョンのみに追加する慣例）

---

## code-reviewer 指摘対応（適用済み）

- [x] [HIGH] `elapsed <= ttl` → `elapsed < ttl`（境界値で即期限切れ）
- [x] [HIGH] 負の elapsed に対する doc comment 誤り修正 + `if elapsed < 0 { return false; }` ガード追加
- [x] [MED] `u64 as i64` キャスト → `i64::try_from(...).unwrap_or(i64::MAX)` に修正
- [x] [MED] 境界値テスト `cache_entry_at_ttl_boundary`（elapsed==ttl → false）追加
- [x] [LOW] `CacheStrategy` / `CacheConfig` / `CacheEntry` に `Hash` derive 追加
