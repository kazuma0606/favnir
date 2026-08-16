# v75.1.0 — `FreshnessPolicy` 型基盤

Date: 2026-08-14
Status: 計画中

ロードマップ: [roadmap-v75.1-v76.0.md](../../roadmap/roadmap-v75.1-v76.0.md)

---

## Background

v75.0.0（Favnir 2.0 宣言）完了後、Phase 6「Favnir 3.0 宣言」の第 1 スプリント「Temporal Data Native」を開始する。

データエンジニアにとって「いつ時点のデータか」は命題である。古いデータを最新として扱うバグは本番でしか気づかない。`FreshnessPolicy` 型は、データの「鮮度」をコード上で明示し、鮮度違反をパイプライン実行時に検出する足がかりとなる。

本バージョン（v75.1.0）は Temporal Data Native スプリントの最初のステップとして、鮮度チェックの型基盤を構築する。

---

## Goals

1. `FreshnessStrategy` enum を Rust に追加（Warn / Fail の二択）
2. `FreshnessPolicy` 構造体を追加（max_age_secs + strategy）
3. `check_freshness` 関数を追加（タイムスタンプ差分とポリシーを照合）
4. `format_freshness_warning` 関数を追加（警告メッセージを生成）
5. Rust テスト 2 件を driver.rs に追加（3692 → 3694）

---

## 型・API 仕様

### `FreshnessStrategy` enum

```rust
pub enum FreshnessStrategy {
    Warn,  // 鮮度違反を警告として出力（処理は続行）
    Fail,  // 鮮度違反をエラーとして扱い処理を停止
}
```

### `FreshnessPolicy` 構造体

```rust
pub struct FreshnessPolicy {
    pub max_age_secs: u64,           // 許容する最大データ年齢（秒）
    pub strategy: FreshnessStrategy, // 違反時の挙動
}
```

### `check_freshness` 関数

```rust
pub fn check_freshness(data_ts: i64, now: i64, policy: &FreshnessPolicy) -> bool
```

- `data_ts`: データの生成・更新タイムスタンプ（UNIX 秒）
- `now`: 現在時刻（UNIX 秒）
- 戻り値: `true` = 鮮度 OK、`false` = 鮮度違反

```
age_secs = now - data_ts
age_secs <= policy.max_age_secs であれば true
```

### `format_freshness_warning` 関数

```rust
pub fn format_freshness_warning(policy: &FreshnessPolicy, age_secs: u64) -> String
```

出力例:
```
[FreshnessPolicy] STALE: data age=3601s exceeds max_age=3600s (strategy=Fail)
```

### Favnir コード例

```favnir
fn get_price(id: String, ctx: AppCtx) -> Result<Float, String> {
    bind raw <- ctx.io.read_file_raw("prices.csv")
    bind _   <- FreshnessPolicy.check(raw, max_age: Duration.minutes(5))
    Result.ok(parse_price(raw))
}
```

---

## Success Criteria

- `FreshnessStrategy` / `FreshnessPolicy` が Rust にコンパイルされる
- `check_freshness(ts, now, policy)` が正しい bool を返す
- `format_freshness_warning` が期待する文字列フォーマットを返す
- 以下の Rust テスト 2 件が pass する:
  - `freshness_policy_enforced` — TTL 内のデータは check_freshness が true を返す
  - `freshness_stale_detected` — TTL 超過のデータは check_freshness が false を返す（`format_freshness_warning` の検証もこのテスト内で行う）
- テスト総数: 3694（3692 + 2）

---

## Error Codes

新規エラーコード追加なし。

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `FreshnessStrategy` enum / `FreshnessPolicy` 構造体 / `check_freshness` / `format_freshness_warning` 関数を追加。`v751000_tests` モジュールを追加 |
| `fav/Cargo.toml` | version `"75.0.0"` → `"75.1.0"` |
| `CHANGELOG.md` | v75.1.0 エントリを追加 |
| `versions/current.md` | 進行中バージョンを v75.1.0 に更新 |
