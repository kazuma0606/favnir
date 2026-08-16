# v73.8.0 実装計画 — GitHub Actions 公式 Action

Date: 2026-08-13

---

## 実装ステップ

### Step 1: `.github/actions/setup-fav/` ディレクトリと `action.yml` を作成

- `.github/actions/setup-fav/` ディレクトリを作成
- `action.yml` を composite action 形式で作成
  - `name: Setup Favnir`
  - `inputs.version`（required, default: latest）
  - `runs.using: composite`
  - OS/ARCH 判別 + URL 生成 + バイナリダウンロード + `$GITHUB_PATH` への追加

### Step 2: `.github/actions/setup-fav/README.md` を作成

- 使用例（basic / バージョン指定）
- CI バッジサンプル
- マトリックスビルドサンプル（ubuntu / macos / windows）
- `fav check` / `fav test` / `fav quality` / `fav audit` の利用例

### Step 3: `driver.rs` に `GithubActionConfig` 構造体 + `format_github_action_url` 関数を追加

```rust
// --- v73.8.0: GitHub Actions 公式 Action ---

#[derive(Debug, Clone)]
pub struct GithubActionConfig {
    pub version: String,
    pub os: String,
    pub arch: String,
}

pub fn format_github_action_url(config: &GithubActionConfig) -> String {
    format!(
        "https://github.com/favnir/favnir/releases/download/v{}/fav-{}-{}",
        config.version, config.os, config.arch
    )
}
```

### Step 4: `v738000_tests` モジュールを追加

```rust
#[cfg(test)]
mod v738000_tests {
    use super::{GithubActionConfig, format_github_action_url};

    #[test]
    fn github_action_setup_fav_action_yml_valid() {
        // action.yml のファイル内容を検証
        let src = include_str!("../../.github/actions/setup-fav/action.yml");
        assert!(src.contains("name: Setup Favnir") || src.contains("name: 'Setup Favnir'") || src.contains("Setup Favnir"));
        assert!(src.contains("version"));
        assert!(src.contains("composite"));
    }

    #[test]
    fn github_action_fav_binary_url_format() {
        let cfg = GithubActionConfig {
            version: "75.0.0".to_string(),
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
        };
        let url = format_github_action_url(&cfg);
        assert!(url.contains("v75.0.0"));
        assert!(url.contains("linux"));
        assert!(url.contains("x86_64"));
        assert!(url.starts_with("https://github.com/favnir/favnir/releases/download/"));
    }
}
```

### Step 5: バージョン更新

- `fav/Cargo.toml`: `version = "73.7.0"` → `version = "73.8.0"`
- `driver.rs` 内の `"73.7.0"` を `"73.8.0"` に replace_all

### Step 6: テスト確認

- `cargo test` で 3663 tests pass を確認

### Step 7: `CHANGELOG.md` 更新

- v73.8.0 エントリを先頭に追加

### Step 8: `versions/current.md` 更新

- 最終更新を `2026-08-13 (v73.8.0)` に変更
- 進行中を `v73.8.0` に変更
- 次を `v73.9.0` に変更
