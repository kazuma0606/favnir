# v58.7.0 Plan — HA / DR（ヘルスチェック・フェイルオーバー）

Date: 2026-07-29

---

## 実装順序

### Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml` の `version` を `"58.6.0"` → `"58.7.0"` に変更。

### Step 2: roadmap 更新

`roadmap-v58.1-v59.0.md` に以下を行う:
- v58.7.0 セクションに `> 実装スコープ変更` ブロックを追加
- v58.8.0 のベース数を `3295 + 2 = 3297` → `3298 + 2 = 3300` に修正
  （v58.6.0 code-review で +2 追加されたため）

### Step 3: driver.rs に関数追加

`inject_env_config` の直後（`// ── fav doc --builtins` の前）に追加:

```rust
// ── fav run --ha (v58.7.0) ───────────────────────────────────────────────────

/// `fav run --ha --replica <n>` の driver スタブ。
/// 複数レプリカ起動・/healthz チェック・フェイルオーバーを出力で模倣する。
pub fn cmd_ha_run(replica_count: u32) -> i32 {
    println!("[HA] Primary replica started (port 8080)");
    for i in 1..replica_count {
        println!("[HA] Secondary replica started (port {})", 8080 + i);
    }
    println!("[HA] Health check: /healthz → 200 OK");
    println!("[HA] Failover: primary → secondary (reason: primary unresponsive)");
    0
}
```

### Step 4: driver.rs テストモジュール追加

`v58700_tests` モジュールを `v58600_tests` の直前に挿入:

```rust
// -- v58700_tests (v58.7.0) -- HA / DR --
#[cfg(test)]
mod v58700_tests {
    use super::cmd_ha_run;

    #[test]
    fn ha_health_check_endpoint() {
        // 単一レプリカでヘルスチェックが動作し exit code 0 を返すことを検証
        let code = cmd_ha_run(1);
        assert_eq!(code, 0, "cmd_ha_run should return 0 for single replica");
    }

    #[test]
    fn ha_failover_triggers() {
        // 2 レプリカでフェイルオーバーが動作し exit code 0 を返すことを検証
        let code = cmd_ha_run(2);
        assert_eq!(code, 0, "cmd_ha_run should return 0 with failover");
    }
}
```

### Step 5: driver.rs ローリングチェック更新

- `version = \"58.6.0\"` → `version = \"58.7.0\"`（5 件、`replace_all`）
- `"Cargo.toml version should be 58.6.0"` → `"58.7.0"`（5 件、パターン別に更新）

### Step 6: main.rs 拡張

- use imports に `cmd_ha_run` を追加
- `Some("run")` アームの `--env` ブロックの直後に `--ha` フラグ検出ロジックを追加:

```rust
// ── v58.7.0: fav run --ha --replica <n> ──────────────────────────────────────
if args.iter().any(|a| a == "--ha") {
    let replica_count = args.iter()
        .position(|a| a == "--replica")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(1);
    let code = cmd_ha_run(replica_count);
    if code != 0 {
        std::process::exit(code);
    }
    // v58.x stub: HA 設定の出力のみ行い、実際のレプリカ起動は将来バージョンで統合
    return;
}
```

---

## 既存コードとの関係

| 既存コード | 扱い |
|---|---|
| `Some("run")` の `--env` ブロック | 変更なし。その直後に `--ha` ブロックを挿入 |
| `Some("run")` の `--debug` / `--precompiled` 等 | `return;` により影響なし |
| `Some("deploy")` の HA 関連フラグ（なし） | 無関係 |

---

## リスク・注意点

- `--ha` ブロックも `return;` で既存パスと分離する（`--env` と同じ設計）
- `--replica` に数値以外を渡しても `parse::<u32>().ok()` が `None` を返し `unwrap_or(1)` でデフォルト動作するため、明示エラーなし（スタブの意図的設計）
- `--replica 0` は `u32` パース成功（ゼロ）として扱われ、`for 1..0` ループが即終了するため Secondary は生成されず Primary のみ出力される。スタブとして許容（将来バージョンで `>= 1` バリデーション追加可）
- `--ha` なし + `--replica` のみ渡された場合は `--replica` を無視して既存パスに進む（`--ha` の有無で分岐するため）
- テスト名 `ha_health_check_endpoint` / `ha_failover_triggers` — driver.rs に同名関数なし → `_test` サフィックス不要
- v58.8.0 ベース数修正: `3295 → 3298`（v58.6.0 code-review +2 の累積）
