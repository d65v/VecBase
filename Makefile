.PHONY: all setup build run test clean fmt lint docker help

BINARY = vecbase
CARGO  = cargo
TARGET = vcore

all: build

## setup: Install system dependencies
setup:
	@echo "[VecBase] Running setup..."
	@bash SetUp/setup.sh

## build: Build release binary
build:
	@echo "[VecBase] Building..."
	@cd $(TARGET) && $(CARGO) build --release
	@echo "[VecBase] Done. Binary: $(TARGET)/target/release/$(BINARY)"

## run: Run the VecBase server
run:
	@echo "[VecBase] Starting..."
	@cd $(TARGET) && $(CARGO) run --release

## test: Run all tests
test:
	@echo "[VecBase] Testing..."
	@cd $(TARGET) && $(CARGO) test

## clean: Remove build artifacts
clean:
	@echo "[VecBase] Cleaning..."
	@cd $(TARGET) && $(CARGO) clean

## fmt: Format all Rust code
fmt:
	@cd $(TARGET) && $(CARGO) fmt

## lint: Run clippy linter
lint:
	@cd $(TARGET) && $(CARGO) clippy -- -D warnings

## docker: Build Docker image
docker:
	@echo "[VecBase] Building Docker image..."
	@docker build -t vecbase:latest .

## help: Show this help message
help:
	@grep -E '^## ' Makefile | sed 's/## /  /'
