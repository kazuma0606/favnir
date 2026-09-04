# Spec: v97.8.0 — サイトドキュメント（Workflow / Approval パターンガイド）

## Background

v97.1.0〜v97.7.0 で SAP Workflow 統合（WorkflowInstance / WorkflowTask / ApprovalClient /
条件分岐 pipeline / iFlowClient / MockWorkflowClient）の実装が完了した。
本バージョンでは、これらの機能を使ったガイドドキュメントを
`site/content/docs/guides/sap-workflow.mdx` として整備する。

## Goals

1. `site/content/docs/guides/sap-workflow.mdx` を新規作成する
   - Workflow / Approval パターンガイド
   - `!Approval` エフェクト型の使い方
   - iFlow connector の設定方法
   - E2E デモのウォークスルー
2. `fav/src/driver.rs` に `mod v97800_tests` を追加する（2 テスト）

## ドキュメント構成（概要）

```
---
title: "SAP Workflow & Approval Guide"
order: 11
category: "Guide"
description: "Favnir で SAP Workflow Management と BTP Integration Suite を統合する
              — ApprovalClient / !Approval / iFlowClient の完全ガイド"
---

# SAP Workflow & Approval Guide

## 全体像
- v97.1〜v97.7 の機能一覧テーブル

## 承認フローの型設計
- TaskDecision ADT（Approve / Reject(String)）
- ApprovalClient interface

## !Approval エフェクト型の使い方
- `!Approval` はエフェクト**マーカー**であり、実体は `ctx.approval.*()` メソッドで提供される（ctx パターン）
  — `effect Approval { ... }` による独立エフェクト宣言は行わない
- pipeline シグネチャに `!Approval` を記述する意味
- route_by_approval_result pipeline の解説

## iFlow connector の設定
- IFlowClient 型定義
- iflow_send スタブの使い方

## E2E デモのウォークスルー
- workflow_demo/ ディレクトリの構成
- bash run.sh の実行手順

## テスト戦略
- MockWorkflowClient の使い方（Ctx.mock_workflow）
```

## Success Criteria

- `site/content/docs/guides/sap-workflow.mdx` が存在する
- `sap-workflow.mdx` に `ApprovalClient` が含まれている
- `mod v97800_tests` の全テストが pass する
- `cargo test` で 4,229 tests, 0 failures（+2）

## Error Codes

新規エラーコードなし。

## Files to Modify

| ファイル | 種別 | 内容 |
|---|---|---|
| `site/content/docs/guides/sap-workflow.mdx` | 新規 | Workflow / Approval パターンガイド |
| `fav/src/driver.rs` | 追記 | `mod v97800_tests`（2 テスト） |
| `CHANGELOG.md` | 追記 | v97.8.0 エントリ |
| `versions/current.md` | 更新 | 最新安定版を v97.8.0 に変更 |
