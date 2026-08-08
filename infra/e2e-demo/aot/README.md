# Favnir AOT E2E Demo

Demonstrates `fav build --link`, `--docker`, and `--validate` on a pure transformation pipeline.

## Usage

```bash
./scripts/build-aot.sh
```

## Pipeline

`src/pipeline.fav` — pure OrderRow → SummaryRow transformation (no emit, AOT-compatible).

## Requirements

- `fav` binary built in release mode (`cargo build --release`)
- Docker (for `--docker` step)
