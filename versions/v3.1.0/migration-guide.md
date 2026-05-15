# Favnir v3.1.0 Migration Guide

## Summary

v3.1.0 has no language-breaking changes. Existing v3.0.0 source code should continue to compile
without edits.

## New Features

### Local Docs UI

Use `fav docs` to inspect project explain metadata and stdlib APIs in a browser:

```text
fav docs src/main.fav
fav docs --port 8080 --no-open
```

### Version Output

The CLI now supports:

```text
fav --version
```

## No Required Source Changes

- No keyword changes
- No type-system changes
- No effect-system changes
- No migration step is required
