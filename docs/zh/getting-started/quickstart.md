# 快速开始

本页是从克隆仓库到验证本地 SDKWork 栈成功运行的最短路径。

它采用与成熟 API 平台相近的上手结构：

1. 启动运行时
2. 验证控制平面
3. 创建终端用户账户
4. 签发网关 API key
5. 发起首个已鉴权网关请求

## 开始前

请先完成：

- [安装准备](/zh/getting-started/installation)

你需要：

- Rust + Cargo
- Node.js 20+
- pnpm 10+

## 第 1 步：启动完整栈

Linux / macOS：

```bash
node scripts/dev/start-workspace.mjs
```

Windows：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\dev\start-workspace.ps1
```

本地默认服务地址：

- gateway：`http://127.0.0.1:8080`
- admin：`http://127.0.0.1:8081`
- portal：`http://127.0.0.1:8082`
- 入口页：`http://127.0.0.1:5173/`
- admin 应用：`http://127.0.0.1:5173/admin/`
- portal 应用：`http://127.0.0.1:5174/`

## 第 2 步：验证运行时健康状态

```bash
curl http://127.0.0.1:8080/health
curl http://127.0.0.1:8081/admin/health
curl http://127.0.0.1:8082/portal/health
```

预期结果：每个端点都返回 `ok`。

## 第 3 步：登录管理控制平面

admin API 在首次使用时会自动写入默认本地 operator 账号：

- 邮箱：`admin@sdkwork.local`
- 密码：`ChangeMe123!`

```bash
curl -X POST http://127.0.0.1:8081/admin/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email":"admin@sdkwork.local",
    "password":"ChangeMe123!"
  }'
```

然后查看当前 admin 身份：

```bash
export ADMIN_JWT="<粘贴 token>"
curl http://127.0.0.1:8081/admin/auth/me \
  -H "Authorization: Bearer $ADMIN_JWT"
```

这一步用于确认控制平面可达，并且 JWT 签发工作正常。浏览器端的 operator UI
位于 `http://127.0.0.1:5173/admin/`。

## 第 4 步：注册门户用户或使用默认 portal 账号

公开门户是面向终端用户的自助边界。本地开发还会自动写入 portal 演示账号：

- 邮箱：`portal@sdkwork.local`
- 密码：`ChangeMe123!`

默认登录：

```bash
curl -X POST http://127.0.0.1:8082/portal/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email":"portal@sdkwork.local",
    "password":"ChangeMe123!"
  }'
```

或者新注册一个门户用户：

```bash
curl -X POST http://127.0.0.1:8082/portal/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email":"portal@example.com",
    "password":"hunter2!",
    "display_name":"Portal User"
  }'
```

响应中会包含：

- portal JWT
- 新创建的用户
- 默认 tenant 和 project 工作区

保存返回的 token：

```bash
export PORTAL_JWT="<粘贴 token>"
```

## 第 5 步：查看门户工作区

```bash
curl http://127.0.0.1:8082/portal/workspace \
  -H "Authorization: Bearer $PORTAL_JWT"
```

这一步确认新用户的默认工作区引导已经完成。

## 第 6 步：创建网关 API key

```bash
curl -X POST http://127.0.0.1:8082/portal/api-keys \
  -H "Authorization: Bearer $PORTAL_JWT" \
  -H "Content-Type: application/json" \
  -d '{"environment":"live"}'
```

响应会一次性返回 `plaintext` key，请立即保存：

```bash
export GATEWAY_KEY="<粘贴明文 key>"
```

## 第 7 步：发起首个网关请求

使用该 key 调用 OpenAI 兼容网关：

```bash
curl http://127.0.0.1:8080/v1/models \
  -H "Authorization: Bearer $GATEWAY_KEY"
```

预期结果：

- `200 OK`
- 返回标准 OpenAI 风格的列表响应
- 在你通过 admin API 配置 model catalog 和 provider 之前，`data` 可能为空

## 下一步

- 打开浏览器应用：
  - admin：`http://127.0.0.1:5173/admin/`
  - portal：`http://127.0.0.1:5174/`
- 理解三套 HTTP 接口面：
  - [API 参考总览](/zh/api-reference/overview)
- 配置 models、providers、credentials 和 routing：
  - [管理端 API](/zh/api-reference/admin-api)
- 理解运行时结构：
  - [软件架构](/zh/architecture/software-architecture)
- 编译二进制或前端产物：
  - [编译与打包](/zh/getting-started/build-and-packaging)
