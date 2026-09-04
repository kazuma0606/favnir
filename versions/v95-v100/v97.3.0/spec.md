# Spec: v97.3.0 — `!Approval` エフェクトマーカー + `ApprovalClient` interface

## Background

v97.2.0 でワークフロータスクの照会・完了操作を実装した。
v97.3.0 では「人間の承認が必要」という事実を型システムに持ち込む。

`!Approval` を pipeline シグネチャのエフェクトマーカーとして追加し、
`ApprovalClient` interface を `AppCtx` の `approval` フィールドとして注入できるようにする。
これにより「このパイプラインは人間の承認フローを必要とする」がコードから読み取れるようになる。

**設計方針**（ロードマップより）:
- `effect Approval { ... }` による独立エフェクト宣言は行わない（ctx パターン統一）
- `ApprovalClient` は `AppCtx` のフィールドとして注入（`SapClient` と同様のパターン）
- Rust 側の `Effect` enum に `Approval` バリアントを追加する

## Goals

1. `ApprovalClient` interface 型を `runes/sap-odata/workflow.fav` に追加する
2. `request_approval(subject, context)` スタブ関数を追加する
3. `runes/ctx/ctx.fav` に `approval: ApprovalClient` フィールドを追加する
4. Rust の `Effect` enum に `Approval` バリアントを追加する
5. `checker.fav` の exhaustive match（`ns_to_effect` / `builtin_ret_ty` 等）に `!Approval` を追加する
6. `fav/src/driver.rs` に `mod v97300_tests`（2 テスト）を追加する

## 型・API 定義

```favnir
-- runes/sap-odata/workflow.fav に追加

-- SapEventClient と同じ fn 構文パターン（第1引数 client: 自型）
public interface ApprovalClient {
    fn request_approval(client: ApprovalClient, subject: String, context: String) -> TaskDecision
}

-- スタブ実装: 常に Approve を返す（テスト・オフライン実行用）
-- subject / context は v97.x で実際の承認フロー API に使用予定（現在はスタブ）
public fn ApprovalClient.request_approval(client: ApprovalClient, subject: String, context: String) -> TaskDecision {
    Approve
}
```

```favnir
-- runes/ctx/ctx.fav への追加（approval フィールド）
-- AppCtx に approval: ApprovalClient を追加する
```

### 使用例

```favnir
-- !Approval マーカーを持つ pipeline
pipeline approve_purchase_order !SapOData !Approval {
    stage Request {
        bind po       <- ctx.sap.purchase_order_by_id(po_id, false)
        bind decision <- ctx.approval.request_approval(
            "発注 " ++ po.po_number ++ " の承認依頼",
            Json.encode(po)
        )
    }
    |> stage Apply {
        bind _ <- match decision {
            Approve     -> ctx.sap.workflow_start("POApproval", po_id)
            Reject(msg) -> ctx.io.println("却下: " ++ msg)
        }
    }
}
```

## Rust 側の変更（調査済み）

**`pub enum Effect {}` は Rust に存在しない**（driver.rs に `!src.contains("pub enum Effect {")` アサートがある）。
エフェクト追跡は `fav/self/checker.fav` の `ns_to_effect` 文字列マッピングで行われる。

ただし `ns_to_effect` は `IO.println` 等の**namespace 直接呼び出し**用であり、`ctx.sap.*()` のようなメソッドチェーンは追跡しない。`!SapOData` も `ns_to_effect` に登録されていない。

`!Approval` は pipeline シグネチャの**宣言的マーカー**（ドキュメント的役割）であり、`ns_to_effect` への追加は不要。`checker.fav` の変更も本バージョンのスコープ外とする。

## Success Criteria

- `cargo test` で 4,219 tests, 0 failures
- `cargo clippy --locked -- -D warnings` pass
- `./target/debug/fav fmt --check self/compiler.fav` pass
- `./target/debug/fav fmt --check self/checker.fav` pass
- `versions/current.md` が `v97.3.0`（4,219 tests）に更新されていること

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `runes/sap-odata/workflow.fav` | `ApprovalClient` interface + `request_approval` スタブ追加 |
| `runes/ctx/ctx.fav` | `approval: ApprovalClient` フィールド追加 |
| `fav/src/driver.rs` | `mod v97300_tests`（2 テスト）追加 |
| `CHANGELOG.md` | `[v97.3.0]` エントリ追加（先頭） |

**注**: `interface` キーワードは v9.12.0 で self-hosted 対応済み。`ApprovalClient` の定義は Favnir の `interface` 構文を使う。

**注**: `ctx.fav` が `workflow.fav` の `ApprovalClient` を参照するため、`ctx.fav` 先頭に `use sap_odata.workflow` の追加が必要になる場合がある（既存の `use` スタイルを確認して判断する）。
