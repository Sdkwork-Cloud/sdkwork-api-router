# 构建与工具链

本页汇总整个仓库涉及的工具链、常用命令和辅助脚本。

## 必需工具链

| 工具 | 用途 |
|---|---|
| Rust + Cargo | 所有后端 crate 与独立服务 |
| Node.js 20+ | 文档运行时、前端运行时、开发脚本 |
| pnpm 10+ | admin 控制台、portal 应用与 docs 依赖管理 |
| Tauri CLI | 可选的桌面开发与打包 |

## 主要命令

| 命令 | 范围 | 用途 |
|---|---|---|
| `cargo build -p gateway-service` | 后端 | 编译单个服务 |
| `cargo build --release -p admin-api-service -p gateway-service -p portal-api-service` | 后端 | 编译发布二进制 |
| `cargo test --workspace -q -j 1` | 后端 | 工作区回归测试 |
| `cargo fmt --all --check` | 后端 | 格式校验 |
| `pnpm --dir console build` | 前端 | admin 控制台生产构建 |
| `pnpm --dir console -r typecheck` | 前端 | admin 控制台 TypeScript 校验 |
| `pnpm --dir apps/sdkwork-router-portal build` | 前端 | 独立 portal 生产构建 |
| `pnpm --dir apps/sdkwork-router-portal typecheck` | 前端 | 独立 portal TypeScript 校验 |
| `pnpm --dir console tauri:build` | 桌面 | 打包 Tauri 应用 |
| `pnpm --dir docs build` | 文档 | 构建文档站点 |
| `pnpm --dir docs typecheck` | 文档 | 校验 VitePress 配置类型 |

## 开发启动脚本

| 脚本 | 用途 |
|---|---|
| `node scripts/dev/start-workspace.mjs` | 启动服务并拉起 admin 控制台与 portal 应用 |
| `node scripts/dev/start-workspace.mjs --tauri` | 启动服务并拉起 Tauri admin 桌面壳与浏览器 portal |
| `node scripts/dev/start-stack.mjs` | 仅启动后端服务 |
| `node scripts/dev/start-console.mjs` | 仅启动 admin 控制台 |
| `node scripts/dev/start-portal.mjs` | 仅启动 portal 应用 |
| `scripts/dev/start-workspace.ps1` | Windows PowerShell 完整工作区启动包装 |
| `scripts/dev/start-servers.ps1` | Windows PowerShell 仅后端启动包装 |
| `scripts/dev/start-console.ps1` | Windows PowerShell 仅 admin 控制台启动包装 |

## 产物位置

| 产物 | 路径 |
|---|---|
| debug Rust 二进制 | `target/debug/` |
| release Rust 二进制 | `target/release/` |
| admin 控制台资源 | `console/dist/` |
| portal Web 资源 | `apps/sdkwork-router-portal/dist/` |
| 文档站点构建产物 | `docs/.vitepress/dist/` |
| Tauri 打包产物 | `console/src-tauri/target/` 下的对应平台输出 |

## 推荐校验组合

### 仅文档改动

```bash
pnpm --dir docs typecheck
pnpm --dir docs build
```

### 前端与文档同时改动

```bash
pnpm --dir console -r typecheck
pnpm --dir console build
pnpm --dir apps/sdkwork-router-portal typecheck
pnpm --dir apps/sdkwork-router-portal build
pnpm --dir docs typecheck
pnpm --dir docs build
```

### 全仓库高置信度校验

```bash
cargo fmt --all --check
cargo test --workspace -q -j 1
pnpm --dir console -r typecheck
pnpm --dir console build
pnpm --dir apps/sdkwork-router-portal typecheck
pnpm --dir apps/sdkwork-router-portal build
pnpm --dir docs typecheck
pnpm --dir docs build
```
