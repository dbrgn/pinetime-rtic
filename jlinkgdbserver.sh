#!/bin/bash
set -euo pipefail

JLinkGDBServerExe \
    -select USB \
    -device nRF52832_xxAA \
    -endian little \
    -if SWD \
    -speed 4000 \
    -noir \
    -LocalhostOnly
