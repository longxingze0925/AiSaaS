# AiSaaS

AiSaaS 是一个可复用的 SaaS + AI 计费底座，内置后台认证、用户、套餐订阅、余额账户、AI 网关、生成任务、素材缓存、审计和运维监控。

## 快速入口

- 三方业务后端接入：[docs/public/三方接口接入文档.md](docs/public/三方接口接入文档.md)
- 文档目录说明：[docs/README.md](docs/README.md)
- 内部开发文档：[docs/internal/development/](docs/internal/development/)
- 架构和完整接口资料：[docs/internal/architecture/](docs/internal/architecture/)
- 部署运维文档：[docs/internal/operations/](docs/internal/operations/)
- 历史兼容资料：[docs/archive/](docs/archive/)

## 一键安装

```bash
bash <(curl -Ls https://raw.githubusercontent.com/longxingze0925/AiSaaS/main/ops/install.sh)
```

一键脚本默认拉取 GHCR 上已经构建好的 `aisaas-backend` / `aisaas-admin` 镜像。只有设置 `AISAAS_DEPLOY_MODE=source` 时，才会在服务器本机从源码重新构建镜像。

## 本地组成

- `backend/`：Rust + Axum 后端服务。
- `admin/`：React + TypeScript 管理后台。
- `client-sdk/`：兼容区 SDK。
- `ops/`：安装、部署、备份、监控和 smoke 脚本。
- `docs/`：规范化文档目录。

## 对外接入原则

三方业务平台只应该阅读 `docs/public/三方接口接入文档.md`。不要直接暴露 `/api/admin/*`，也不要把 Server Key 放在浏览器、移动端或公开仓库中。

