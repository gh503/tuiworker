.PHONY: help build run test clean lint fmt doc check package-deb package-rpm package-all install-tools

help: ## 显示帮助信息
	@echo "TUI Workstation - 开发命令"
	@echo ""
	@echo "可用命令:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'

build: ## 构建项目（Debug）
	cargo build

build-release: ## 构建项目（Release）
	cargo build --release

run: ## 运行应用
	cargo run --bin tui-workstation

test: ## 运行所有测试
	cargo test

test-watch: ## 监听测试变化
	cargo watch -x test

lint: ## 运行 Clippy linter
	cargo clippy -- -D warnings

fmt: ## 格式化代码
	cargo fmt

fmt-check: ## 检查代码格式
	cargo fmt -- --check

doc: ## 生成文档
	cargo doc --open

doc-no-deps: ## 生成文档（无依赖）
	cargo doc --no-deps

check: ## 快速语法检查
	cargo check

clean: ## 清理构建缓存
	cargo clean

package-deb: build-release ## 构建 DEB 包
	cargo-deb --no-build
	@echo "DEB package built at: target/debian/*.deb"

package-rpm: build-release ## 构建 RPM 包
	cargo-generate-rpm --output=target/rpm/tuiworker.rpm
	@echo "RPM package built at: target/rpm/tuiworker.rpm"

package-all: package-deb package-rpm ## 构建 DEB 和 RPM 包
	@echo "All packages built successfully"

all: fmt lint build test ## 运行所有检查并构建

dev: fmt lint check ## 开发快速检查

install-tools: ## 安装开发工具和打包工具
	cargo install cargo-watch
	cargo install cargo-edit
	cargo install cargo-deb
	cargo install cargo-generate-rpm
