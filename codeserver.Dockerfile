# ============================================================
# code-server + Rust + mingw-w64 交叉编译
# 浏览器访问 http://localhost:8080 即可开发
# ============================================================
FROM codercom/code-server:latest

USER root

# 安装编译工具
RUN apt update && apt install -y --no-install-recommends \
        curl \
        build-essential \
        pkg-config \
        gcc-mingw-w64-x86-64 \
        g++-mingw-w64-x86-64 \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# 安装 Rust（使用中科大镜像源，国内下载快）
RUN curl --proto '=https' --tlsv1.2 -sSf https://mirrors.ustc.edu.cn/rust-static/rustup/rustup-init.sh \
    | RUSTUP_DIST_SERVER=https://mirrors.ustc.edu.cn/rust-static \
      sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
# 配置 cargo 国内镜像（中科大）
RUN mkdir -p /root/.cargo && \
    cat > /root/.cargo/config.toml << 'ENDCARGO'
[source.crates-io]
replace-with = "ustc"

[source.ustc]
registry = "sparse+https://mirrors.ustc.edu.cn/crates.io-index/"
ENDCARGO

# 添加 Windows 编译目标（使用中科大镜像）
RUN RUSTUP_DIST_SERVER=https://mirrors.ustc.edu.cn/rust-static \
    rustup target add x86_64-pc-windows-gnu

# 安装 VS Code 扩展（小扩展单独装，大扩展带重试）
RUN code-server --install-extension tamasfe.even-better-toml
RUN code-server --install-extension serayuzgur.crates
RUN code-server --install-extension vadimcn.vscode-lldb
RUN for i in 1 2 3; do \
        code-server --install-extension rust-lang.rust-analyzer && break || \
        echo "Attempt $i failed, retrying..." && sleep 3; \
    done

# 设置 VS Code 默认配置（rust-analyzer 指定 Windows 目标）
RUN mkdir -p /root/.local/share/code-server/User && \
    cat > /root/.local/share/code-server/User/settings.json << 'ENDJSON'
{
    "rust-analyzer.cargo.target": "x86_64-pc-windows-gnu",
    "rust-analyzer.check.command": "clippy",
    "editor.formatOnSave": true,
    "files.autoSave": "afterDelay"
}
ENDJSON

WORKDIR /workspace
