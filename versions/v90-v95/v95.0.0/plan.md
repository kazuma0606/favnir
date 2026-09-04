# Plan: v95.0.0 — SAP Advanced 1.0 宣言 ★クリーンアップ

## 依存関係

宣言バージョン。実装ではなく宣言・クリーンアップがメイン。
CHANGELOG → MILESTONE → README の順で更新後、driver.rs テスト更新、Cargo.toml 更新、cargo clean の順で実施する。

---

## Step 1: `CHANGELOG.md` に v95.0.0 エントリを追加する

先頭に追加する:

```markdown
## [v95.0.0] — 2026-08-30 — SAP Advanced 1.0 宣言

SAP Advanced Era（v94.1〜v94.9）の全機能完成を宣言する。

### Declaration
> 「`ctx.sap.batch(req)` で複数 SAP エンティティをまとめて更新できる。
>  `QueryBuilder<T>` で型安全なクエリを組み立て、`fetch_all_pages` で全件自動取得できる。
>  `fav infer --sap-metadata` で SAP の型定義が自動生成される。
>  Lambda SnapStart でコールドスタートは 93% 削減される。
>  それが、Favnir SAP Advanced 1.0 である。」

### Added
- `fav/src/driver.rs` — `mod v95000_tests`（テスト 4 件）を追加
  - `cargo_toml_version_is_95_0_0`: Cargo.toml バージョンが 95.0.0 である
  - `changelog_has_v95_0_0`: CHANGELOG.md に v95.0.0 が含まれる
  - `milestone_has_sap_advanced`: MILESTONE.md に SAP Advanced が含まれる
  - `readme_mentions_sap_advanced`: README.md に SAP Advanced が含まれる
- 合計テスト数: **4,164**（+4）
```

---

## Step 2: `MILESTONE.md` に v95.0.0 エントリを追加する

先頭（現在の `## v94.0.0` の前）に追加する:

```markdown
## v95.0.0（2026-08-30）— SAP Advanced 1.0 宣言

> 「`ctx.sap.batch(req)` で複数 SAP エンティティをまとめて更新できる。
>  `QueryBuilder<T>` で型安全なクエリを組み立て、`fetch_all_pages` で全件自動取得できる。
>  `fav infer --sap-metadata` で SAP の型定義が自動生成される。
>  Lambda SnapStart でコールドスタートは 93% 削減される。
>  それが、Favnir SAP Advanced 1.0 である。」

**SAP Advanced 1.0** の宣言バージョン。v94.1.0〜v94.9.0 で実装した
OData $batch / Lambda SnapStart / ベンチマーク / E2E デモ / ドキュメント完全化の完成を宣言した。
テスト数: 4,164。

**SAP Advanced 1.0（v94.1〜v94.9）達成内容:**
- **$batch**: `BatchOperation<T>` ADT / `BatchRequest<T>` / `batch_request_builder<T>` / `ctx.sap.batch(req)`
- **Lambda SnapStart**: `infra/lambda/sap-sync/main.tf`（SnapStart 設定）
- **コールドスタートベンチマーク**: `scripts/bench_sap_coldstart.sh`（93% 削減確認）
- **SAP 総合ベンチマーク**: `fav bench --sap`（`bench_sap_all` 6 ベンチマーク）
- **E2E デモ（シナリオ 5）**: `infra/e2e-demo/sap-odata/pipeline_advanced.fav`（$batch 完全デモ）
- **ドキュメント**: `site/content/docs/guides/sap-integration.mdx`（SAP Advanced Era 総まとめ）

---
```

---

## Step 3: `README.md` に v95.0「SAP Advanced 1.0」セクションを追加する

現在の `## v94.0 — SAP Metadata Infer 1.0 宣言` の前に追加する:

