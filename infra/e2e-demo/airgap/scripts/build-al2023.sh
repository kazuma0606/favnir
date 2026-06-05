#!/bin/bash
set -e
dnf install -y gcc gcc-c++ openssl-devel perl cmake make tar gzip which 2>&1 | tail -3
gcc --version | head -1
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --quiet
export PATH="$HOME/.cargo/bin:$PATH"
cd /fav
rm -rf target
CXXFLAGS='' cargo build --release 2>&1 | tail -5
if [ -f target/release/fav ]; then
  cp target/release/fav /fav/fav-al2023
  echo BUILD_SUCCESS
else
  echo BUILD_FAILED
fi
