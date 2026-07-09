import { Alert, Button, Modal, Space, Table, Tag, Typography, message } from "antd";
import type { ColumnsType } from "antd/es/table";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { PlayCircle } from "lucide-react";
import { useState } from "react";

import {
  executeProductSeedPlan,
  getProductSeedPlan,
  listProductSeedExecutions,
  type ProductSeedExecuteEnvelope,
  type ProductSeedExecutionRecord,
  type ProductSeedPreflightCheck,
  type ProductSeedPreflightStatus,
  type ProductSeedStep
} from "../api/admin";
import { dateTime } from "../utils/format";
import { productAccessConfig } from "./access";
import { productAiBillingConfig, type ProductProviderModelTemplate } from "./ai";
import { productConfig } from "./config";
import {
  productSubscriptionConfig,
  type ProductSubscriptionPlan
} from "./subscription";

interface SeedItem {
  key: string;
  target: string;
  source: string;
  value: string;
}

const seedExecutionConfirmation = "INITIALIZE_PRODUCT";

const seedItems: SeedItem[] = [
  {
    key: "brand",
    target: "产品标识",
    source: "config.ts",
    value: `${productConfig.name} / ${productConfig.code}`
  },
  {
    key: "access",
    target: "默认接入配置",
    source: "access.ts",
    value: `${productAccessConfig.applicationDefaults.name} / ${productAccessConfig.applicationDefaults.slug} · ${productAccessConfig.applicationDefaults.authMode} · ${productAccessConfig.applicationDefaults.seatLimit} 席`
  },
  {
    key: "server-key",
    target: "服务端 Key 权限",
    source: "access.ts",
    value: productAccessConfig.serverApiKeyDefaults.scopes.join(", ")
  },
  {
    key: "subscription",
    target: "默认套餐",
    source: "subscription.ts",
    value: productSubscriptionConfig.defaultPlanCode
  },
  {
    key: "wallet",
    target: "钱包调整",
    source: "ai.ts",
    value: `${productAiBillingConfig.walletAdjustment.defaultReasons.credit} / ${productAiBillingConfig.walletAdjustment.defaultReasons.debit}`
  },
  {
    key: "ai-jobs",
    target: "AI 任务操作",
    source: "ai.ts",
    value: Object.values(productAiBillingConfig.jobActionReasons).join(" / ")
  }
];

const seedColumns: ColumnsType<SeedItem> = [
  {
    title: "对象",
    dataIndex: "target",
    key: "target",
    width: 180
  },
  {
    title: "配置源",
    dataIndex: "source",
    key: "source",
    width: 180,
    render: (value: string) => <Typography.Text code>{value}</Typography.Text>
  },
  {
    title: "当前值",
    dataIndex: "value",
    key: "value",
    render: (value: string) => (
      <Typography.Text ellipsis title={value}>
        {value}
      </Typography.Text>
    )
  }
];

const planColumns: ColumnsType<ProductSubscriptionPlan> = [
  {
    title: "套餐",
    dataIndex: "label",
    key: "label",
    width: 180,
    render: (value: string, record) => (
      <Space direction="vertical" size={0}>
        <Typography.Text>{value}</Typography.Text>
        <Typography.Text type="secondary">{record.code}</Typography.Text>
      </Space>
    )
  },
  {
    title: "默认席位",
    dataIndex: "defaultSeatLimit",
    key: "defaultSeatLimit",
    width: 120
  },
  {
    title: "功能标记",
    dataIndex: "features",
    key: "features",
    render: (features: string[]) => (
      <Space size={6} wrap>
        {features.map((feature) => (
          <Tag key={feature}>{feature}</Tag>
        ))}
      </Space>
    )
  }
];

const templateColumns: ColumnsType<ProductProviderModelTemplate> = [
  {
    title: "渠道",
    dataIndex: "provider_kind",
    key: "provider_kind",
    width: 140,
    render: (value?: string) => value ?? "all"
  },
  {
    title: "类型",
    dataIndex: "modality",
    key: "modality",
    width: 120
  },
  {
    title: "三方模型",
    dataIndex: "provider_model",
    key: "provider_model",
    width: 220,
    render: (value: string) => <Typography.Text code>{value}</Typography.Text>
  },
  {
    title: "名称",
    dataIndex: "name",
    key: "name",
    render: (value?: string) => value ?? "-"
  }
];

