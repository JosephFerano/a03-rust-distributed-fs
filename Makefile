all: build

dist: build
	@rm -f assig-03-dfs.tar.gz
	@cp target/release/copy copy
	@tar -czf assig-03-dfs.tar.gz src/ README.md copy
	@rm copy

build:
	cargo build --release
