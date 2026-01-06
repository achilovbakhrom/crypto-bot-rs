.PHONY: dev run build test fmt lint check clean db-up db-down db-reset migrate logs

# Development
dev:
	RUST_LOG=debug cargo run

run:
	cargo run

build:
	cargo build --release

# Testing & Quality
test:
	cargo test

fmt:
	cargo fmt

lint:
	cargo clippy

check:
	cargo check

clean:
	cargo clean

# Database
db-up:
	docker compose up -d

db-down:
	docker compose down

db-reset:
	docker compose down -v
	docker compose up -d
	sleep 2
	cd tools/migrate && go run . up

db-logs:
	docker compose logs -f postgres

# Migrations
migrate:
	cd tools/migrate && go run . up

migrate-down:
	cd tools/migrate && go run . down

migrate-status:
	cd tools/migrate && go run . status

migrate-fresh:
	cd tools/migrate && go run . fresh

# Setup
setup: db-up
	sleep 2
	cd tools/migrate && go run . up
	@echo "Setup complete. Run 'make dev' to start the application."
