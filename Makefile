all: build

dist: clean build
	@cp target/release/copy .
	@cp target/release/ls .
	@cp target/release/data_node .
	@cp target/release/meta_data .
	@tar -czf assig-03-dfs.tar.gz src/ README.md copy ls data_node meta_data Cargo.* clean_db createdb.py
	@rm copy
	@rm ls
	@rm data_node
	@rm meta_data

clean:
	@rm -f assig-03-dfs.tar.gz


build:
	cargo build --release
