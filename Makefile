.PHONY: build dev clean exec

# ─── 一键构建 ────────────────────────────────────────────
build:
	cd code && docker compose run --rm build

# ─── 进入交互式开发环境 ──────────────────────────────────
dev:
	cd code && docker compose run --rm dev

# ─── 后台启动开发容器（保持运行） ─────────────────────────
dev-d:
	cd code && docker compose up -d dev && docker attach clipboardx-dev

# ─── 只编译，产物复制到当前目录 ──────────────────────────
release:
	cd code && docker compose run --rm build
	@echo "复制产物到本项目根目录..."
	@docker cp clipboardx-build:/app/target/x86_64-pc-windows-gnu/release/clipboardx.exe ./clipboardx.exe 2>/dev/null || true
	@echo "完成！"

# ─── 清理 ────────────────────────────────────────────────
clean:
	cd code && docker compose down -v 2>/dev/null || true
	docker rmi clipboardx-builder 2>/dev/null || true
