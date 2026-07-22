# Plan: v52.5.0 — SLA 監視 Rune

Status: PLANNED
Date: 2026-07-21

---

## 実装順序

### Step 1 — `runes/sla/sla.fav` 新規作成

- ファイル: `/c/Users/yoshi/favnir/runes/sla/sla.fav`（`runes/sla/` ディレクトリごと新規）
- 参照: `runes/prometheus/prometheus.fav`（スタブパターンの手本）

**実装内容**:

```favnir
// runes/sla/sla.fav — SLA 監視 Rune (v52.5.0)
//
// 使い方:
//   import sla
//
// SLA 違反時は Err を返す（呼び出し側が bind で fail-fast を実現）。
// アラート発火は !Observe エフェクト経由（v52.5.0 はスタブ実装）。

// データの鮮度を確認する。
// timestamp（Unix 秒）が現在時刻 - max_age_seconds より古い場合 Err を返す。
public fn check_freshness(timestamp: Int, max_age_seconds: Int) -> Result<Unit, String> {
    Sla.check_freshness_raw(timestamp, max_age_seconds)
}

// ステージの実行レイテンシが threshold_ms ミリ秒以内か確認する。
// 超過した場合は Err を返す。
public fn check_latency(stage: String, threshold_ms: Int) -> Result<Unit, String> {
    Sla.check_latency_raw(stage, threshold_ms)
}

// SLA 違反アラートを送信する。
// 外部アラート基盤（Prometheus / Datadog 等）へ転送するスタブ（v52.5.0）。
public fn alert(message: String) -> Result<Unit, String> {
    Sla.alert_raw(message)
}
```

**`include_str!` パス確認**:
- テストは `driver.rs`（`fav/src/driver.rs`）に記述
- `include_str!("../../runes/sla/sla.fav")` → `fav/src/` から 2 階層上 = `favnir/` → `favnir/runes/sla/sla.fav` ✓

### Step 2 — `driver.rs` にテスト追加 + バージョン更新

- `v52500_tests` モジュールを `v52400_tests` の直前に追加（2 件）
- `fav/Cargo.toml` version → `"52.5.0"`
- `cargo test` → 3146 passed, 0 failed を確認
- `cargo clippy -- -D warnings` クリーンを確認

**テスト挿入位置の確認**:
```bash
rg -n "v52400_tests" fav/src/driver.rs
```

**`v52500_tests` モジュール**:

```rust
// -- v52500_tests (v52.5.0) -- SLA 監視 Rune --
#[cfg(test)]
mod v52500_tests {
    #[test]
    fn sla_rune_latency_check() {
        let src = include_str!("../../runes/sla/sla.fav");
        assert!(src.contains("fn check_latency("), "sla rune must define fn check_latency(");
        assert!(src.contains("threshold_ms"), "check_latency must have threshold_ms parameter");
    }

    #[test]
    fn sla_rune_freshness_check() {
        let src = include_str!("../../runes/sla/sla.fav");
        assert!(src.contains("fn check_freshness("), "sla rune must define fn check_freshness(");
        assert!(src.contains("max_age_seconds"), "check_freshness must have max_age_seconds parameter");
    }
}
```

**`v52400_tests` に version テストなし → 削除対象なし**（確認済み: `v52400_tests` は `lineage_html_output` / `lineage_html_has_stage_detail` / `lineage_html_renders_stage_node` の 3 件のみ）。

### Step 3 — 後処理

- `CHANGELOG.md` に v52.5.0 エントリ追加
- `versions/current.md` を v52.5.0（3146 tests）に更新
- `versions/roadmap/roadmap-v52.1-v53.0.md` の v52.5.0 実績欄を更新
- `tasks.md` を COMPLETE に更新（T0〜T3 全 `[x]`）

---

## 注意事項

- `runes/sla/` ディレクトリは新規作成（既存の `runes/slack/` とは別）。
- Favnir の rune ファイルは `.fav` 形式。コメントは `//` 形式。
- `public fn` キーワードを必ず付けること（他の rune と同様）。
- `Sla.check_freshness_raw` / `Sla.check_latency_raw` / `Sla.alert_raw` は VM primitive スタブ
  （実際の VM ハンドラ実装は将来バージョン）。
- `!Observe` エフェクトは現時点では Effect enum に存在しない。
  rune ファイルのコメントでのみ言及し、実際の型注釈には付けない。
- `include_str!` はコンパイル時にパスを解決するため、ファイルが存在しないとビルドエラーになる。
  Step 1 のファイル作成後に Step 2 を実施すること（逆順不可）。
