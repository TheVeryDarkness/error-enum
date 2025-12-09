#!/bin/bash

set -euxo pipefail

cargo publish -p error-enum-core
cargo publish -p error-enum-macros
cargo publish -p error-enum
