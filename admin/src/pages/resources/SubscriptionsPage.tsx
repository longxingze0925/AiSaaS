import {
  Alert,
  AutoComplete,
  Button,
  DatePicker,
  Form,
  Input,
  InputNumber,
  Modal,
  Select,
  Space,
  Typography,
  message
} from "antd";
import type { ColumnsType } from "antd/es/table";
import type { Dayjs } from "dayjs";
import {
  useInfiniteQuery,
  useMutation,
  useQuery,
  useQueryClient
} from "@tanstack/react-query";
import {
  Ban,
  CalendarPlus,
  Pause,
  Play,
  Plus,
  RefreshCw,
  RotateCcw
} from "lucide-react";
import { useEffect, useMemo, useState } from "react";

import {
  cancelSubscription,
  createSubscription,
  listApplications,
  listCustomers,
  listSubscriptions,
  renewSubscription,
  resetSubscriptionDevices,
  resumeSubscription,
  suspendSubscription,
  type CreateSubscriptionPayload,
  type SubscriptionSummary
} from "../../api/admin";
import { ConfirmActionButton } from "../../components/ConfirmActionButton";
import { HistoryToggle } from "../../components/HistoryToggle";
import { InfiniteTable } from "../../components/InfiniteTable";
import { StatusTag } from "../../components/StatusTag";
import { productSubscriptionConfig } from "../../product/subscription";
import { useAuthStore } from "../../stores/authStore";
import { dateTime, shortId } from "../../utils/format";
import {
  INFINITE_TABLE_QUERY_OPTIONS,
  trimInfiniteQueryCache
} from "../../utils/infiniteQueryCache";
import { effectiveTemporalStatus, tMessage, tOption } from "../../utils/i18n";
import { hasPermission } from "../../utils/permissions";

const pageSize = 30;

interface CreateSubscriptionFormValues {
  app_id: string;
  customer_id: string;
  plan: string;
  max_devices?: number | null;
  starts_at?: Dayjs;
  expires_at?: Dayjs;
  features?: string[];
}

interface RenewSubscriptionFormValues {
  expires_at?: Dayjs;
}

interface ResetDevicesFormValues {
  reason?: string;
}

