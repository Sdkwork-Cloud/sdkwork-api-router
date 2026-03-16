# 运行模式

SDKWork API Server 目前支持两种主要运行形态：独立服务模式和桌面嵌入模式。

## 独立服务模式

独立服务模式是共享部署形态。

特点：

- 各服务以独立二进制运行
- gateway、admin、portal 都通过 HTTP 暴露
- PostgreSQL 是更适合部署环境的首选数据库
- 上游凭据更适合交由服务端 secret backend 管理

适用场景：

- 需要一个浏览器可访问的共享环境
- 需要多个 operator 或 portal 用户
- 需要挂在反向代理或服务管理器之后

## 嵌入模式

嵌入模式是面向桌面的部署形态。

特点：

- 运行时可以通过 runtime host 抽象在进程内承载
- Tauri 主要承载 operator 控制台，portal 保持浏览器优先
- 默认信任边界是本机 loopback
- SQLite 是更适合本地桌面场景的首选持久化方式

适用场景：

- 需要桌面优先的 operator 体验
- 在单机本地环境运行
- 希望 admin 控制台同时具备浏览器和桌面形态

## 浏览器与 Tauri 同时可用

开发时：

- `pnpm --dir console tauri:dev` 依赖 Vite dev server
- 同一个 admin 前端地址仍然可以从浏览器访问
- `start-workspace --tauri` 可以在一次启动流程里拉起后端服务和桌面壳，同时让 portal 继续运行在 `http://127.0.0.1:5174/`

## 下一步

- 本地启动与接入：
  - [源码运行](/zh/getting-started/source-development)
- 编译与打包：
  - [编译与打包](/zh/getting-started/build-and-packaging)
- 运行时和监督机制深挖：
  - [运行模式详解](/zh/architecture/runtime-modes)
