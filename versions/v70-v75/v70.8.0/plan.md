# v70.8.0 Plan — `fav doctor` 強化

Date: 2026-08-09
Status: 計画中

---

## 実装ステップ（依存順）

### Step 1: driver.rs に `doctor_check_paper_rune` を追加

`cmd_doctor_run` の直後（line ~49017 付近）に追加:

```rust
/// Paper Rune 検出: rune.toml が存在するが実装 .fav ファイルが空のディレクトリを検出する。
pub fn doctor_check_paper_rune(rune_dir: &str) -> DoctorCheck {
    let dir = std::path::Path::new(rune_dir);
    let rune_toml = dir.join("rune.toml");
    if !rune_toml.exists() {
        return DoctorCheck {
            status: DoctorStatus::Ok,
            label: format!("rune: {rune_dir}"),
            detail: String::new(),
        };
    }
    let name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("rune");
    let fav_file = dir.join(format!("{name}.fav"));
    let is_empty = if fav_file.exists() {
        std::fs::read_to_string(&fav_file)
            .map(|s| s.trim().is_empty())
            .unwrap_or(true)
    } else {
        true
    };
    if is_empty {
        DoctorCheck {
            status: DoctorStatus::Fail,
            label: format!("rune: {rune_dir}"),
            detail: "Paper Rune detected (rune.toml exists but implementation is empty)"
                .to_string(),
        }
    } else {
        DoctorCheck {
            status: DoctorStatus::Ok,
            label: format!("rune: {rune_dir}"),
            detail: String::new(),
        }
    }
}
```

### Step 2: driver.rs に `doctor_check_changelog_entry` を追加

```rust
/// CHANGELOG 整合性チェック: 指定バージョンのエントリが changelog_content に含まれるか確認。
pub fn doctor_check_changelog_entry(changelog_content: &str, version: &str) -> DoctorCheck {
    let marker = format!("[{version}]");
    if changelog_content.contains(&marker) {
        DoctorCheck {
            status: DoctorStatus::Ok,
            label: "CHANGELOG.md".to_string(),
            detail: format!("{version} entry found"),
        }
    } else {
        DoctorCheck {
            status: DoctorStatus::Fail,
            label: "CHANGELOG.md".to_string(),
            detail: format!("{version} エントリが存在しない"),
        }
    }
}
```

確認: `cargo test` で既存テスト（3576 件）が全 pass することを確認。

---

### Step 3: `v708000_tests` モジュールを driver.rs 末尾に追加

```rust
#[cfg(test)]
mod v708000_tests {
    #[test]
    fn doctor_detects_paper_rune() {
        let tmp = std::env::temp_dir().join("fav_test_paper_rune_v708");
        std::fs::create_dir_all(&tmp).ok();
        std::fs::write(tmp.join("rune.toml"), "[rune]\nname = \"test\"\n").ok();
        std::fs::write(tmp.join("test.fav"), "").ok();

        let check = super::doctor_check_paper_rune(tmp.to_str().unwrap());
        assert!(
            matches!(check.status, super::DoctorStatus::Fail),
            "expected Fail for paper rune, got {:?}: {}",
            check.status,
            check.detail
        );
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn doctor_detects_missing_changelog_entry() {
        let changelog = "# Changelog\n\n## [v70.6.0] — 2026-08-09\n";
        let check = super::doctor_check_changelog_entry(changelog, "v70.8.0");
        assert!(
            matches!(check.status, super::DoctorStatus::Fail),
            "expected Fail for missing entry, got {:?}: {}",
            check.status,
            check.detail
        );
        // 存在するバージョンは Ok
        let check_ok = super::doctor_check_changelog_entry(changelog, "v70.6.0");
        assert!(
            matches!(check_ok.status, super::DoctorStatus::Ok),
            "expected Ok for existing entry, got {:?}",
            check_ok.status
        );
    }
}
```

確認: `cargo test v708000` で 2 件 pass することを確認。

---

### Step 4: Cargo.toml バージョン更新

- `fav/Cargo.toml` の `version = "70.7.0"` → `"70.8.0"`
- driver.rs 内の `"70.7.0"` を `sed` で `"70.8.0"` に一括更新

---

### Step 5: CHANGELOG.md 更新

v70.8.0 エントリを v70.7.0 の直前に追加。

---

### Step 6: 最終確認

- `cargo test v708000` で 2 件 pass
- `cargo test` 全体で 3578 tests pass（0 failures）
- `versions/current.md` を v70.8.0 進行中に更新
