Build the Favnir compiler.

Default (no $ARGUMENTS): debug build
```bash
cd /c/Users/yoshi/favnir/fav && cargo build -j 8 2>&1 | grep -v "^warning\|Compiling\|^$" | tail -20
```

If $ARGUMENTS is `release`:
```bash
cd /c/Users/yoshi/favnir/fav && cargo build --release -j 8 2>&1 | grep -v "^warning\|Compiling\|^$" | tail -20
```

If $ARGUMENTS is `wasm`:
```bash
cd /c/Users/yoshi/favnir/fav && cargo build --target wasm32-unknown-unknown --lib -j 8 2>&1 | grep -v "^warning\|Compiling\|^$" | tail -20
```

If $ARGUMENTS is `check`:
```bash
cd /c/Users/yoshi/favnir/fav && cargo check -j 8 2>&1 | grep "^error" | head -20
```

Report: build success with binary size (for release), or all error lines on failure.
