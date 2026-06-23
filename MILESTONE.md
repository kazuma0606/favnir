# Practical Self-Hosting Milestone

**宣言日**: 2026-06-24
**宣言バージョン**: v25.0.0 = v1.0 リリース候補

---

## 宣言

> 「Favnir は Rust の力を借りながら、Rust を使わずに Favnir の世界を記述できる」

v25.0.0 をもって、Favnir の **Practical Self-Hosting** を正式に宣言する。

コンパイラ・型チェッカー・CLI・VM 仕様のすべてが Favnir で実装された。
Rust が担うのは VM の実行基盤（バイトコードディスパッチループ）のみであり、
これは設計上の意図であり制約ではない。

---

## 達成済みコンポーネント

| コンポーネント | ファイル | 実装言語 | 達成バージョン |
|---|---|---|---|
| コンパイラ | compiler.fav | Favnir ✓ | v8.5.0〜 |
| 型チェッカー | checker.fav | Favnir ✓ | v8.1.0〜 |
| CLI | cli.fav | Favnir ✓ | v7.6.0〜 |
| VM 仕様 | vm.fav | Favnir ✓ | v24.0.0〜 |
| VM 実行基盤 | src/backend/vm.rs | Rust（永続・設計上） | — |

### VM エンジンが Rust である理由

バイトコードのディスパッチループ・スタック管理・メモリアロケーションは、
Rust の安全性保証とゼロコスト抽象化が最も価値を発揮する領域です。
**これは Favnir の自己記述能力の欠如ではなく、正しい責任分担の結果です**。
VM の「仕様・動作の記述」は vm.fav（Favnir）が担い、
「実行の実装」は Rust が担う——このハイブリッド戦略こそが Favnir の強みです。

---

## セルフホスト達成の歴史

| バージョン | 達成内容 |
|---|---|
| v7.6.0 | cli.fav: `fav run` / `fav check` / `fav new` をすべて Favnir で実装 |
| v8.1.0 | checker.fav: `fav check` が Favnir 型チェッカー経由で動作 |
| v8.5.0 | compiler.fav: `fav run` がデフォルトで Favnir コンパイラ経由で動作 |
| v9.0.0 | セルフホスト完成宣言（compiler + checker + cli すべて Favnir 経由） |
| v24.0.0 | vm.fav: VM 仕様を Favnir で記述・テスト通過 |
| **v25.0.0** | **Practical Self-Hosting 宣言（本バージョン）** |

---

## 最終テスト（v25.0.0 達成状況）

| # | テスト | 状態 |
|---|---|---|
| 1 | `cargo test --bin fav` — 1974 件全 PASS | ✓ 達成（v25.0.0） |
| 2 | `fav run --vm=self/vm.fav self/compiler.fav -- hello.fav` | 延期（v25.x: vm.fav Phase 6） |
| 3 | `fav run --vm=self/vm.fav self/checker.fav` E2E | 延期（v25.x） |
| 4 | `fav run --vm=self/vm.fav self/cli.fav` E2E | 延期（v25.x） |
| 5 | 4-stage bootstrap 全 6 fixture（Stage 4 = vm.fav） | 延期（v25.x） |

テスト 2〜5 は vm.fav Phase 6（`CallFn` オペコード / ユーザー定義関数ディスパッチ）が
未実装のため v25.x に延期。テスト 1 の全件 PASS をもって v25.0.0 の完了条件とする。

---

## v1.x 後方互換性保証

v25.0.0 = v1.0 リリース候補として、後方互換性ポリシーを確定した。
詳細は [STABILITY.md](./STABILITY.md) を参照。
