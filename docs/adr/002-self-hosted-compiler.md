# ADR-002: コンパイラを Favnir でセルフホストする

- **状態**: 採用・完了
- **決定日**: v6.0.0（セルフホスト Phase A 開始）
- **完了**: v9.0.0（セルフホスト完成宣言）

---

## 背景

v4〜v5 の時点では、コンパイラ・型チェッカー・CLI はすべて Rust で実装されていた。
「Favnir が実用言語として成立するか」を証明するために、
コンパイラ自身を Favnir で書き直すというセルフホストの目標を設定した。

## 決定

以下のコンポーネントを Favnir で再実装し、Rust 実装を非推奨化する。

| コンポーネント | 場所 | 完了バージョン |
|---|---|---|
| コンパイラ | `fav/self/compiler.fav` | v8.5.0 |
| 型チェッカー | `fav/self/checker.fav` | v8.1.0 |
| CLI | `fav/self/cli.fav` | v7.6.0 |

## 正しさの証明方法（Bootstrap 検証）

セルフホストが「正しく」動作することを以下の手順で検証する：

```
Stage 1: Rust コンパイラで compiler.fav をコンパイル → bytecode_A
Stage 2: bytecode_A（Favnir compiler）で compiler.fav をコンパイル → bytecode_B
Stage 3: bytecode_B（Favnir compiler）で compiler.fav をコンパイル → bytecode_C

検証: bytecode_B == bytecode_C（不動点に達した）
```

この検証が通ることが「Favnir のコンパイラは Favnir として正しい」ことの根拠になる。
テスト: `cargo test bootstrap_full_self_hosting`

## 理由

1. **言語としての信頼性の証明**: コンパイラが自分自身をコンパイルできることは、
   言語が実用的な複雑さを扱えることを示す。

2. **Favnir 自身でのバグ発見**: Rust 実装にないバグが Favnir 実装で発見され、
   言語設計の改善につながった（実際に多数のバグを発見）。

3. **ドッグフーディング**: 開発者が Favnir を日常的に使うことになり、
   使い勝手の問題が早期に発見できる。

## 却下した選択肢

- **Rust のまま維持**: Favnir の実用性を証明できない。
- **Python 等で中間実装**: 三段階の実装管理が複雑になりすぎる。

## 影響

- `fav run` / `fav check` のデフォルトは Favnir パイプライン（`INVARIANTS.md` II-3）
- `--legacy` フラグで Rust パイプラインにフォールバック可能（`INVARIANTS.md` II-2）
- compiler.fav・checker.fav・cli.fav の self-check は CI で必須（`INVARIANTS.md` I-2）
