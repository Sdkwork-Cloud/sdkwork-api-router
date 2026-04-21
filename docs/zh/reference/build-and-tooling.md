# 构建与工具链

本页汇总仓库中使用的工具链、常用命令和辅助脚本。

完整的启动与停止生命周期，请参阅 [脚本生命周期](/zh/getting-started/script-lifecycle)。

## 必需工具链

| 工具 | 用途 |
|---|---|
| Rust + Cargo | 所有后端 crate 与独立服务 |
| Node.js 20+ | 文档运行时、前端运行时、开发辅助脚本 |
| pnpm 10+ | admin 应用、portal 应用与 docs 的依赖管理 |
| Tauri CLI | 可选的桌面开发与打包 |

## 主要命令

| 命令 | 范围 | 用途 |
|---|---|---|
| `cargo build -p gateway-service` | 后端 | 编译单个服务 |
| `node scripts/release/run-service-release-build.mjs --target <triple>` | 正式本地发布 | 使用托管 target 与 temp 路径构建受治理的 release 二进制集合 |
| `cargo build --release -p admin-api-service -p gateway-service -p portal-api-service -p router-web-service` | 后端 | 编译独立服务 release 二进制 |
| `cargo build --release -p router-product-service` | 后端 | 编译集成产品宿主运行时 |
| `cargo test --workspace -q -j 1` | 后端 | 工作区回归测试 |
| `cargo fmt --all --check` | 后端 | 格式校验 |
| `pnpm --dir apps/sdkwork-router-admin build` | 前端 | admin 生产构建 |
| `pnpm --dir apps/sdkwork-router-admin typecheck` | 前端 | admin TypeScript 校验 |
| `pnpm --dir apps/sdkwork-router-portal build` | 前端 | portal 生产构建 |
| `pnpm --dir apps/sdkwork-router-portal typecheck` | 前端 | portal TypeScript 校验 |
| `node scripts/check-router-product.mjs` | product verification | 运行受治理的 product gate，覆盖 portal/admin 校验、浏览器 smoke、Tauri capability 审计、桌面 release-like 载荷预制，以及仅回环地址的 server dry-run 计划 |
| `node scripts/check-tauri-capabilities.mjs` | desktop | 校验每个 desktop capability 都覆盖自动生成的 invoke 权限、受控 desktop bridge 实际需要的 window 权限，并确保 Tauri globals 与 `@tauri-apps/api/window` 只出现在治理白名单 bridge 文件中 |
| `node scripts/check-browser-storage-governance.mjs` | frontend governance | 校验浏览器 `localStorage` 与 `sessionStorage` 访问是否只存在于批准的受治理 store 模块中 |
| `./bin/build.sh --verify-release` / `.\bin\build.ps1 -VerifyRelease` | 正式本地 release 验证 | 执行受治理的本地正式发布路径，包含 docs 构建、打包运行时 smoke，以及 release governance preflight |
| `node scripts/prepare-router-portal-desktop-runtime.mjs` | 桌面 | 预制 portal desktop 的 sidecar 载荷到 `bin/portal-rt/router-product/` |
| `pnpm --dir apps/sdkwork-router-portal tauri:build` | 桌面 | 打包正式 portal desktop 安装器 |
| `pnpm --dir docs build` | 文档 | 构建 docs 站点 |
| `pnpm --dir docs typecheck` | 文档 | 校验 VitePress 配置类型 |

## 正式本地 Release 验证

`./bin/build.sh --verify-release` 和 `.\bin\build.ps1 -VerifyRelease` 是仓库内执行本地正式 release 校验的标准入口。

这个受治理的路径会始终包含：

- docs 站点构建，因为 docs surface 属于公开 release 契约的一部分
- 基于正式 release 资产树执行的打包 installed-runtime 与平台 smoke 校验
- 本地 `release governance preflight`，也就是 `node scripts/release/run-release-governance-checks.mjs --profile preflight`

不要把 `--skip-docs` 和 `--verify-release` 组合使用。如果你只是需要一次可以跳过 docs 治理的临时工程构建，请使用普通 build 模式，而不是正式的本地 release 验证模式。

## 启动脚本矩阵