export function SubscriptionsPage() {
  const [keyword, setKeyword] = useState("");
  const [status, setStatus] = useState<string | undefined>();
  const [includeHistory, setIncludeHistory] = useState(false);
  const [createOpen, setCreateOpen] = useState(false);
  const [renewTarget, setRenewTarget] = useState<SubscriptionSummary | null>(null);
  const [resetDevicesTarget, setResetDevicesTarget] =
    useState<SubscriptionSummary | null>(null);
  const [form] = Form.useForm<CreateSubscriptionFormValues>();
  const [renewForm] = Form.useForm<RenewSubscriptionFormValues>();
  const [resetDevicesForm] = Form.useForm<ResetDevicesFormValues>();
  const queryClient = useQueryClient();
  const permissions = useAuthStore((state) => state.permissions);
  const canCreate = hasPermission(permissions, "subscription:create");
  const canCancel = hasPermission(permissions, "subscription:cancel");
  const canSuspend = hasPermission(permissions, "subscription:suspend");
  const canResume = hasPermission(permissions, "subscription:resume");
  const canRenew = hasPermission(permissions, "subscription:renew");
  const canResetDevice = hasPermission(permissions, "subscription:reset_device");
  const query = useInfiniteQuery({
    ...INFINITE_TABLE_QUERY_OPTIONS,
    queryKey: ["admin", "subscriptions", keyword, status, includeHistory],
    queryFn: ({ pageParam }) =>
      listSubscriptions({
        keyword,
        status,
        include_history: includeHistory,
        page: Number(pageParam),
        page_size: pageSize
      }),
    initialPageParam: 1,
    getNextPageParam: (lastPage) =>
      lastPage.meta?.has_more || lastPage.meta?.hasMore
        ? lastPage.meta.page + 1
        : undefined
  });

  useEffect(() => {
    return () => {
      trimInfiniteQueryCache(queryClient, ["admin", "subscriptions"]);
    };
  }, [queryClient]);

  const refetchFirstSubscriptionPage = async () => {
    trimInfiniteQueryCache(queryClient, ["admin", "subscriptions"]);
    await query.refetch();
  };

  const appsQuery = useQuery({
    queryKey: ["admin", "apps", "active-options"],
    queryFn: () => listApplications({ status: "active", page: 1, page_size: 100 }),
    enabled: createOpen
  });
  const customersQuery = useQuery({
    queryKey: ["admin", "customers", "active-options"],
    queryFn: () => listCustomers({ status: "active", page: 1, page_size: 100 }),
    enabled: createOpen
  });
  const createMutation = useMutation({
    mutationFn: createSubscription,
    onSuccess: async () => {
      message.success(tMessage("subscription_created"));
      setCreateOpen(false);
      form.resetFields();
      await refetchFirstSubscriptionPage();
    }
  });
  const cancelMutation = useMutation({
    mutationFn: cancelSubscription,
    onSuccess: async (data) => {
      message.success(tMessage(`subscription_cancelled:${data.revoked_sessions}`));
      await refetchFirstSubscriptionPage();
    }
  });
  const suspendMutation = useMutation({
    mutationFn: suspendSubscription,
    onSuccess: async (data) => {
      message.success(tMessage(`subscription_suspended:${data.revoked_sessions}`));
      await refetchFirstSubscriptionPage();
    }
  });
  const resumeMutation = useMutation({
    mutationFn: resumeSubscription,
    onSuccess: async () => {
      message.success(tMessage("subscription_resumed"));
      await refetchFirstSubscriptionPage();
    }
  });
  const renewMutation = useMutation({
    mutationFn: renewSubscription,
    onSuccess: async () => {
      message.success(tMessage("subscription_renewed"));
      setRenewTarget(null);
      renewForm.resetFields();
      await refetchFirstSubscriptionPage();
    }
  });
  const resetDevicesMutation = useMutation({
    mutationFn: resetSubscriptionDevices,
    onSuccess: async (data) => {
      message.success(tMessage(`subscription_devices_reset:${data.revoked_sessions}`));
      setResetDevicesTarget(null);
      resetDevicesForm.resetFields();
      await refetchFirstSubscriptionPage();
    }
  });
  const planOptions = productSubscriptionConfig.plans.map((plan) => ({
    value: plan.code,
    label: `${plan.label} · ${plan.code}`
  }));
  const items = useMemo(
    () => (query.data?.pages ?? []).flatMap((page) => page.items),
    [query.data]
  );

  const applyPlanDefaults = (planCode: string) => {
    const plan = productSubscriptionConfig.plans.find((item) => item.code === planCode);
    if (!plan) {
      return;
    }

    form.setFieldsValue({
      max_devices: plan.defaultSeatLimit,
      features: plan.features
    });
  };

  const columns: ColumnsType<SubscriptionSummary> = [
    {
      title: "订阅",
      dataIndex: "id",
      key: "id",
      render: (value: string) => shortId(value)
    },
    {
      title: "接入配置",
      dataIndex: "app_id",
      key: "app_id",
      render: (value: string) => shortId(value)
    },
    {
      title: "用户",
      dataIndex: "customer_id",
      key: "customer_id",
      render: (value: string) => shortId(value)
    },
    {
      title: "套餐",
      dataIndex: "plan",
      key: "plan",
      width: 120,
      render: (value) => <StatusTag value={value} />
    },
    {
      title: "状态",
      dataIndex: "status",
      key: "status",
      width: 120,
      render: (_, record) => (
        <StatusTag value={effectiveTemporalStatus(record)} />
      )
    },
    {
      title: "席位上限",
      dataIndex: "max_devices",
      key: "max_devices",
      width: 110
    },
    {
      title: "开始时间",
      dataIndex: "starts_at",
      key: "starts_at",
      render: (value: string) => dateTime(value)
    },
    {
      title: "到期时间",
      dataIndex: "expires_at",
      key: "expires_at",
      render: (value?: string | null) => dateTime(value)
    },
    {
      title: "操作",
      key: "actions",
      width: 420,
      render: (_, record) => {
        const effectiveStatus = effectiveTemporalStatus(record);

        return (
          <Space wrap>
            {canSuspend && effectiveStatus === "active" ? (
              <ConfirmActionButton
                title="暂停订阅"
                description="暂停后用户仍可登录，但需要订阅的功能会立即不可用，并撤销相关活跃会话。"
                buttonProps={{
                  size: "small",
                  icon: <Pause size={14} />
                }}
                loading={suspendMutation.isPending}
                onConfirm={() => suspendMutation.mutate(record.id)}
              >
                暂停
              </ConfirmActionButton>
            ) : null}
            {canResume && record.status === "suspended" ? (
              <ConfirmActionButton
                title="恢复订阅"
                description="恢复后订阅重新生效；如果已到期，需要先续期。"
                buttonProps={{
                  size: "small",
                  icon: <Play size={14} />
                }}
                loading={resumeMutation.isPending}
                onConfirm={() => resumeMutation.mutate(record.id)}
              >
                恢复
              </ConfirmActionButton>
            ) : null}
            {canRenew && record.status !== "cancelled" ? (
              <Button
                size="small"
                icon={<CalendarPlus size={14} />}
                onClick={() => setRenewTarget(record)}
              >
                续期
              </Button>
            ) : null}
            {canResetDevice && record.status !== "cancelled" ? (
              <Button
                size="small"
                icon={<RotateCcw size={14} />}
                onClick={() => setResetDevicesTarget(record)}
              >
                重置绑定
              </Button>
            ) : null}
            {canCancel && record.status !== "cancelled" ? (
              <ConfirmActionButton
                title="取消订阅"
                description="取消后订阅不再继续生效，并会撤销相关活跃会话。"
                buttonProps={{
                  size: "small",
                  icon: <Ban size={14} />
                }}
                loading={cancelMutation.isPending}
                onConfirm={() => cancelMutation.mutate(record.id)}
              >
                取消
              </ConfirmActionButton>
            ) : null}
          </Space>
        );
      }
    }
  ];

  const submitCreate = (values: CreateSubscriptionFormValues) => {
    const payload: CreateSubscriptionPayload = {
      app_id: values.app_id,
      customer_id: values.customer_id,
      plan: values.plan,
      max_devices: cleanNumber(values.max_devices),
      starts_at: values.starts_at?.toISOString(),
      expires_at: values.expires_at?.toISOString(),
      features: cleanFeatures(values.features)
    };
    createMutation.mutate(payload);
  };

  const submitRenew = (values: RenewSubscriptionFormValues) => {
    if (!renewTarget || !values.expires_at) {
      return;
    }

    renewMutation.mutate({
      id: renewTarget.id,
      expires_at: values.expires_at.toISOString()
    });
  };

  const submitResetDevices = (values: ResetDevicesFormValues) => {
    const reason = clean(values.reason);
    if (!resetDevicesTarget || !reason) {
      return;
    }

    resetDevicesMutation.mutate({
      id: resetDevicesTarget.id,
      reason
    });
  };

  return (
    <section className="workspace-page">
      <div className="page-heading">
        <div>
          <Typography.Title level={2}>套餐订阅</Typography.Title>
          <Typography.Text type="secondary">
            用户套餐、有效期、席位上限和功能开关
          </Typography.Text>
        </div>
        <Space>
          <Input.Search
            allowClear
            placeholder="关键词"
            onSearch={(value) => {
              setKeyword(value);
            }}
            className="table-search"
          />
          <Select
            allowClear
            placeholder="状态"
            className="table-filter"
            value={status}
            options={[
              tOption("active"),
              tOption("trialing"),
              tOption("past_due"),
              tOption("suspended"),
              tOption("cancelled"),
              tOption("expired")
            ]}
            onChange={(value) => {
              setStatus(value);
            }}
          />
          <HistoryToggle
            checked={includeHistory}
            onChange={(checked) => {
              setIncludeHistory(checked);
            }}
          />
          <Button
            icon={<RefreshCw size={16} />}
            onClick={() => void refetchFirstSubscriptionPage()}
          />
          {canCreate ? (
            <Button
              type="primary"
              icon={<Plus size={16} />}
              onClick={() => setCreateOpen(true)}
            >
              创建订阅
            </Button>
          ) : null}
        </Space>
      </div>
      {query.error ? (
        <Alert type="error" message={tMessage("subscriptions_load_failed")} />
      ) : null}
      {createMutation.error ? (
        <Alert type="error" message={tMessage("subscription_create_failed")} />
      ) : null}
      {cancelMutation.error ? (
        <Alert type="error" message={tMessage("subscription_cancel_failed")} />
      ) : null}
      {suspendMutation.error ||
      resumeMutation.error ||
      renewMutation.error ||
      resetDevicesMutation.error ? (
        <Alert type="error" message={tMessage("subscription_status_update_failed")} />
      ) : null}
      <InfiniteTable<SubscriptionSummary>
        rowKey="id"
        loading={query.isLoading}
        columns={columns}
        items={items}
        hasMore={Boolean(query.hasNextPage)}
        isFetchingNextPage={query.isFetchingNextPage}
        itemLabel="订阅"
        onLoadMore={() => query.fetchNextPage()}
        scroll={{ x: "max-content" }}
        locale={{ emptyText: "暂无数据" }}
      />

      <Modal
        title="创建订阅"
        open={createOpen}
        onCancel={() => setCreateOpen(false)}
        onOk={() => form.submit()}
        confirmLoading={createMutation.isPending}
        destroyOnClose
      >
        <Form<CreateSubscriptionFormValues>
          form={form}
          layout="vertical"
          onFinish={submitCreate}
          initialValues={{
            plan: productSubscriptionConfig.defaultPlanCode,
            max_devices: productSubscriptionConfig.defaultSeatLimit,
            features: productSubscriptionConfig.defaultFeatures
          }}
        >
          <Form.Item
            name="app_id"
            label="接入配置"
            rules={[{ required: true, message: "请选择接入配置" }]}
          >
            <Select
              showSearch
              optionFilterProp="label"
              loading={appsQuery.isLoading}
              options={(appsQuery.data?.items ?? []).map((app) => ({
                value: app.id,
                label: `${app.name} · ${shortId(app.id)}`
              }))}
            />
          </Form.Item>
          <Form.Item
            name="customer_id"
            label="用户"
            rules={[{ required: true, message: "请选择用户" }]}
          >
            <Select
              showSearch
              optionFilterProp="label"
              loading={customersQuery.isLoading}
              options={(customersQuery.data?.items ?? []).map((customer) => ({
                value: customer.id,
                label: `${customer.name ?? customer.email} · ${customer.email}`
              }))}
            />
          </Form.Item>
          <Form.Item
            name="plan"
            label="套餐"
            rules={[{ required: true, message: "请输入套餐" }]}
          >
            <AutoComplete
              options={planOptions}
              placeholder={productSubscriptionConfig.defaultPlanCode}
              onSelect={applyPlanDefaults}
              filterOption={(inputValue, option) =>
                String(option?.label ?? option?.value ?? "")
                  .toLowerCase()
                  .includes(inputValue.toLowerCase())
              }
            />
          </Form.Item>
          <Form.Item name="max_devices" label="席位上限">
            <InputNumber min={0} precision={0} className="form-number" />
          </Form.Item>
          <Form.Item name="starts_at" label="开始时间">
            <DatePicker showTime className="form-date" />
          </Form.Item>
          <Form.Item name="expires_at" label="到期时间">
            <DatePicker showTime className="form-date" />
          </Form.Item>
          <Form.Item name="features" label="功能标记">
            <Select
              mode="tags"
              tokenSeparators={[","]}
              options={productSubscriptionConfig.featureOptions}
            />
          </Form.Item>
        </Form>
      </Modal>

      <Modal
        title="续期订阅"
        open={Boolean(renewTarget)}
        onCancel={() => {
          setRenewTarget(null);
          renewForm.resetFields();
        }}
        onOk={() => renewForm.submit()}
        confirmLoading={renewMutation.isPending}
        destroyOnClose
      >
        <Form<RenewSubscriptionFormValues>
          form={renewForm}
          layout="vertical"
          onFinish={submitRenew}
        >
          <Form.Item label="订阅">
            <Typography.Text>{shortId(renewTarget?.id)}</Typography.Text>
          </Form.Item>
          <Form.Item
            name="expires_at"
            label="新的到期时间"
            rules={[{ required: true, message: "请选择新的到期时间" }]}
          >
            <DatePicker showTime className="form-date" />
          </Form.Item>
        </Form>
      </Modal>

      <Modal
        title="重置订阅绑定"
        open={Boolean(resetDevicesTarget)}
        onCancel={() => {
          setResetDevicesTarget(null);
          resetDevicesForm.resetFields();
        }}
        onOk={() => resetDevicesForm.submit()}
        confirmLoading={resetDevicesMutation.isPending}
        destroyOnClose
      >
        <Form<ResetDevicesFormValues>
          form={resetDevicesForm}
          layout="vertical"
          onFinish={submitResetDevices}
        >
          <Form.Item label="订阅">
            <Typography.Text>{shortId(resetDevicesTarget?.id)}</Typography.Text>
          </Form.Item>
          <Form.Item
            name="reason"
            label="原因"
            rules={[{ required: true, message: "请输入重置原因" }]}
          >
            <Input.TextArea rows={3} />
          </Form.Item>
        </Form>
      </Modal>
    </section>
  );
}

function cleanNumber(value?: number | null): number | undefined {
  return typeof value === "number" ? value : undefined;
}

function cleanFeatures(values?: string[]): string[] | undefined {
  const features = values
    ?.map((value) => value.trim())
    .filter((value) => value.length > 0);

  return features && features.length > 0 ? features : undefined;
}

function clean(value?: string): string | undefined {
  const trimmed = value?.trim();

  return trimmed ? trimmed : undefined;
}
