#!/usr/bin/env bash
set -euo pipefail

cargo test --manifest-path rust/Cargo.toml --test fixture_discovery
cargo test --manifest-path rust/Cargo.toml --test docx_fixture_review_set
cargo test --manifest-path rust/Cargo.toml -p format_docx --test acceptance_docx_review_set
