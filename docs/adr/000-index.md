# Architectural Decision Records

Favnir の主要な設計決定を記録します。
ADR は「なぜそうしたか」を残すためのものです。実装の詳細は各ソースファイルを参照してください。

| 番号 | タイトル | 状態 |
|------|----------|------|
| [001](./001-vm-stays-rust.md) | VM は Rust で実装する | 採用 |
| [002](./002-self-hosted-compiler.md) | コンパイラを Favnir でセルフホストする | 採用 |
| [003](./003-legacy-flag-kept.md) | `--legacy` フラグは削除せず非推奨に留める | 採用 |
| [004](./004-effect-system.md) | 副作用をエフェクト型で追跡する | 採用 |
| [005](./005-single-binary.md) | `fav` 単一バイナリにすべての機能を統合する | 採用 |
