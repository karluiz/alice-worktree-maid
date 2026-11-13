#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")"
./animate-rigid-sweeping.sh
./animate-rigid-greeting.sh
./animate-rigid-rest.sh
