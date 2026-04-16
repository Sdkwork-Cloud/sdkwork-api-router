# 门户 API

门户服务在 `/portal/*` 下暴露终端用户自助边界。

## 基础地址与鉴权

- 默认本地基地址：`http://127.0.0.1:8082/portal`
- 健康检查：`GET /portal/health`
- 鉴权边界：portal JWT，与 admin JWT 完全独立

最小注册示例：

```bash
curl -X POST http://127.0.0.1:8082/portal/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email":"portal@example.com",
    "password":"PortalPass123!",
    "display_name":"Portal User"
  }'
```

本地开发时，如果 `SDKWORK_BOOTSTRAP_PROFILE=dev`，请先检查 `data/identities/dev.json`；或者直接通过 `/portal/auth/register` 注册账户。

使用已注入的门户身份登录：

```bash
curl -X POST http://127.0.0.1:8082/portal/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email":"<portal-email>",
    "password":"<portal-password>"
  }'
```

密码修改：

```bash
curl -X POST http://127.0.0.1:8082/portal/auth/change-password \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <portal-jwt>" \
  -d '{
    "current_password":"<current-portal-password>",
    "new_password":"<new-portal-password>"
  }'
```

## 路由家族

| 家族 | 路由 | 用途 |
|---|---|---|
| health | `GET /portal/health` | 存活性 |
| auth | `POST /portal/auth/register`、`POST /portal/auth/login`、`GET /portal/auth/me`、`POST /portal/auth/change-password` | 终端用户注册、会话生命周期与密码轮换 |
| workspace | `GET /portal/workspace` | 查看调用者拥有的默认工作区 |
| dashboard | `GET /portal/dashboard` | 返回当前项目的工作区身份、用量与计费组合快照 |
| usage | `GET /portal/usage/records`、`GET /portal/usage/summary` | 返回最近请求、token-unit 历史与聚合请求统计 |
| billing | `GET /portal/billing/summary`、`GET /portal/billing/ledger`、`GET /portal/billing/events`、`GET /portal/billing/events/summary`、`GET /portal/billing/account`、`GET /portal/billing/account/balance`、`GET /portal/billing/account-history`、`GET /portal/billing/account/benefit-lots`、`GET /portal/billing/account/holds`、`GET /portal/billing/account/request-settlements`、`GET /portal/billing/pricing-plans`、`GET /portal/billing/pricing-rates` | 返回 quota 态势、工作区范围内的 Billing 2.0 事件视图，以及面向租户的规范化商业账户可见性 |
| API keys | `GET /portal/api-keys`、`POST /portal/api-keys` | 自助查询和创建 gateway API key |

## 典型用户路径

1. 注册 portal 账户
2. 登录并获取 portal JWT
3. 查看默认 tenant 和 project 工作区
4. 打开 dashboard 快照查看最近请求、token units 与 quota 态势
5. 查看 usage 与 billing 明细视图
6. 创建 gateway API key
7. 使用该 key 调用 gateway 的 `/v1/*` 接口

## 浏览器应用

门户浏览器体验是独立应用：

- `http://127.0.0.1:5174/`

## 相关文档

- 产品使用流程：
  - [公开门户](/zh/getting-started/public-portal)
- operator 控制平面：
  - [管理端 API](/zh/api-reference/admin-api)

## 计费事件说明

- `GET /portal/billing/events` 仅返回当前认证工作区租户与项目范围内可见的计费事件
- `GET /portal/billing/events/summary` 仅对当前工作区可见的计费事件做聚合，聚合维度包括：
  - 项目
  - API key 分组
  - 能力
  - 计费模式
- 事件摘要会暴露网关事件账本已经记录的多模态维度，包括：
  - token 总量
  - `image_count`
  - `audio_seconds`
  - `video_seconds`
  - `music_seconds`

## 规范化商业账户说明

- `GET /portal/billing/account` 返回当前认证工作区的主商业账户，并汇总可用、冻结、消耗与授予余额态势
- `GET /portal/billing/account/balance` 返回同一工作区账户的实时余额快照
- `GET /portal/billing/account-history` 返回同一工作区账户及其账本时间线、有效权益批次、冻结记录和已结算请求，便于门户渲染完整的自助账户历史视图
- `GET /portal/billing/account/benefit-lots`、`GET /portal/billing/account/holds` 与 `GET /portal/billing/account/request-settlements` 都严格限定在当前工作区范围内，不会泄露其他租户或项目的账户数据
- `GET /portal/billing/pricing-plans` 与 `GET /portal/billing/pricing-rates` 仅暴露绑定到当前工作区商业范围的定价记录
- portal 商业账户相关路由会在账户缺失时自动补齐工作区主商业账户，因此已认证的工作区读取请求会返回规范化的空余额账户视图，而不是仅用于 bootstrap 的 `404`