const backendStepColumns: ColumnsType<ProductSeedStep> = [
  {
    title: "步骤",
    dataIndex: "key",
    key: "key",
    width: 220,
    render: (value: string) => <Typography.Text code>{value}</Typography.Text>
  },
  {
    title: "目标",
    dataIndex: "target",
    key: "target",
    width: 240
  },
  {
    title: "模式",
    dataIndex: "mode",
    key: "mode",
    width: 150,
    render: (value: ProductSeedStep["mode"]) => <Tag>{seedStepModeLabel(value)}</Tag>
  },
  {
    title: "幂等键",
    dataIndex: "idempotency_key",
    key: "idempotency_key",
    width: 260,
    render: (value: string) => <Typography.Text code>{value}</Typography.Text>
  },
  {
    title: "配置源",
    dataIndex: "source",
    key: "source",
    render: (value: string) => (
      <Typography.Text ellipsis title={value}>
        {value}
      </Typography.Text>
    )
  }
];

const preflightColumns: ColumnsType<ProductSeedPreflightCheck> = [
  {
    title: "检查项",
    dataIndex: "key",
    key: "key",
    width: 230,
    render: (value: string) => <Typography.Text code>{value}</Typography.Text>
  },
  {
    title: "目标",
    dataIndex: "target",
    key: "target",
    width: 240
  },
  {
    title: "状态",
    dataIndex: "status",
    key: "status",
    width: 120,
    render: (value: ProductSeedPreflightStatus) => (
      <Tag color={seedCheckStatusColor(value)}>{seedCheckStatusLabel(value)}</Tag>
    )
  },
  {
    title: "结果",
    dataIndex: "message",
    key: "message",
    render: (value: string) => (
      <Typography.Text ellipsis title={value}>
        {value}
      </Typography.Text>
    )
  },
  {
    title: "已有记录",
    dataIndex: "existing_id",
    key: "existing_id",
    width: 250,
    render: (value?: string | null) =>
      value ? (
        <Typography.Text code copyable={{ text: value }}>
          {value}
        </Typography.Text>
      ) : (
        "-"
      )
  }
];

const executionColumns: ColumnsType<ProductSeedExecutionRecord> = [
  {
    title: "时间",
    dataIndex: "created_at",
    key: "created_at",
    width: 190,
    render: (value: string) => dateTime(value)
  },
  {
    title: "操作者",
    key: "actor",
    width: 220,
    render: (_, record) => (
      <Space direction="vertical" size={0}>
        <Typography.Text>{record.actor_name ?? record.actor_email ?? "-"}</Typography.Text>
        {record.actor_email ? (
          <Typography.Text type="secondary">{record.actor_email}</Typography.Text>
        ) : null}
      </Space>
    )
  },
  {
    title: "接入配置",
    key: "application",
    width: 260,
    render: (_, record) => (
      <SeedResourceCell
        createdId={record.created_application_id}
        existingId={record.existing_application_id}
      />
    )
  },
  {
    title: "服务端 Key",
    key: "server_api_key",
    width: 260,
    render: (_, record) => (
      <SeedResourceCell
        createdId={record.created_server_api_key_id}
        existingId={record.existing_server_api_key_id}
      />
    )
  },
  {
    title: "跳过项",
    dataIndex: "skipped",
    key: "skipped",
    render: (items: ProductSeedExecutionRecord["skipped"]) =>
      items.length ? (
        <Space size={6} wrap>
          {items.map((item) => (
            <Tag key={`${item.key}:${item.status}`}>
              {item.key}:{item.status}
            </Tag>
          ))}
        </Space>
      ) : (
        "-"
      )
  },
  {
    title: "请求 ID",
    dataIndex: "request_id",
    key: "request_id",
    width: 220,
    render: (value?: string | null) =>
      value ? (
        <Typography.Text code copyable={{ text: value }}>
          {value}
        </Typography.Text>
      ) : (
        "-"
      )
  }
];

