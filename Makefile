.PHONY: *

run:
	@export $$(grep -v '^#' ./dist/.env | xargs) && ./dist/server

server:
	export $$(grep -v '^#' ./dist/.env | xargs) && cargo build --release
	mkdir -p ./dist
	cp ./target/release/server ./dist/server

server-dbg:
	export $$(grep -v '^#' ./dist/.env | xargs) && cargo build
	mkdir -p ./dist
	cp ./target/debug/server ./dist/server

prepare:
	export $$(grep -v '^#' ./dist/.env | xargs) && cargo sqlx prepare

migrations:
	export $$(grep -v '^#' ./dist/.env | xargs) && cd ./server && sqlx migrate run

ui:
	cd ./ui && npm install && npm run build
	mkdir -p ./dist/ui
	cp -r ./ui/dist/* ./dist/ui/

db:
	mkdir -p /tmp/data
	export $$(grep -v '^#' ./dist/.env | xargs) && cargo sqlx database create

env:
	mkdir -p ./dist
	{ \
		echo "DATABASE_URL=sqlite:///tmp/data/data.db"; \
		echo "PORT=8080"; \
		echo "UI=./dist/ui"; \
	} > ./dist/.env

test-env:
	mkdir -p ./dist
	{ \
		echo "DATABASE_URL=sqlite:///tmp/data/data.db"; \
		echo "PORT=8080"; \
		echo "UI=./dist/ui"; \
	} > ./dist/.test.env

test: test-server

test-server:
	cargo test -- --nocapture

mail:
	mailtutan --maildir-path=/tmp

clean:
	rm -rf ./dist/
