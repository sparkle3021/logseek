# LogSeek

🚀 **高性能本地日志分析工具** — 基于 Rust + egui

[![Build](https://github.com/sparkle3021/logseek/actions/workflows/build.yml/badge.svg)](https://github.com/sparkle3021/logseek/actions)
[![Release](https://img.shields.io/github/v/release/sparkle3021/logseek)](https://github.com/sparkle3021/logseek/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

---

## ✨ 功能特性

### 📄 多格式支持
- **纯文本**: `.log`, `.txt`, `.out`
- **JSON Lines**: `.jsonl`, `.ndjson` — 自动提取字段
- **CSV**: `.csv` — 表头识别，引号支持

### 🔍 强大的搜索
- 普通搜索 + 正则表达式
- 大小写敏感/不敏感切换
- 全词匹配
- 搜索当前文件 / 搜索整个工作区
- 搜索结果聚合面板，点击跳转

### 🔽 实时过滤
- 关键词过滤
- 正则过滤
- 搜索 + 过滤联动

### 📂 工作区管理
- 创建/切换/删除工作区
- 默认工作区自动加载
- 文件管理（添加/移除）

### 📑 多标签页
- 同时打开多个日志文件
- 标签页切换
- 独立关闭

### 🔄 自动刷新
- Tail -f 模式（自动刷新）
- 文件更新检测 + 确认弹窗
- 刷新后保持滚动位置

### 🎨 界面定制
- 亮色/暗色主题切换
- 字体大小自定义（预设方案）
- 斑马纹 + 行号显示

### 🚀 高性能
- 内存映射 (mmap) — 大文件秒开
- 并行搜索 (rayon)
- 虚拟滚动 — 百万行流畅
- 索引缓存 — 二次打开更快

### 🌐 多编码支持
- UTF-8 / UTF-8 BOM / UTF-16
- GBK / GB2312 / GB18030
- Shift_JIS / EUC-KR

---

## 📦 下载

前往 [Releases](https://github.com/sparkle3021/logseek/releases) 下载最新版本：

| 平台 | 架构 | 文件 |
|------|------|------|
| Windows | x64 | `LogSeek-windows-x64.exe` |
| Windows | x86 | `LogSeek-windows-x86.exe` |
| Linux | x64 | `LogSeek-linux-x64` |
| Linux | x86 | `LogSeek-linux-x86` |
| macOS | x64 (Intel) | `LogSeek-macos-x64` |
| macOS | arm64 (Apple Silicon) | `LogSeek-macos-arm64` |

---

## 🚀 快速开始

### Windows
```bash
# 下载并运行
LogSeek-windows-x64.exe
```

### Linux
```bash
# 添加执行权限
chmod +x LogSeek-linux-x64
./LogSeek-linux-x64
```

### macOS
```bash
# 添加执行权限
chmod +x LogSeek-macos-x64
./LogSeek-macos-x64
```

**无需安装，单文件直接运行！**

---

## 🛠️ 从源码构建

### 环境要求
- Rust 1.70+
- Windows: Visual Studio Build Tools
- Linux: `libgtk-3-dev`, `libwebkit2gtk-4.1-dev`
- macOS: Xcode Command Line Tools

### 构建步骤
```bash
# 克隆仓库
git clone https://github.com/sparkle3021/logseek.git
cd logseek

# 构建 Release 版本
cargo build --release

# 运行
./target/release/LogSeek
```

---

## 📖 使用指南

### 基本操作
1. **打开文件**: 点击"打开文件"或创建工作区后导入
2. **搜索**: 输入关键词，按 Enter 或点击"搜索"
3. **过滤**: 勾选"过滤"，输入条件，自动应用
4. **切换主题**: 点击"亮色"/"暗色"按钮

### 快捷键
| 快捷键 | 功能 |
|--------|------|
| `Ctrl+F` | 聚焦搜索框 |
| `Enter` | 执行搜索 |

### 工作区
- 默认工作区在启动时自动加载
- 可创建多个工作区管理不同项目
- 文件在工作区内持久化

---

## 🏗️ 项目结构

```
logseek/
├── crates/
│   ├── logseek-core/    # 核心逻辑
│   │   ├── domain/      # 领域类型
│   │   ├── parser/      # 日志解析器
│   │   ├── source/      # 数据源管理
│   │   ├── query/       # 搜索引擎
│   │   ├── cache/       # 索引缓存
│   │   ├── watcher/     # 文件监听
│   │   ├── workspace/   # 工作区管理
│   │   └── utils/       # 工具函数
│   ├── logseek-ui/      # UI 层
│   │   ├── app/         # 应用状态
│   │   ├── panels/      # UI 面板
│   │   ├── views/       # 内容视图
│   │   ├── widgets/     # 可复用组件
│   │   └── theme/       # 主题系统
│   └── logseek-app/     # 入口点
├── assets/              # 图标资源
└── docs/                # 文档
```

---

## 🤝 贡献

欢迎贡献！请遵循以下步骤：

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

---

## 📄 许可证

本项目基于 [MIT License](LICENSE) 开源。