export function ProductSetupPage() {
  const queryClient = useQueryClient();
  const [executionResult, setExecutionResult] = useState<ProductSeedExecuteEnvelope | null>(null);
  const seedPlanQuery = useQuery({
    queryKey: ["admin", "product-seed-plan"],
    queryFn: getProductSeedPlan
  });
  const executionHistoryQuery = useQuery({
    queryKey: ["admin", "product-seed-executions"],
    queryFn: () => listProductSeedExecutions({ page: 1, page_size: 20 })
  });
  const executeMutation = useMutation({
    mutationFn: executeProductSeedPlan,
    onSuccess: (result) => {
      setExecutionResult(result);
      message.success("初始化执行完成");
      queryClient.invalidateQueries({ queryKey: ["admin", "product-seed-plan"] });
      queryClient.invalidateQueries({ queryKey: ["admin", "product-seed-executions"] });
    },
    onError: (error) => {
      message.error(error instanceof Error ? error.message : "初始化执行失败");
    }
  });
  const backendPlan = seedPlanQuery.data?.plan;
  const preflight = seedPlanQuery.data?.preflight;

  const confirmExecution = () => {
    Modal.confirm({
      title: "执行产品初始化",
      content:
        "将创建缺失的默认接入配置和服务端 Key。新生成的 app_secret 和服务端 Key 明文只会在本次结果中显示一次。",
      okText: "执行初始化",
      cancelText: "取消",
      okButtonProps: {
        danger: true
      },
      onOk: () =>
        executeMutation.mutateAsync({
          confirm: seedExecutionConfirmation,
          create_server_api_key: true
        })
    });
  };

  return (
    <section className="workspace-page">
      <div className="page-heading">
        <div>
          <Typography.Title level={2}>初始化蓝图</Typography.Title>
          <Typography.Text type="secondary">
            当前 Product Layer 默认配置、套餐、权限和 AI 模板覆盖
          </Typography.Text>
        </div>
        <Button
          type="primary"
          danger
          icon={<PlayCircle size={16} />}
          loading={executeMutation.isPending}
          disabled={!preflight || preflight.summary.blocked}
          onClick={confirmExecution}
        >
          执行初始化
        </Button>
      </div>

      <Alert
        type="info"
        showIcon
        message="预览与受控初始化"
        description="执行前检查和 dry-run 计划默认只读；只有点击执行初始化并确认后，才会创建缺失的默认接入配置和服务端 Key。套餐和 AI 计费默认值仍作为配置项展示，不会写库。"
      />
      {seedPlanQuery.error ? (
        <Alert
          type="error"
          showIcon
          message="后端初始化计划加载失败"
          description="当前页面仍会展示前端 Product Layer 配置快照。"
        />
      ) : null}

      <div className="settings-grid">
        {executionResult ? (
          <section className="settings-panel settings-panel-wide">
            <div className="settings-panel-title">
              <Typography.Title level={3}>初始化执行结果</Typography.Title>
            </div>
            <Alert
              type="warning"
              showIcon
              message="密钥明文只显示一次"
              description="请在离开或刷新页面前记录新生成的 app_secret 和服务端 Key。已有资源不会回显历史明文。"
            />
            <div className="settings-grid-inner" style={{ marginTop: 12 }}>
              <ValueItem
                label="接入配置"
                value={
                  executionResult.execution.result.created_application
                    ? `已创建 ${executionResult.execution.result.created_application.id}`
                    : executionResult.execution.result.existing_application_id
                      ? `已存在 ${executionResult.execution.result.existing_application_id}`
                      : "-"
                }
              />
              <ValueItem
                label="服务端 Key"
                value={
                  executionResult.execution.result.created_server_api_key
                    ? `已创建 ${executionResult.execution.result.created_server_api_key.id}`
                    : executionResult.execution.result.existing_server_api_key_id
                      ? `已存在 ${executionResult.execution.result.existing_server_api_key_id}`
                      : "-"
                }
              />
              <ValueItem
                label="跳过项"
                value={executionResult.execution.result.skipped
                  .map((item) => `${item.key}:${item.status}`)
                  .join(" / ")}
              />
            </div>
            {executionResult.execution.result.created_application ? (
              <Space direction="vertical" size={6} style={{ marginTop: 12 }}>
                <Typography.Text type="secondary">接入密钥 app_secret</Typography.Text>
                <Typography.Text copyable code>
                  {executionResult.execution.result.created_application.app_secret}
                </Typography.Text>
              </Space>
            ) : null}
            {executionResult.execution.result.created_server_api_key ? (
              <Space direction="vertical" size={6} style={{ marginTop: 12 }}>
                <Typography.Text type="secondary">服务端 Key 明文</Typography.Text>
                <Typography.Text copyable code>
                  {executionResult.execution.result.created_server_api_key.plain_key}
                </Typography.Text>
              </Space>
            ) : null}
          </section>
        ) : null}

        <section className="settings-panel settings-panel-wide">
          <div className="settings-panel-title settings-panel-title-split">
            <Typography.Title level={3}>执行历史</Typography.Title>
            <Tag>最近 20 条</Tag>
          </div>
          {executionHistoryQuery.error ? (
            <Alert
              type="error"
              showIcon
              style={{ marginBottom: 12 }}
              message="初始化执行历史加载失败"
              description="当前页面仍可继续展示初始化计划和执行前检查。"
            />
          ) : null}
          <Table
            rowKey="id"
            loading={executionHistoryQuery.isLoading}
            columns={executionColumns}
            dataSource={executionHistoryQuery.data?.items ?? []}
            pagination={false}
            scroll={{ x: "max-content" }}
            locale={{ emptyText: "暂无初始化执行记录" }}
          />
        </section>

        <section className="settings-panel settings-panel-wide">
          <div className="settings-panel-title settings-panel-title-split">
            <Typography.Title level={3}>执行前检查</Typography.Title>
            {preflight ? (
              <Space size={6} wrap>
                <Tag color={preflight.summary.blocked ? "error" : "success"}>
                  {preflight.summary.blocked ? "存在冲突" : "无阻塞冲突"}
                </Tag>
                <Tag>已存在 {preflight.summary.exists}</Tag>
                <Tag>缺失 {preflight.summary.missing}</Tag>
                <Tag>人工核对 {preflight.summary.manual}</Tag>
                <Tag>配置项 {preflight.summary.config_only}</Tag>
              </Space>
            ) : null}
          </div>
          <Table
            rowKey="key"
            loading={seedPlanQuery.isLoading}
            columns={preflightColumns}
            dataSource={preflight?.checks ?? []}
            pagination={false}
            scroll={{ x: "max-content" }}
            locale={{ emptyText: "暂无执行前检查结果" }}
          />
        </section>

        <section className="settings-panel settings-panel-wide">
          <div className="settings-panel-title settings-panel-title-split">
            <Typography.Title level={3}>后端 dry-run 计划</Typography.Title>
            {backendPlan ? (
              <Space size={6}>
                <Tag color={backendPlan.destructive ? "error" : "success"}>
                  {backendPlan.destructive ? "破坏性" : "非破坏性"}
                </Tag>
                <Tag>v{backendPlan.version}</Tag>
                <Tag>scope: {backendPlan.idempotency_scope}</Tag>
              </Space>
            ) : null}
          </div>
          <Table
            rowKey="key"
            loading={seedPlanQuery.isLoading}
            columns={backendStepColumns}
            dataSource={backendPlan?.steps ?? []}
            pagination={false}
            scroll={{ x: "max-content" }}
            locale={{ emptyText: "暂无后端计划" }}
          />
          {backendPlan?.warnings.length ? (
            <Alert
              type="warning"
              showIcon
              style={{ marginTop: 12 }}
              message="执行前确认"
              description={backendPlan.warnings.join(" / ")}
            />
          ) : null}
        </section>

        <section className="settings-panel settings-panel-wide">
          <div className="settings-panel-title">
            <Typography.Title level={3}>初始化对象</Typography.Title>
          </div>
          <Table
            rowKey="key"
            columns={seedColumns}
            dataSource={seedItems}
            pagination={false}
            scroll={{ x: "max-content" }}
          />
        </section>

        <section className="settings-panel">
          <div className="settings-panel-title">
            <Typography.Title level={3}>接入默认值</Typography.Title>
          </div>
          <div className="settings-grid-inner">
            <ValueItem label="接入名称" value={productAccessConfig.applicationDefaults.name} />
            <ValueItem label="接入标识" value={productAccessConfig.applicationDefaults.slug} />
            <ValueItem label="认证模式" value={productAccessConfig.applicationDefaults.authMode} />
            <ValueItem
              label="默认席位"
              value={String(productAccessConfig.applicationDefaults.seatLimit)}
            />
            <ValueItem
              label="心跳间隔"
              value={`${productAccessConfig.applicationDefaults.heartbeatIntervalSeconds}s`}
            />
            <ValueItem
              label="离线容忍"
              value={`${productAccessConfig.applicationDefaults.offlineToleranceSeconds}s`}
            />
            <ValueItem
              label="服务端 Key 名称"
              value={productAccessConfig.serverApiKeyDefaults.name}
            />
            <ValueItem
              label="服务端权限"
              value={productAccessConfig.serverApiKeyDefaults.scopes.join(", ")}
            />
          </div>
        </section>

        <section className="settings-panel">
          <div className="settings-panel-title">
            <Typography.Title level={3}>AI 计费默认值</Typography.Title>
          </div>
          <div className="settings-grid-inner">
            <ValueItem
              label="充值原因"
              value={productAiBillingConfig.walletAdjustment.defaultReasons.credit}
            />
            <ValueItem
              label="扣减原因"
              value={productAiBillingConfig.walletAdjustment.defaultReasons.debit}
            />
            <ValueItem
              label="充值快捷金额"
              value={productAiBillingConfig.walletAdjustment.quickAmounts.credit.join(" / ")}
            />
            <ValueItem
              label="扣减快捷金额"
              value={productAiBillingConfig.walletAdjustment.quickAmounts.debit.join(" / ")}
            />
            <ValueItem
              label="失败释放原因"
              value={productAiBillingConfig.jobActionReasons.failRelease}
            />
            <ValueItem label="退款原因" value={productAiBillingConfig.jobActionReasons.refund} />
          </div>
        </section>

        <section className="settings-panel settings-panel-wide">
          <div className="settings-panel-title">
            <Typography.Title level={3}>套餐预设</Typography.Title>
          </div>
          <Table
            rowKey="code"
            columns={planColumns}
            dataSource={productSubscriptionConfig.plans}
            pagination={false}
            scroll={{ x: "max-content" }}
          />
        </section>

        <section className="settings-panel settings-panel-wide">
          <div className="settings-panel-title">
            <Typography.Title level={3}>项目模型模板覆盖</Typography.Title>
          </div>
          <Table
            rowKey={(record) => `${record.provider_kind ?? "all"}:${record.modality}:${record.provider_model}`}
            columns={templateColumns}
            dataSource={productAiBillingConfig.providerModelTemplates}
            pagination={false}
            scroll={{ x: "max-content" }}
            locale={{ emptyText: "暂无项目模板覆盖" }}
          />
        </section>

        {backendPlan ? (
          <section className="settings-panel settings-panel-wide">
            <div className="settings-panel-title">
              <Typography.Title level={3}>后端计划快照</Typography.Title>
            </div>
            <Typography.Paragraph
              copyable={{ text: JSON.stringify(backendPlan, null, 2) }}
              className="json-view"
            >
              {JSON.stringify(backendPlan, null, 2)}
            </Typography.Paragraph>
          </section>
        ) : null}

        <section className="settings-panel settings-panel-wide">
          <div className="settings-panel-title">
            <Typography.Title level={3}>前端配置快照</Typography.Title>
          </div>
          <Typography.Paragraph
            copyable={{ text: JSON.stringify(productSeedSnapshot, null, 2) }}
            className="json-view"
          >
            {JSON.stringify(productSeedSnapshot, null, 2)}
          </Typography.Paragraph>
        </section>
      </div>
    </section>
  );
}

