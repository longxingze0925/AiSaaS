# SaaS + AI 计费改造方案

## 目标

本项目主线从“多客户端应用授权和分发后台”调整为“可复用的 SaaS + AI 计费底座”。

核心平台负责：

- 后台认证、团队、角色权限、审计。
- SaaS 用户、套餐订阅、余额账户、计费流水。
- AI 渠道、模型商品、生成任务、调用日志、素材缓存。
- 服务端 API Key 和 Web 后端接入。
- 通知、系统配置、异步任务和运维监控。

具体业务产品只在 Product Layer 内扩展，例如 AI 视频站、AI 图片站、AI 写作站、数字人产品或工作流工具。

## 分层

```text
SaaS Core
  auth / iam / tenant / team / customer / subscription / audit / notification / system

AI Billing Core
  ai / server_api / web_assets / web_works / outbox / media

Product Layer
  admin/src/product
  backend/src/modules/product
```

## 命名约定

为了降低一次性重构风险，数据库和后端接口暂时保留历史字段名：

- `customers` 在产品语义中表示 SaaS 用户。
- `applications` 在产品语义中表示默认产品接入配置或 API 接入配置。
- `max_devices` 在 SaaS 主线中先作为席位/绑定上限展示。
- `license`、`device`、`release`、`secure_script`、`client-sdk` 进入兼容区，不作为新主线入口。

UI 和新文档优先使用新语义：用户、套餐订阅、余额账户、AI 能力、API 接入、素材缓存。

## 魔改规则

后续针对具体项目魔改时，优先改：

1. `admin/src/product/config.ts`
2. `admin/src/product/access.ts`
3. `admin/src/product/subscription.ts`
4. `admin/src/product/ai.ts`
5. `admin/src/product/menu.tsx`
6. `admin/src/product/routes.tsx`
7. `backend/src/modules/product`

不要直接改核心模块来承载项目个性化需求。核心模块只放通用能力；项目模板、工作流、角色库、风格包、场景、专属作品结构等放 Product Layer。

前端魔改优先级：

- 品牌和首页文案：改 `config.ts`。
- API 接入默认值、服务端 Key 默认权限和权限文案：改 `access.ts`。
- 套餐编码、默认席位、功能标记：改 `subscription.ts`。
- 钱包充值/扣减默认原因、快捷金额、AI 模型模板、项目默认渠道能力、人工退款/释放原因：改 `ai.ts`。
- 具体产品页面和导航：改 `routes.tsx`、`menu.tsx`。
- 后端默认接入、套餐、功能标记、钱包和 AI 操作原因：改 `backend/src/modules/product/defaults.rs`。

`backend/src/modules/product/defaults.rs` 只作为默认配置源，不自动写数据库。后续如果要做一键初始化，应基于这里的配置新增显式命令或接口，并在执行前确认目标租户、数据库环境和幂等策略。

当前前端已提供 `/product/setup` 只读初始化蓝图页，用于核对 Product Layer 默认配置。该页面只展示和复制配置快照，不生成接入配置、服务端 Key、套餐或模型商品。

后端已提供 `backend/src/modules/product/seed_plan.rs`，用于生成可序列化的 dry-run 初始化计划。`GET /api/admin/product/seed-plan` 只读暴露该计划，供 `/product/setup` 展示。它包含步骤、幂等键建议、默认配置快照和风险提示，但当前不注册执行接口、不写数据库。

## AI 计费闭环

所有 AI 调用必须走统一计费闭环，不允许业务产品自行改余额：

```text
订阅校验
  -> 钱包检查
  -> hold 预扣
  -> 创建 usage/job
  -> 调用三方
  -> 缓存素材
  -> capture 结算 / release 释放 / refund 人工退款
```

规则：

- `hold`：请求进入三方前冻结预计金额，写入 `ai_wallet_ledger_entries`，不直接减少余额。
- `capture`：三方成功且平台完成必要结果处理后确认扣费，从余额和冻结金额中扣除。
- `release`：三方失败、提交失败、任务取消或人工标记失败时释放未结算预扣。
- `refund`：已经 `capture` 的成功任务需要人工退款时增加余额并记录退款流水。
- `ai_usage_records.price_snapshot_json` 必须保存当时价格和预扣金额，价格调整不能影响历史账单。
- `Idempotency-Key` 必须参与同步网关和异步任务的幂等约束，避免重试重复预扣或扣费。
- 结算逻辑统一放在 `backend/src/modules/ai/billing.rs`，同步网关和异步生成任务必须复用同一套 `settle_charge`。

状态约定：

- 同步网关：`running -> succeeded` 时结算，`running -> failed` 时释放预扣。
- 异步任务：`submitted/running/caching -> succeeded` 时结算，`provider_failed/failed/cancelled` 时释放预扣，`timeout_review` 保持人工确认，不自动猜测失败。
- 已成功扣费的任务不能走 `fail-release`，只能走 `refund`。

## 当前状态

已完成：

- 前端品牌和登录页切到 `AI SaaS Core`。
- 后台菜单切为用户与计费、AI 能力、运营系统。
- 旧授权/设备/版本/脚本入口从主导航移除，并保留路由重定向。
- 新增前端 Product Layer 配置、菜单和路由扩展点。
- 新增后端 Product Layer 模块边界。
- 抽出共享 AI 结算模块，避免同步网关和异步任务分叉。

下一步：

- 将 API 文档和数据库文档补充为“双语义过渡”：历史表名不变，新产品语义明确。
- 强化订阅、余额、用量、任务、素材的计费闭环说明。
- 再决定是否真正迁移数据库字段名或删除旧客户端授权模块。
