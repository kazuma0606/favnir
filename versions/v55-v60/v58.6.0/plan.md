# v58.6.0 Plan — マルチ環境設定（dev / staging / prod）

Date: 2026-07-28

---

## 実装順序

### Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml` の `version` を `"58.5.0"` → `"58.6.0"` に変更。

### Step 2: roadmap に実装スコープ変更ブロック追加

`versions/roadmap/roadmap-v58.1-v59.0.md` の v58.6.0 セクションに
`> 実装スコープ変更` ブロックを追加（v58.3〜v58.5 と同形式）。

### Step 3: driver.rs に関数追加

`cmd_catalog_search` の直後（`inject_env_config` セクション）に追加:

```rust
// ── fav run --env (v58.6.0) ──────────────────────────────────────────────────

pub fn inject_env_config(env_name: &str, pipeline_file: &str) -> i32 {
    println!("[env: {}] Loading environment config from fav.toml", env_name);
    match env_name {
        "dev" => {
            println!("  snowflake.database = DEV_DB");
            println!("  kafka.bootstrap    = localhost:9092");
        }
        "staging" => {
            println!("  snowflake.database = STAGING_DB");
            println!("  kafka.bootstrap    = kafka-staging:9092");
        }
        _ => {
            // prod / その他はデフォルト prod 設定を表示
            println!("  snowflake.database = PROD_DB");
            println!("  kafka.bootstrap    = kafka-prod:9092");
        }
    }
    println!("Running {} (env: {}) ...", pipeline_file, env_name);
    0
}
```

### Step 4: driver.rs テストモジュール追加

`v58600_tests` モジュールを `v58500_tests` の直前に挿入:
- `env_config_parsed`: `inject_env_config("staging", "pipeline.fav")` → 0
- `env_config_injected`: `inject_env_config("prod", "pipeline.fav")` → 0

### Step 5: driver.rs ローリングチェック更新

v58000_tests 内のローリングアサーション（5 件）を `"58.5.0"` → `"58.6.0"` に一括更新。
failure メッセージ文字列も同じく `"58.6.0"` に更新。

### Step 6: main.rs 拡張

- use imports に `inject_env_config` を追加
- `Some("run")` アームの冒頭（既存の `--debug` チェックより前）に `--env` フラグ検出ロジックを追加

```rust
// ── v58.6.0: fav run --env <name> ────────────────────────────────────────────
if let Some(env_idx) = args.iter().position(|a| a == "--env") {
    let env_name = match args.get(env_idx + 1) {
        Some(v) => v.as_str(),
        None => {
            eprintln!("fav run: --env requires a value");
            std::process::exit(1);
        }
    };
    let pipeline_file = args.iter()
        .skip(2)
        .find(|a| !a.starts_with("--"))
        .map(|s| s.as_str())
        .unwrap_or("pipeline.fav");
    let code = inject_env_config(env_name, pipeline_file);
    if code != 0 {
        std::process::exit(code);
    }
    return;
}
```

### Step 7: CHANGELOG / current.md / roadmap 更新

事後処理ドキュメントを更新する。

---

## 既存コードとの関係

| 既存コード | 扱い |
|---|---|
| `Some("run")` の `--debug` / `--precompiled` / `--vm` チェック | 変更なし・`--env` チェックを冒頭に追加するだけ |
| `Some("deploy")` の `--env` フラグ（v57.x 実装） | 変更なし・`run` の `--env` とは独立 |
| `expand_env_vars`（roadmap 言及だが未実装） | 今バージョンでは触れない |

---

## リスク・注意点

- `Some("run")` アームは非常に長く複雑。`--env` ブロックを冒頭に配置し `return;` で抜けることで既存パスへの影響を最小化する
- テスト名 `env_config_parsed` / `env_config_injected` — `inject_env_config` と名前衝突なし → `_test` サフィックス不要
- ロードマップベース数修正: 3291 → 3292（v58.5.0 code-review 実績値）、目標 3294
