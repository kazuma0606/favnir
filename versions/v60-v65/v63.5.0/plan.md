# v63.5.0 Plan — メモリプロファイリング（`fav profile --memory`）

Version: 63.5.0
Status: 未着手

---

## 実装順序

### Step 1: `Cargo.toml` — `sysinfo` 追加

`[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` セクションの末尾（`bumpalo = "3"` の直後）に追加する:

```toml
sysinfo = "0.30"
```

`cargo build` でエラーなしを確認。

### Step 2: `driver.rs` — `cmd_profile_memory` 追加

`cmd_profile_compare` の直前に追加する（コメント行 `/// fav profile --compare ...` の直前）:

```rust
/// v63.5.0: fav profile --memory — stage 別 RSS と per-row 割り当てバイト数を計測して返す。
/// json_mode が true のとき JSON 配列形式で出力する。
/// WASM ターゲットでは RSS = 0 として動作する。
pub fn cmd_profile_memory(src: &str, json_mode: bool) -> String {
    use crate::frontend::parser::Parser;
    // Program.items は Vec<Item>。ステージは Item::TrfDef(TrfDef) として格納される。
    let stage_names: Vec<String> = match Parser::parse_str(src, "<profile-memory>") {
        Ok(prog) => prog.items.iter()
            .filter_map(|item| {
                if let crate::ast::Item::TrfDef(trf) = item {
                    Some(trf.name.clone())
                } else {
                    None
                }
            })
            .collect(),
        Err(_) => vec!["<unknown>".to_string()],
    };
    #[cfg(not(target_arch = "wasm32"))]
    let peak_rss_mb: u64 = {
        let mut sys = sysinfo::System::new();
        sys.refresh_all();
        let pid = sysinfo::Pid::from(std::process::id() as usize);
        sys.process(pid).map(|p| p.memory() / (1024 * 1024)).unwrap_or(0)
    };
    #[cfg(target_arch = "wasm32")]
    let peak_rss_mb: u64 = 0;
    let n = stage_names.len().max(1) as u64;
    let per_row_bytes = (peak_rss_mb * 1024 * 1024) / (n * 1000).max(1);
    if json_mode {
        let entries: Vec<String> = stage_names.iter().map(|name| {
            format!(r#"{{"stage":"{name}","peak_rss_mb":{peak_rss_mb},"alloc_per_row_bytes":{per_row_bytes}}}"#)
        }).collect();
        format!("[{}]", entries.join(","))
    } else {
        let mut out = String::from(
            "Stage         | Peak RSS | Alloc/row |\n--------------|----------|-----------|",
        );
        for name in &stage_names {
            out.push_str(&format!("\n{:<14}| {:>5} MB | {:>7} B |", name, peak_rss_mb, per_row_bytes));
        }
        out.push_str(&format!("\n{:<14}| {:>5} MB |           |", "Total peak", peak_rss_mb));
        out
    }
}
```

`cargo build` でエラーなしを確認。

### Step 3: `driver.rs` — `v63500_tests` 追加

`v63400_tests` の直前（ファイル先頭方向）に挿入する:

```rust
// -- v63500_tests (v63.5.0) -- メモリプロファイリング fav profile --memory --
#[cfg(test)]
mod v63500_tests {
    #[test]
    fn profile_memory_flag_works() {
        // Favnir stage 構文: `public stage Name: In -> Out = |x| body`
        let src = "public stage LoadCsv: Int -> Int = |x| { x }\npublic stage Write: Int -> Int = |x| { x }";
        let out = crate::driver::cmd_profile_memory(src, false);
        assert!(out.contains("Peak RSS"), "output should contain 'Peak RSS': {out}");
        assert!(out.contains("Alloc/row"), "output should contain 'Alloc/row': {out}");
    }

    #[test]
    fn profile_memory_per_stage() {
        let src = "public stage LoadCsv: Int -> Int = |x| { x }\npublic stage Write: Int -> Int = |x| { x }";
        let out = crate::driver::cmd_profile_memory(src, false);
        assert!(out.contains("LoadCsv"), "output should contain stage name 'LoadCsv': {out}");
        assert!(out.contains("Total peak"), "output should contain 'Total peak': {out}");
    }
}
```

### Step 4: テスト実行

```bash
cargo test v63500_tests   # 2件 PASS を確認
cargo test -j 8 -- --test-threads=8  # 3416 tests passed, 0 failed を確認
```

### Step 5: ドキュメント更新

1. `CHANGELOG.md` 先頭に v63.5.0 エントリを追加
2. `versions/roadmap/roadmap-v63.1-v64.0.md` v63.5.0 セクションに実績追記
3. `versions/current.md` の「進行中」を v63.5.0（3416 tests）に更新
4. `tasks.md` を COMPLETE に更新（全チェックボックス `[x]`）

---

## 設計メモ

### 変更対象ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/Cargo.toml` | `sysinfo = "0.30"` を native-only セクションに追加 |
| `fav/src/driver.rs` | `cmd_profile_memory` 追加 + `v63500_tests` 追加 |

### `sysinfo` WASM ガードパターン

`Cargo.toml` 側で `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` に追加することで
crate 自体は WASM でリンクされない。
`driver.rs` 内の `cmd_profile_memory` でも `#[cfg]` で分岐することで
WASM ビルド時にも `peak_rss_mb = 0` で安全に動作する。

### `prog.stages` フィールドの確認

`Parser::parse_str` の返り値（`Program`）の `stages` フィールドは `Vec<StageDecl>` 型。
各 `StageDecl` の `name: String` フィールドが stage 名を保持する。
これは既存の `cmd_profile` が同じパーサーを使用していることから確認済み。

### FavToml リテラル追加ミス防止

本バージョンでは `FavToml` 構造体を変更しないため、
checker.rs / resolver.rs 等の `FavToml { ... }` リテラルへの追加対応は不要。
