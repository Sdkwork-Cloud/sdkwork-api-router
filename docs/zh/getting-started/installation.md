# 安装准备

本页覆盖 Windows、Linux、macOS 上的环境依赖、仓库初始化和首次校验步骤。

## 你将安装什么

SDKWork API Router 不是单一二进制，而是一套工作区：

- 三个独立 Rust 服务
- 一个位于 `console/`、同时支持浏览器和 Tauri 的 operator 控制台
- 一个位于 `apps/sdkwork-router-portal/` 的独立浏览器 portal 应用
- 一个 VitePress 文档站
- 一组用于 provider 与扩展运行时的 Rust 工作区模块

建议采用以下最短路径进入可用状态：

1. 安装必需工具链
2. 克隆仓库
3. 安装 `console/`、`apps/sdkwork-router-portal/` 与 `docs/` 依赖
4. 继续阅读 [快速开始](/zh/getting-started/quickstart)

## 必需工具

所有平台都需要安装：

- Rust stable 与 Cargo
- Node.js 20+
- pnpm 10+

推荐但非必需：

- PostgreSQL 15+
- Tauri CLI

安装 Tauri CLI：

```bash
cargo install tauri-cli
```

## 平台说明

### Windows

推荐环境：

- 使用 `rustup` 安装 Rust
- Node.js 20+
- PowerShell 7 或 Windows PowerShell
- 如果使用 Tauri，确保系统可用 WebView2

### Linux

推荐环境：

- 使用 `rustup` 安装 Rust
- Node.js 20+
- 通过 Corepack 启用 pnpm 或单独安装
- 如果使用 Tauri，准备好系统 WebView 相关依赖

### macOS

推荐环境：

- 使用 `rustup` 安装 Rust
- Node.js 20+
- 通过 Corepack 启用 pnpm 或单独安装
- 安装 Xcode Command Line Tools

## 克隆与安装

克隆仓库：

```bash
git clone https://github.com/Sdkwork-Cloud/sdkwork-api-router.git
cd sdkwork-api-router
```

如果系统尚未提供 pnpm，先启用 Corepack：

```bash
corepack enable
```

安装 console 依赖：

```bash
pnpm --dir console install
```

安装 portal 应用依赖：

```bash
pnpm --dir apps/sdkwork-router-portal install
```

安装 docs 依赖：

```bash
pnpm --dir docs install
```

## 校验工具链

```bash
rustc --version
cargo --version
node --version
pnpm --version
```

如果需要 PostgreSQL：

```bash
psql --version
```

## 推荐的首次校验

不需要一开始就跑完整工作区回归，先做快速信心校验即可：

```bash
cargo fmt --all --check
pnpm --dir docs build
```

如果接下来要改控制台：

```bash
pnpm --dir console build
pnpm --dir apps/sdkwork-router-portal build
```

## 下一步

- 跑通一条可验证的首个请求路径：
  - [快速开始](/zh/getting-started/quickstart)
- 运行完整本地栈：
  - [源码运行](/zh/getting-started/source-development)
- 编译服务、admin 控制台、portal 应用、文档或桌面产物：
  - [编译与打包](/zh/getting-started/build-and-packaging)
- 生成可部署发布产物：
  - [发布构建](/zh/getting-started/release-builds)
- 在改代码前先理解系统形态：
  - [软件架构](/zh/architecture/software-architecture)

本地预览文档站：

```bash
pnpm --dir docs dev
```
