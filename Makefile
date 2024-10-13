.PHONY: *

run:
	@export $$(grep -v '^#' ./dist/.env | xargs) && ./dist/server

server:
	export $$(grep -v '^#' ./dist/.env | xargs) && cd ./server && cargo build --release
	mkdir -p ./dist
	cp ./server/target/release/server ./dist/server

server-dbg:
	export $$(grep -v '^#' ./dist/.env | xargs) && cd ./server && cargo build
	mkdir -p ./dist
	cp ./server/target/debug/server ./dist/server

prepare:
	export $$(grep -v '^#' ./dist/.env | xargs) && cd ./server && cargo sqlx prepare

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

clean:
	rm -rf ./dist/