function seedStepModeLabel(value: ProductSeedStep["mode"]): string {
  const labels: Record<ProductSeedStep["mode"], string> = {
    preview_only: "只读预览",
    create_if_missing: "缺失时创建",
    reconcile_manually: "人工核对"
  };

  return labels[value] ?? value;
}

function seedCheckStatusLabel(value: ProductSeedPreflightStatus): string {
  const labels: Record<ProductSeedPreflightStatus, string> = {
    exists: "已存在",
    missing: "缺失",
    conflict: "冲突",
    config_only: "配置项",
    manual: "人工核对"
  };

  return labels[value] ?? value;
}

function seedCheckStatusColor(value: ProductSeedPreflightStatus): string {
  const colors: Record<ProductSeedPreflightStatus, string> = {
    exists: "success",
    missing: "warning",
    conflict: "error",
    config_only: "processing",
    manual: "default"
  };

  return colors[value] ?? "default";
}

function SeedResourceCell({
  createdId,
  existingId
}: {
  createdId?: string | null;
  existingId?: string | null;
}) {
  const id = createdId ?? existingId;
  if (!id) {
    return "-";
  }

  return (
    <Space direction="vertical" size={2}>
      <Tag color={createdId ? "success" : "default"}>{createdId ? "已创建" : "已复用"}</Tag>
      <Typography.Text code copyable={{ text: id }}>
        {id}
      </Typography.Text>
    </Space>
  );
}

function ValueItem({ label, value }: { label: string; value: string }) {
  return (
    <Space direction="vertical" size={2}>
      <Typography.Text type="secondary">{label}</Typography.Text>
      <Typography.Text>{value || "-"}</Typography.Text>
    </Space>
  );
}

const productSeedSnapshot = {
  product: productConfig,
  access: productAccessConfig,
  subscription: productSubscriptionConfig,
  aiBilling: productAiBillingConfig
};
