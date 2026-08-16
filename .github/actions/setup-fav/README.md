# setup-fav

GitHub Action to install the [Favnir](https://favnir.dev) compiler (`fav`) from GitHub Releases.

![Favnir CI](https://img.shields.io/badge/Favnir-CI-blue)

## Usage

### Basic

```yaml
steps:
  - uses: actions/checkout@v4
  - uses: favnir/setup-fav@v1
    with:
      version: "75.0.0"
  - run: fav check pipeline.fav
```

### With version pinning

```yaml
steps:
  - uses: actions/checkout@v4
  - uses: favnir/setup-fav@v1
    with:
      version: "73.8.0"
  - name: Type Check
    run: fav check pipeline.fav
  - name: Test
    run: fav test pipeline.fav
  - name: Quality Gate  # coming soon (v74+)
    run: fav quality report pipeline.fav --min-score 80 --fail-below
  - name: Audit  # coming soon (v74+)
    run: fav audit --deny-high
```

### Matrix build (multi-OS)

```yaml
jobs:
  ci:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: favnir/setup-fav@v1
        with:
          version: "75.0.0"
      - run: fav check pipeline.fav
      - run: fav test pipeline.fav
```

## Inputs

| Input | Required | Default | Description |
|---|---|---|---|
| `version` | Yes | `latest` | Favnir version to install |

## Binary URL format

Binaries are downloaded from:

```
https://github.com/favnir/favnir/releases/download/v{version}/fav-{os}-{arch}
```

- `os`: `linux` / `darwin` / `windows`
- `arch`: `x86_64` / `aarch64`
