# 安装准备

本页覆盖 Windows、Linux 和 macOS 上的环境依赖、仓库初始化以及首次验证步骤。

## 你将安装什么

SDKWork API Router 不是单一二进制，而是一套多运行面的产品工作区，包含：

- 独立的 gateway、admin、portal Rust 服务
- 正式 server 产品使用的一体化 `router-product-service`
- 位于 `apps/sdkwork-router-admin/` 的仅用于源码开发的 admin 浏览器应用
- 位于 `apps/sdkwork-router-portal/` 的独立 portal 应用与正式 desktop 外壳
- VitePress 文档站
- provider 与 extension 运行时相关的 Rust 工作区

`apps/sdkwork-router-admin/` 属于源码工作区界面和 server 内嵌的 admin 界面，不是正式发布产品。

建议采用以下最短路径进入可用状态：

1. 安装必须工具链
2. 克隆仓库
3. 安装前端与文档依赖
4. 继续阅读 [快速开始](/zh/getting-started/quickstart)

## 必需工具

所有平台都需要安装：

- Rust stable 与 Cargo
- Node.js 20+
- pnpm 10+

推荐但非必需：

- PostgreSQL 15+，用于 server 侧部署
- Tauri CLI，用于 desktop 开发和打包

安装 Tauri CLI：

```bash
cargo install tauri-cli
```

## 平台说明

### Windows

推荐环境：

- 通过 `rustup` 安装 Rust
- Node.js 20+
- PowerShell 7 或 Windows PowerShell
- 如果使用 Tauri，确保系统可用 WebView2

### Linux

推荐环境：

- 通过 `rustup` 安装 Rust
- Node.js 20+
- 通过 Corepack 启用 pnpm 或单独安装
- 如果使用 Tauri，准备好系统 WebView 相关依赖

### macOS

推荐环境：

- 通过 `rustup` 安装 Rust
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

安装 admin 应用依赖：

```bash
pnpm --dir apps/sdkwork-router-admin install
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

Rust 工具链：

```bash
rustc --version
cargo --version
```

Node 与 pnpm：

```bash
node --version
pnpm --version
```

可选 PostgreSQL：

```bash
psql --version
```

## 推荐的首次校验

不需要一开始就跑完整工作区回归，先做一轮快速信心校验即可：

```bash
cargo fmt --all --check
pnpm --dir docs build
```

如果你准备修改浏览器或桌面运行面：

```bash
pnpm --dir apps/sdkwork-router-admin build
pnpm --dir apps/sdkwork-router-portal build
```

如果你准备修改正式 desktop 打包链路：

```bash
node scripts/prepare-router-portal-desktop-runtime.mjs
```

## 下一步

- 跑通一条可验证的首个请求链路：
  - [快速开始](/zh/getting-started/quickstart)
- 运行本地完整开发栈：
  - [源码运行](/zh/getting-started/source-development)
- 编译与打包正式产品：
  - [编译与打包](/zh/getting-started/build-and-packaging)
  - [发布构建](/zh/getting-started/release-builds)
- 查看线上部署与系统安装：
  - [生产部署](/zh/getting-started/production-deployment)
- 在深入修改前先理解系统形态：
  - [软件架构](/zh/architecture/software-architecture)

本地预览文档站：

```bash
pnpm --dir docs dev
```
