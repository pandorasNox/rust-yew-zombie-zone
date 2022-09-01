
.PHONY: clippy
clippy:
	cargo clippy -- -A clippy::needless_return
