# ============================================================
# ClipBoardX 交叉编译容器 (Linux → Windows x64)
# ============================================================
# 构建阶段
FROM rust:1.77-slim-bookworm AS builder

# 安装 mingw-w64 交叉编译工具链
RUN apt update && \
    apt install -y --no-install-recommends \
        gcc-mingw-w64-x86-64 \
        g++-mingw-w64-x86-64 \
        pkg-config \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# 添加 Windows x64 GNU 目标
RUN rustup target add x86_64-pc-windows-gnu && \
    rustup toolchain list

# 创建空项目先编译依赖（利用 Docker 层缓存）
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release --target x86_64-pc-windows-gnu 2>/dev/null || true

# 复制源码并真正编译
COPY src/ src/
RUN touch src/main.rs && \
    cargo build --release --target x86_64-pc-windows-gnu

# 输出产物
RUN ls -lh target/x86_64-pc-windows-gnu/release/clipboardx.exe

# ============================================================
# 最小运行镜像（仅含 exe，方便 copy）
# ============================================================
FROM scratch AS artifact
COPY --from=builder /app/target/x86_64-pc-windows-gnu/release/clipboardx.exe /
COPY --from=builder /app/target/x86_64-pc-windows-gnu/release/clipboardx.exe /clipboardx.exe

# ============================================================
# 开发镜像（带完整工具链，可进入交互式开发）
# ============================================================
FROM builder AS dev
WORKDIR /app
# 默认进入 bash，用户可以手动执行 cargo build
CMD ["bash"]
