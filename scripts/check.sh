#!/bin/bash
set -eo pipefail

# (c) Meta Platforms, Inc. and affiliates.
#
# Use of this source code is governed by an MIT-style
# license that can be found in the LICENSE file or at
# https://opensource.org/licenses/MIT.

echo "Running CI checks..."

cargo fmt --check

# ensure the tests run ok with all features disabled
cargo test

cargo test --locked --all-features

# docs-rs supersedes cargo doc
cargo +nightly docs-rs

# finish with coverage, so we get an output to check
cargo llvm-cov --locked --all-features
