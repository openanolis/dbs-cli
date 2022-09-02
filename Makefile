# Copyright (c) 2019-2022 Alibaba Cloud. All rights reserved.
# Copyright (c) 2019-2022 Ant Group. All rights reserved.
# SPDX-License-Identifier: Apache-2.0

default: build

build:
	cargo build --all-features

check: clippy format

clippy:
	@echo "INFO: cargo clippy..."
	cargo clippy --all-targets --all-features \
		-- \
		-D warnings

format:
	@echo "INFO: cargo fmt..."
	cargo fmt -- --check

clean:
	cargo clean

test:
	@echo "INFO: testing dbs-cli for development build"
	cargo test --all-features -- --nocapture
