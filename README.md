# ClipboardX - 全局剪切板管理器

Windows 全局剪切板管理器，类手机输入法剪切板体验。

## 功能

- **全局热键 Alt+V** 呼出剪切板面板
- 记录剪切历史，支持搜索
- 双击条目自动粘贴到当前窗口
- 支持收藏、删除管理
- 自动写回跳过机制（防重复捕获）
- 自动清理过期记录

## 使用方法

1. 运行 `clipboardx.exe`
2. 任意软件中 **复制** 内容
3. 按 **Alt+V** 打开面板
4. 用 **↑↓** 选择，**双击** 粘贴 / **Enter** 粘贴

## 构建

```cmd
cd code
cargo build --release
```

生成 `target\release\clipboardx.exe`。

## 技术栈

- Rust
- Win32 API（raw FFI）
- SQLite（rusqlite + bundled）
