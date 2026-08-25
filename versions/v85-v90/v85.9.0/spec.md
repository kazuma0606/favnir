# Spec: v85.9.0 — 安定化・コードフリーズ

## Background

v85.1.0〜v85.8.0 で実装した SAP Foundation スプリント全機能を通しで確認する安定化バージョン。
新機能の追加は行わず、バグ修正と動作確認のみを受け入れる（コードフリーズ）。

v86.0.0「SAP Foundation 1.0 宣言」への移行前に、以下の全コンポーネントが正しく揃っているかを確認する。

| コンポーネント | 実装バージョン |
|---|---|
| `SapTomlConfig` + `inject_sap_config()` | v85.1.0 |
| `SapConfig` + `sap_config_from_env()` | v85.2.0 |
| Docker Compose モックサーバー | v85.3.0 |
| `runes/sap-odata/` 骨格 + `rune.toml` | v85.4.0 |
| OData v4 HTTP クライアント基盤 | v85.5.0 |
| `SapError` 型 + エラーコード | v85.6.0 |
| `fav.toml [sap]` テンプレート | v85.7.0 |
| SSM Parameter Store Terraform | v85.8.0 |

ロードマップ: `versions/roadmap/roadmap-v85.1-v86.0.md`（v85.9.0 セクション）

## Goals

- v85.1〜v85.8 の全コンポーネントが一貫して揃っていることを安定化テストで検証する
- Rust テスト 2 件を追加して **3,949 tests** を達成する
- バグ修正のみ受け入れ（新機能追加なし）

### 延期項目の明示

v85.6.0 で「パース処理の実装は v85.9.0 安定化バージョンで実施」と記載されていた OData v4 エラーレスポンスのパース処理は、安定化バージョンの性格（コードフリーズ）と矛盾するため **v85.9.0 では実施しない**。
v86.x 以降の適切なバージョンで改めて計画する。

## Files to Modify

| ファイル | 操作 | 内容 |
|---|---|---|
| `fav/src/driver.rs` | 追記 | `mod v85900_tests`（テスト 2 件） |

## テスト内容

### `sap_foundation_rune_toml_has_correct_name`

`runes/sap-odata/rune.toml` に `name = "sap-odata"` が含まれることを確認する。
v85.4.0 で作成した rune.toml の名前が変更されていないことのリグレッションガード。

```rust
fn sap_foundation_rune_toml_has_correct_name() {
    let content = std::fs::read_to_string("../runes/sap-odata/rune.toml")
        .expect("runes/sap-odata/rune.toml should exist");
    assert!(
        content.contains("sap-odata"),
        "rune.toml should have name = sap-odata"
    );
}
```

### `sap_foundation_docker_compose_has_sap_mock_service`

`infra/e2e-demo/sap-odata/docker-compose.yml` に `sap-mock` サービスが定義されていることを確認する。
v85.3.0 で作成したモックサーバーの骨格が intact であることのリグレッションガード。

```rust
fn sap_foundation_docker_compose_has_sap_mock_service() {
    let content = std::fs::read_to_string("../infra/e2e-demo/sap-odata/docker-compose.yml")
        .expect("docker-compose.yml should exist");
    assert!(
        content.contains("sap-mock"),
        "docker-compose.yml should define sap-mock service"
    );
}
```

## Success Criteria

- `cargo test` が **3,949 tests**, 0 failures
- `sap_foundation_rune_toml_has_correct_name`:
  - `runes/sap-odata/rune.toml` に `name        = "sap-odata"` が含まれる
- `sap_foundation_docker_compose_has_sap_mock_service`:
  - `infra/e2e-demo/sap-odata/docker-compose.yml` に `sap-mock` が含まれる

## Error Codes

新規エラーコードなし。

## 注記

- 安定化バージョンの性格上、テスト 2 件は既存コンポーネントの「存在と整合性」の確認のみ
- `rune.toml` の `name` フィールドの正確な文字列（スペース含む）を確認するため、`content.contains("name        = \"sap-odata\"")` のように整形後の文字列を使う
  （v85.4.0 で作成した rune.toml の実際の書式に合わせること。不明な場合は `content.contains("sap-odata")` に緩める）
- MILESTONE.md / README.md の更新は v86.0.0 宣言バージョンで実施する
