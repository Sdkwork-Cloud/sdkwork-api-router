# 管理端 API

管理端服务在 `/admin/*` 下暴露 operator 控制平面。

## 基础地址与鉴权

- 默认本地基地址：`http://127.0.0.1:8081/admin`
- 登录流程：
  - `POST /admin/auth/login`
  - `GET /admin/auth/me`
  - `POST /admin/auth/change-password`

登录示例：

```bash
curl -X POST http://127.0.0.1:8081/admin/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email":"admin@sdkwork.local",
    "password":"ChangeMe123!"
  }'
```

把返回的 JWT 用在：

```bash
-H "Authorization: Bearer <jwt>"
```

最小校验请求：

```bash
curl http://127.0.0.1:8081/admin/auth/me \
  -H "Authorization: Bearer <jwt>"
```

密码修改：

```bash
curl -X POST http://127.0.0.1:8081/admin/auth/change-password \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <jwt>" \
  -d '{
    "current_password":"ChangeMe123!",
    "new_password":"AdminPassword456!"
  }'
```

## 路由家族

| 家族 | 路由 | 用途 |
|---|---|---|
| health 与 metrics | `GET /admin/health`、`GET /metrics` | 存活性与 Prometheus 风格 metrics |
| auth | `POST /auth/login`、`GET /auth/me`、`POST /auth/change-password` | operator 身份认证与密码轮换 |
| tenancy | `GET/POST /tenants`、`GET/POST /projects` | tenant 与 project 生命周期 |
| gateway access | `GET/POST /api-keys` | gateway API key 签发与查询 |
| provider catalog | `GET/POST /channels`、`GET/POST /providers`、`GET/POST /credentials`、`GET/POST /models` | 上游生态定义 |
| extensions | `GET/POST /extensions/installations`、`GET /extensions/packages`、`GET/POST /extensions/instances`、`GET /extensions/runtime-statuses`、`POST /extensions/runtime-reloads` | 扩展运行时管理 |
| extension rollouts | `GET/POST /extensions/runtime-rollouts`、`GET /extensions/runtime-rollouts/{rollout_id}` | 扩展 rollout 协调控制 |
| runtime config rollouts | `GET/POST /runtime-config/rollouts`、`GET /runtime-config/rollouts/{rollout_id}` | 配置重载 rollout 协调 |
| usage 与 billing | `GET /usage/records`、`GET /usage/summary`、`GET /billing/ledger`、`GET /billing/summary`、`GET/POST /billing/quota-policies` | 运维观察与配额约束 |
| routing | `GET/POST /routing/policies`、`GET /routing/health-snapshots`、`GET /routing/decision-logs`、`POST /routing/simulations` | 分发策略与诊断 |

## 管理端 API 负责什么

管理端 API 是以下系统事实的控制面入口：

- providers 与 credentials
- model catalog
- routing policy
- runtime rollout 状态
- usage 与 billing 汇总
- quota 控制

如果你需要改变网关的底层行为，这就是负责下发变更的 API。

## 浏览器应用

operator UI 是独立浏览器应用：

- `http://127.0.0.1:5173/admin/`

## 相关文档

- 服务边界：
  - [API 参考总览](/zh/api-reference/overview)
- 终端用户自助边界：
  - [门户 API](/zh/api-reference/portal-api)
- 架构上下文：
  - [软件架构](/zh/architecture/software-architecture)
