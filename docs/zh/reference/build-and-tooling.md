# 构建与工具链

本页汇总整个仓库使用的工具链、常用命令和辅助脚本。

如果你要查看完整的启停生命周期，请阅读 [脚本生命周期](/zh/getting-started/script-lifecycle)。

## 必需工具链

| 工具 | 用途 |
|---|---|
| Rust + Cargo | 所有后端 crate 和独立服务 |
| Node.js 20+ | 文档运行时、前端运行时、开发脚本 |
| pnpm 10+ | admin 应用、portal 应用和 docs 依赖管理 |
| Tauri CLI | 可选桌面开发与打包 |

## 主要命令

| 命令 | 范围 | 用途 |
|---|---|---|
| `cargo build -p gateway-service` | 后端 | 编译单个服务 |
| `cargo build --release -p admin-api-service -p gateway-service -p portal-api-service` | 后端 | 编译 release 二进制 |
| `cargo test --workspace -q -j 1` | 后端 | 工作区回归测试 |
| `cargo fmt --all --check` | 后端 | 格式校验 |
| `pnpm --dir apps/sdkwork-router-admin build` | 前端 | admin 生产构建 |
| `pnpm --dir apps/sdkwork-router-admin typecheck` | 前端 | admin TypeScript 校验 |
| `pnpm --dir apps/sdkwork-router-portal build` | 前端 | portal 生产构建 |
| `pnpm --dir apps/sdkwork-router-portal typecheck` | 前端 | portal TypeScript 校验 |
| `pnpm --dir apps/sdkwork-router-admin tauri:build` | 桌面 | 打包 admin Tauri 应用 |
| `pnpm --dir docs build` | 文档 | 构建 docs 站点 |
| `pnpm --dir docs typecheck` | 文档 | 校验 VitePress 配置类型 |

## 启动脚本矩阵

| 脚本 | 层级 | 生命周期角色 | 说明 |
|---|---|---|---|
| `./bin/start-dev.sh` / `.\bin\start-dev.ps1` | 托管开发态 | 启动托管开发运行时 | 默认进入 preview 模式，统一浏览器入口是 `9983` |
| `./bin/stop-dev.sh` / `.\bin\stop-dev.ps1` | 托管开发态 | 停止托管开发运行时 | 使用 `artifacts/runtime/dev/` 下的 PID 文件 |
| `./bin/build.sh` / `.\bin\build.ps1` | 托管发布态 | 构建可发布产物 | 写入 release 输出和 Rust 构建产物 |
| `./bin/install.sh` / `.\bin\install.ps1` | 托管发布态 | 创建安装目录 | 布置 `bin/`、`config/`、`sites/`、`service/` 和 `var/` |
| `./bin/start.sh` / `.\bin\start.ps1` | 托管发布态 | 启动安装后的发布运行时 | 启动 `router-product-service` 并打印统一与独立 URL |
| `./bin/stop.sh` / `.\bin\stop.ps1` | 托管发布态 | 停止安装后的发布运行时 | 使用安装目录中的 PID 文件 |
| `node scripts/dev/start-workspace.mjs` | 原生源码开发态 | 启动后端服务和浏览器前端 | 默认是 browser 模式，前端直接跑独立 dev server |
| `node scripts/dev/start-workspace.mjs --preview` | 原生源码开发态 | 启动后端服务和统一 Pingora Host | 统一 Host 为 `9983` |
| `node scripts/dev/start-workspace.mjs --tauri` | 原生源码开发态 | 启动后端服务和 admin 桌面壳 | 同时继续通过统一 Host 暴露浏览器入口 |
| `node scripts/dev/start-stack.mjs` | 原生源码开发态 | 仅启动后端服务 | 后端默认端口为 `9980`、`9981`、`9982` |
| `node scripts/dev/start-admin.mjs` | 原生源码开发态 | 仅启动 admin 应用 | 可跑浏览器或 Tauri |
| `node scripts/dev/start-portal.mjs` | 原生源码开发态 | 仅启动 portal 应用 | 仅浏览器 |
| `node scripts/dev/start-web.mjs --bind 0.0.0.0:9983` | 原生源码开发态 | 构建 admin / portal 静态资源并通过 Pingora 暴露 | 适合 preview 风格浏览器验证 |
| `scripts/dev/start-workspace.ps1` | 原生源码开发态 | Windows PowerShell 工作区启动包装 | 透传相同的 bind 和模式参数 |
| `scripts/dev/start-servers.ps1` | 原生源码开发态 | Windows PowerShell 后端启动包装 | 只启动后端 |

## 默认端口说明

- 托管脚本和辅助脚本默认使用 `998x`
- 原始服务二进制在未覆盖时仍保留内建 `808x`
- browser 模式下 admin 和 portal Vite dev server 仍分别使用 `5173`、`5174`

## 产物位置

| 产物 | 路径 |
|---|---|
| debug Rust 二进制 | `target/debug/` |
| release Rust 二进制 | `target/release/` |
| admin 浏览器资源 | `apps/sdkwork-router-admin/dist/` |
| portal 浏览器资源 | `apps/sdkwork-router-portal/dist/` |
| docs 站点构建产物 | `docs/.vitepress/dist/` |
| 托管开发运行目录 | `artifacts/runtime/dev/` |
| 托管安装目录 | `artifacts/install/sdkwork-api-router/current/` |

## 推荐校验组合

### 仅文档改动

```bash
pnpm --dir docs typecheck
pnpm --dir docs build
```

### 前端与文档同时改动

```bash
pnpm --dir apps/sdkwork-router-admin typecheck
pnpm --dir apps/sdkwork-router-admin build
pnpm --dir apps/sdkwork-router-portal typecheck
pnpm --dir apps/sdkwork-router-portal build
pnpm --dir docs typecheck
pnpm --dir docs build
```

### 全仓库高置信度校验

```bash
cargo fmt --all --check
cargo test --workspace -q -j 1
pnpm --dir apps/sdkwork-router-admin typecheck
pnpm --dir apps/sdkwork-router-admin build
pnpm --dir apps/sdkwork-router-portal typecheck
pnpm --dir apps/sdkwork-router-portal build
pnpm --dir docs typecheck
pnpm --dir docs build
```
