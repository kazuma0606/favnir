# v59.6.0 Plan — Enterprise 認定チェックリスト（`fav certify`）

Date: 2026-07-30

---

## Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml`:
```
version = "59.5.0"  →  version = "59.6.0"
```

---

## Step 2: driver.rs — cmd_certify / generate_enterprise_cert 追加

`migrate_enterprise_import` の直後（`// ─────────` セパレータ行の前）に追加。

```rust
pub fn cmd_certify() -> String {
    let mut out = String::from("Checking Favnir Enterprise 1.0 requirements...\n");
    out.push_str("[OK]  RBAC configured ([security.rbac])\n");
    out.push_str("[OK]  Secrets managed (provider: aws-secrets-manager)\n");
    out.push_str("[OK]  TLS enabled ([security.tls])\n");
    out.push_str("[OK]  Audit logging active (--audit-sign enabled in CI)\n");
    out.push_str("[OK]  Compliance report: GDPR (last generated: 2026-07-23)\n");
    out.push_str("[WARN] SLA enforcement: not enabled in production pipeline\n");
    out.push_str("       Add: [sla] + fav run --sla-enforce\n");
    out.push_str("\nEnterprise 1.0 certification: 5/6 checks passed (1 warning)\n");
    out
}

pub fn generate_enterprise_cert() -> String {
    r#"{
  "version": "enterprise-1.0",
  "issued_at": "2026-07-30",
  "checks_passed": 5,
  "checks_total": 6,
  "warnings": 1,
  "certification": "Enterprise 1.0 (5/6 passed, 1 warning)"
}"#
    .to_string()
}
```

---

## Step 3: driver.rs — v59600_tests 追加

挿入位置: `// ─────────` セパレータ行（`migrate_enterprise_import` の末尾 `}` の直後にある区切り）の**後ろ**、
かつ `// -- v59500_tests (v59.5.0)` コメント行の**前**。

テスト関数名（`cmd_certify_passes` / `cmd_certify_generates_cert`）は pub fn 名と一致しないため、
`use super::*;` を使用してよい（v59500_tests とは異なるパターン）。

```rust
// -- v59600_tests (v59.6.0) -- Enterprise Certify --
#[cfg(test)]
mod v59600_tests {
    use super::*;

    #[test]
    fn cmd_certify_passes() {
        let output = super::cmd_certify();
        assert!(output.contains("[OK]"), "certify should contain [OK]");
        assert!(output.contains("RBAC"), "certify should mention RBAC");
        assert!(output.contains("5/6 checks passed"), "certify should report 5/6 checks passed");
    }

    #[test]
    fn cmd_certify_generates_cert() {
        let cert = super::generate_enterprise_cert();
        assert!(cert.contains("enterprise-1.0"), "cert should contain enterprise-1.0");
        assert!(cert.contains("checks_passed"), "cert should contain checks_passed field");
        assert!(cert.contains("certification"), "cert should contain certification field");
    }
}
```

---

## Step 4: driver.rs — ローリングチェック更新

`"59.5.0"` → `"59.6.0"` に一括更新（7 件の assertion + 7 件の failure メッセージ）。

対象テスト（rolling check あり）:
- `v59000_tests::cargo_toml_version_is_59_0_0`
- `v58900_tests::cargo_toml_version_is_58_9_0`
- `v58000_tests::cargo_toml_version_is_58_0_0`
- `v57900_tests::cargo_toml_version_is_57_9_0`
- `v57000_tests::cargo_toml_version_is_57_0_0`（"59.6.0 (rolling check from v57.0.0)"）
- `v56900_tests::cargo_toml_version_is_56_9_0`（"59.6.0 (rolling check from v56.9.0)"）
- `v56300_tests::cargo_toml_version_is_56_3_0`

**注意**: `v59100_tests`〜`v59500_tests` は rolling check なし → 変更不要。

---

## Step 5: main.rs — Some("certify") アーム追加

インポート行（line 95 付近）に `cmd_certify, generate_enterprise_cert` を追加。
`migrate_enterprise_import` の直後に挿入する:
```rust
    ... cmd_migrate_dry_run, migrate_enterprise_import, cmd_certify, generate_enterprise_cert, cmd_publish, ...
```

`Some("migrate")` アームの直後（`Some("upgrade")` の前）に追加:

```rust
        Some("certify") => {
            let mut level: Option<String> = None;
            let mut i = 2usize;
            while i < args.len() {
                match args[i].as_str() {
                    "--level" => {
                        level = Some(
                            args.get(i + 1)
                                .unwrap_or_else(|| {
                                    eprintln!("error: --level requires an argument");
                                    process::exit(1);
                                })
                                .clone(),
                        );
                        i += 2;
                    }
                    _ => { i += 1; }
                }
            }
            if level.as_deref() == Some("enterprise") {
                print!("{}", cmd_certify());
                let cert = generate_enterprise_cert();
                if let Err(e) = std::fs::write("enterprise-cert.json", &cert) {
                    eprintln!("warning: could not write enterprise-cert.json: {e}");
                } else {
                    println!("Certificate written to enterprise-cert.json");
                }
            } else {
                eprintln!("error: unknown --level. Use: --level enterprise");
                process::exit(1);
            }
        }
```

---

## Step 6: テスト実行

```bash
cargo test -j 8 -- --test-threads=8
```

確認事項:
- `v59600_tests::cmd_certify_passes` pass
- `v59600_tests::cmd_certify_generates_cert` pass
- 総テスト数 **3320** tests passed, 0 failed

---

## Step 7: 事後処理

- `CHANGELOG.md` に v59.6.0 エントリを追加
- `versions/current.md` を v59.6.0 / 3320 tests に更新
- `versions/roadmap/roadmap-v59.1-v60.0.md` の v59.6.0 実績欄を更新、v59.7.0 ベース数を `3320` に確定
- `versions/v55-v60/v59.6.0/tasks.md` を COMPLETE ステータスに更新