```markdown
## v95.0 — SAP Advanced 1.0 宣言（2026-08-30）

Favnir v95.0 で **SAP Advanced 1.0** を宣言しました。

`ctx.sap.batch(req)` で複数 SAP エンティティをまとめて更新できます。
`QueryBuilder<T>` で型安全なクエリを組み立て、Lambda SnapStart でコールドスタートを 93% 削減します。

```favnir
fn cleanup_partners(ctx: AppCtx) -> Result<String, String> {
    bind bps  <- ctx.sap.business_partners(BusinessPartnerFilter {
        country: Option.some("JP"), category: Option.none(),
        changed_after: Option.none(), top: Option.some(200)
    })
    bind ops  <- List.map(bps, fn(bp) { BatchDelete(bp.partner_id) })
    bind req  <- batch_request_builder("A_BusinessPartner", ops)
    bind resp <- ctx.sap.batch(req)
    Result.ok(String.concat("deleted ", Int.to_string(List.length(resp.succeeded))))
}
```

---
```

---

## Step 4: `versions/current.md` を v95.0.0 に更新する

- `最終更新: 2026-08-30 (v94.0.0)` → `最終更新: 2026-08-30 (v95.0.0)`
- `**v94.0.0** — SAP Metadata Infer 1.0 宣言` → `**v95.0.0** — SAP Advanced 1.0 宣言 — 4,164 tests`
- `v94.0 — SAP Metadata Infer 1.0 | **計画中**` → `**完了**`
- `v95.0 — SAP Advanced 1.0 | **計画中**` → `**完了**`
- 「進行中バージョン」セクションを更新する
- 「次に切る版」セクションを更新する

---

## Step 5: SAP Advanced Era ロードマップを「完了」マークする

以下 5 ファイルの先頭 Status 行を `完了` に更新する:
- `versions/roadmap/roadmap-v90.1-v91.0.md`
- `versions/roadmap/roadmap-v91.1-v92.0.md`
- `versions/roadmap/roadmap-v92.1-v93.0.md`
- `versions/roadmap/roadmap-v93.1-v94.0.md`
- `versions/roadmap/roadmap-v94.1-v95.0.md`

---

## Step 6: `driver.rs` の `cargo_toml_version_is_94_0_0` テストを stub 化する

ロードマップでは「旧 `cargo_toml_version` テストを一括更新」と記載しているが、
v94.0.0 以前の同名テストは各宣言バージョン時にすでに stub 化済みである。
Cargo.toml の実 version（`"94.0.0"`）を assert している唯一の生きているテストが
`v94000_tests::cargo_toml_version_is_94_0_0` のみのため、更新対象は 1 件のみ。

```rust
fn cargo_toml_version_is_94_0_0() {
    // stubbed: version has advanced to 95.0.0
}
```

---

## Step 7: `driver.rs` に `v95000_tests` 4 件を追加する

`mod v94900_tests { ... }` の直後に追加する:

```rust
#[cfg(test)]
mod v95000_tests {
    // use super::* は不要（std::fs のみ使用）
    #[test]
    fn cargo_toml_version_is_95_0_0() {
        let content = std::fs::read_to_string("Cargo.toml")
            .expect("Cargo.toml should exist");
        assert!(
            content.contains("version = \"95.0.0\""),
            "Cargo.toml should have version 95.0.0"
        );
    }

    #[test]
    fn changelog_has_v95_0_0() {
        let content = std::fs::read_to_string("../CHANGELOG.md")
            .expect("CHANGELOG.md should exist");
        assert!(
            content.contains("v95.0.0"),
            "CHANGELOG.md should mention v95.0.0"
        );
    }

    #[test]
    fn milestone_has_sap_advanced() {
        let content = std::fs::read_to_string("../MILESTONE.md")
            .expect("MILESTONE.md should exist");
        assert!(
            content.contains("SAP Advanced"),
            "MILESTONE.md should mention SAP Advanced"
        );
    }

    #[test]
    fn readme_mentions_sap_advanced() {
        let content = std::fs::read_to_string("../README.md")
            .expect("README.md should exist");
        assert!(
            content.contains("SAP Advanced"),
            "README.md should mention SAP Advanced"
        );
    }
}
```

---

## Step 8: `Cargo.toml` バージョンを `95.0.0` に更新する

`version = "94.0.0"` → `version = "95.0.0"`

---

## Step 9: `cargo clean` を実施する

```bash
cd fav && cargo clean
```

---

## Step 10: `cargo test` で 4,164 tests 確認

```bash
cargo test 2>&1 | grep "test result"
```

4,164 tests, 0 failures であることを確認する。

---

## Step 11: CI 事前確認

```bash
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