| 脚本 | 层级 | 生命周期角色 | 说明 |
|---|---|---|---|
| `./bin/start-dev.sh` / `.\bin\start-dev.ps1` | 托管开发态 | 启动托管开发运行时 | 默认进入 preview 模式，并使用统一浏览器入口 `9983` |
| `./bin/stop-dev.sh` / `.\bin\stop-dev.ps1` | 托管开发态 | 停止托管开发运行时 | 使用 `artifacts/runtime/dev/` 下的 PID 文件 |
| `./bin/build.sh` / `.\bin\build.ps1` | 托管发布态 | 构建可发布产物 | 写入 release 输出与 Rust 构建产物 |
| `./bin/install.sh` / `.\bin\install.ps1` | 托管发布态 | 创建产品根目录 | 布置 `current/`、`releases/<version>/`、`config/`、`data/`、`log/`、`run/` |
| `./bin/start.sh` / `.\bin\start.ps1` | 托管发布态 | 启动安装后的发布运行时 | 从 `current/release-manifest.json` 指向的激活载荷启动 `router-product-service` |
| `./bin/stop.sh` / `.\bin\stop.ps1` | 托管发布态 | 停止安装后的发布运行时 | 使用产品根目录 `run/` 下的 PID 文件 |
| `node scripts/dev/start-workspace.mjs` | 原始源码开发态 | 启动后端服务和浏览器前端 | 默认是 browser 模式，直接跑前端 dev server |
| `node scripts/dev/start-workspace.mjs --preview` | 原始源码开发态 | 启动后端服务和统一 Pingora Host | 使用统一 `9983` Host |
| `node scripts/dev/start-workspace.mjs --tauri` | 原始源码开发态 | 启动后端服务和 portal desktop 壳 | 同时继续暴露统一浏览器入口 |
| `node scripts/dev/start-stack.mjs` | 原始源码开发态 | 只启动后端服务 | 后端端口默认为 `9980`、`9981`、`9982` |
| `node scripts/dev/start-admin.mjs` | 原始源码开发态 | 只启动 admin 应用 | 浏览器或 Tauri |
| `node scripts/dev/start-portal.mjs` | 原始源码开发态 | 只启动 portal 应用 | 浏览器或 Tauri |
| `node scripts/dev/start-web.mjs --bind 0.0.0.0:9983` | 原始源码开发态 | 构建 admin / portal 静态资源并通过 Pingora 暴露 | 适合 preview 风格验证 |
| `scripts/dev/start-workspace.ps1` | 原始源码开发态 | Windows PowerShell 工作区包装器 | 透传相同的模式与 bind 参数 |
| `scripts/dev/start-servers.ps1` | 原始源码开发态 | Windows PowerShell 后端包装器 | 只启动后端 |

## 默认端口说明

- 托管和辅助脚本默认使用 `998x` 端口段
- 原始服务二进制在没有覆盖时仍保留内建 `808x` 默认值
- 浏览器模式下 admin / portal 的 Vite dev server 仍在 `5173` 和 `5174`

## 产物位置

| 产物 | 路径 |
|---|---|
| debug Rust 二进制 | `target/debug/` |
| 原始 release Rust 二进制 | `target/release/` |
| 托管正式 release Rust 二进制 | `$CARGO_TARGET_DIR/<triple>/release/` |
| admin 浏览器资源 | `apps/sdkwork-router-admin/dist/` |
| portal 浏览器资源 | `apps/sdkwork-router-portal/dist/` |
| portal desktop sidecar 载荷 | `bin/portal-rt/router-product/` |
| docs 站点构建产物 | `docs/.vitepress/dist/` |
| 托管开发运行目录 | `artifacts/runtime/dev/` |
| 托管安装根目录 | `artifacts/install/sdkwork-api-router/` |

正式 release 服务二进制优先使用 `node scripts/release/run-service-release-build.mjs --target <triple>`，而不是直接运行原始 `cargo build --release`。
这个 runner 会复用受治理的 release build plan、打印实际输出目录，并在 Windows 上自动启用短路径 target 与 temp 根目录，避免官方 release 构建受到深路径限制影响。

## 推荐验证组合

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

## 前端治理约束

- portal 和 admin 的 desktop 能力只能通过受治理的 bridge 模块访问，`node scripts/check-tauri-capabilities.mjs` 会作为产品门禁强制校验。
- portal 认证会话必须保持 session-only。`readPortalSessionToken`、`persistPortalSessionToken`、`clearPortalSessionToken` 统一委托给 canonical user-center session store，`usePortalAuthStore` 只保留运行时内存态，通过 `hydrate()` 恢复身份与工作区上下文。
- 敏感浏览器持久化必须保持 session 级约束。会话 token、一次性 API key 明文 reveal，以及包含手机号/微信号等个人联系信息的 user-center 本地偏好草稿，只能通过受治理的 sessionStorage 模块或易失性内存兜底保存。遗留 `localStorage` 副本只能作为迁移来源，并且在受治理的 session store 接管后必须被清除。
- 非敏感浏览器偏好只有通过独立受治理的 store 模块时，才允许跨会话持久化。locale 这类 shell 偏好不能在 i18n 或功能入口里内联读写 `localStorage`。
- `node scripts/check-browser-storage-governance.mjs` 是这条浏览器持久化契约的静态门禁。任何新功能如果需要浏览器存储，必须先落到批准的受治理 store 模块中，而不是在功能入口里直接内联访问存储。
- 任何直接 source-link 到带显式 `.ts` 扩展名标准件的前端包，都必须在本地 `tsconfig.json` 中开启 `"allowImportingTsExtensions": true`。这属于产品 TypeScript 校验契约，不是可选便利项。

### 正式本地 Release 验证

```bash
./bin/build.sh --verify-release
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
