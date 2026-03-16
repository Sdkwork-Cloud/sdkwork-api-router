# 编译与打包

本页是仓库的编译说明，适用于需要生成产物而不是仅从源码运行的场景。

## 构建目标

| 目标 | 命令 | 输出 |
|---|---|---|
| gateway 服务 | `cargo build --release -p gateway-service` | `target/release/gateway-service` |
| admin 服务 | `cargo build --release -p admin-api-service` | `target/release/admin-api-service` |
| portal 服务 | `cargo build --release -p portal-api-service` | `target/release/portal-api-service` |
| admin 控制台 | `pnpm --dir console build` | `console/dist/` |
| portal Web 应用 | `pnpm --dir apps/sdkwork-router-portal build` | `apps/sdkwork-router-portal/dist/` |
| 文档站点 | `pnpm --dir docs build` | `docs/.vitepress/dist/` |
| Tauri 桌面应用 | `pnpm --dir console tauri:build` | 平台相关的 Tauri 打包输出 |

## 编译独立服务

一次构建三个生产二进制：

```bash
cargo build --release -p admin-api-service -p gateway-service -p portal-api-service
```

日常开发通常只需要非 release 构建：

```bash
cargo build -p admin-api-service -p gateway-service -p portal-api-service
```

## 构建 admin 控制台

如有需要先安装依赖：

```bash
pnpm --dir console install
```

执行构建：

```bash
pnpm --dir console build
```

本地预览：

```bash
pnpm --dir console preview
```

## 构建独立 portal 应用

如有需要先安装依赖：

```bash
pnpm --dir apps/sdkwork-router-portal install
```

执行构建：

```bash
pnpm --dir apps/sdkwork-router-portal build
```

本地预览：

```bash
pnpm --dir apps/sdkwork-router-portal preview
```

## 构建文档站点

如有需要先安装依赖：

```bash
pnpm --dir docs install
```

执行构建：

```bash
pnpm --dir docs build
```

本地预览：

```bash
pnpm --dir docs preview
```

## 构建 Tauri 桌面应用

开发态桌面壳：

```bash
pnpm --dir console tauri:dev
```

生产打包：

```bash
pnpm --dir console tauri:build
```

当你需要一个内嵌服务运行时的桌面运维端，而不是分别部署浏览器端与独立服务时，应优先走这一条路径。

## 打包前推荐校验

```bash
cargo fmt --all --check
cargo test --workspace -q -j 1
pnpm --dir console build
pnpm --dir apps/sdkwork-router-portal build
pnpm --dir docs build
```

如果同时修改了 TypeScript 或 docs 配置：

```bash
pnpm --dir console -r typecheck
pnpm --dir apps/sdkwork-router-portal typecheck
pnpm --dir docs typecheck
```

## 相关文档

- 源码开发流程：
  - [源码运行](/zh/getting-started/source-development)
- 可部署发布产物：
  - [发布构建](/zh/getting-started/release-builds)
- 工作区结构：
  - [仓库结构](/zh/reference/repository-layout)
