#!/bin/bash
# E2E integration demo runner (v55-demo)
# Usage: ./run.sh
set -euo pipefail

fav run pipeline.fav --audit-log ./audit.log
