# ============================================================
# ClipBoardX 开发容器
# 带 Rust + mingw-w64 交叉编译工具链
# ============================================================
FROM rust:1.77-slim-bookworm

# 安装 mingw-w64（Windows 交叉编译需要）
RUN apt update && apt install -y --no-install-recommends \
        gcc-mingw-w64-x86-64 \
        g++-mingw-w64-x86-64 \
        pkg-config \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# 添加 Windows 编译目标
RUN rustup target add x86_64-pc-windows-gnu

# 常用工具（可选）
RUN cargo install cargo-watch 2>/dev/null || true
