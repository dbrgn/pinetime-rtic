#!/bin/bash
set -euo pipefail

openocd -f jlink.ocd -c 'init; reset; halt'
