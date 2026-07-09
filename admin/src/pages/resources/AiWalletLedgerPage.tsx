import { Alert, Button, Input, Select, Space, Tag, Typography } from "antd";
import type { ColumnsType } from "antd/es/table";
import { RefreshCw, Search, X } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { useInfiniteQuery, useQueryClient } from "@tanstack/react-query";

import {
  listAiWalletLedgerEntries,
  type AiWalletLedgerEntry
} from "../../api/admin";
import { BillingSettlementSummary } from "../../components/BillingSettlementSummary";
import { InfiniteTable } from "../../components/InfiniteTable";
import { dateTime, shortId } from "../../utils/format";
import {
  INFINITE_TABLE_QUERY_OPTIONS,
  trimInfiniteQueryCache
} from "../../utils/infiniteQueryCache";
import { tMessage } from "../../utils/i18n";

const pageSize = 30;

const ledgerTypeOptions = [
  { label: "全部类型", value: "" },
  { label: "充值", value: "credit" },
  { label: "扣减", value: "debit" },
  { label: "预扣", value: "hold" },
  { label: "结算", value: "capture" },
  { label: "释放", value: "release" },
  { label: "退款", value: "refund" },
  { label: "调整", value: "adjustment" }
];

export function AiWalletLedgerPage() {
  const [draftCustomerId, setDraftCustomerId] = useState("");
  const [draftReferenceId, setDraftReferenceId] = useState("");
  const [draftEntryType, setDraftEntryType] = useState("");
  const [customerId, setCustomerId] = useState("");
  const [referenceId, setReferenceId] = useState("");
  const [entryType, setEntryType] = useState("");
  const queryClient = useQueryClient();

  const query = useInfiniteQuery({
    ...INFINITE_TABLE_QUERY_OPTIONS,
    queryKey: ["admin", "ai-wallet-ledger", customerId, entryType, referenceId],
    queryFn: ({ pageParam }) =>
      listAiWalletLedgerEntries({
        customer_id: customerId,
        entry_type: entryType,
        reference_id: referenceId,
        page: Number(pageParam),
        page_size: pageSize
      }),
    initialPageParam: 1,
    getNextPageParam: (lastPage) =>
      lastPage.meta?.has_more ? lastPage.meta.page + 1 : undefined
  });

  useEffect(() => {
    return () => {
      trimInfiniteQueryCache(queryClient, ["admin", "ai-wallet-ledger"]);
    };
  }, [queryClient]);

  const refreshLedger = () => {
    trimInfiniteQueryCache(queryClient, ["admin", "ai-wallet-ledger"]);
    query.refetch();
  };

  const applyFilters = () => {
    setCustomerId(draftCustomerId.trim());
    setReferenceId(draftReferenceId.trim());
    setEntryType(draftEntryType);
  };

  const resetFilters = () => {
    setDraftCustomerId("");
    setDraftReferenceId("");
    setDraftEntryType("");
    setCustomerId("");
    setReferenceId("");
    setEntryType("");
  };

  const items = useMemo(
    () => (query.data?.pages ?? []).flatMap((page) => page.items),
    [query.data]
  );

  const columns: ColumnsType<AiWalletLedgerEntry> = [
    {
      title: "用户",
      dataIndex: "customer_email",
      key: "customer_email",
      width: 320,
      render: (value: string | null | undefined, record) => (
        <Space className="ai-stacked-cell" direction="vertical" size={0}>
          <Typography.Text ellipsis title={record.customer_name || value || "-"}>
            {record.customer_name || value || "-"}
          </Typography.Text>
          <Typography.Text ellipsis title={value || record.customer_id} type="secondary">
            {value || shortId(record.customer_id)}
          </Typography.Text>
        </Space>
      )
    },
    {
      title: "类型",
      dataIndex: "entry_type",
      key: "entry_type",
      width: 100,
      render: (value: string) => <Tag>{ledgerTypeLabel(value)}</Tag>
    },
    {
      title: "金额",
      dataIndex: "amount_minor",
      key: "amount_minor",
      width: 130,
      render: (value: number, record) => (
        <Typography.Text type={value < 0 ? "danger" : "success"}>
          {money(value, record.currency)}
        </Typography.Text>
      )
    },
    {
      title: "余额",
      dataIndex: "balance_after_minor",
      key: "balance_after_minor",
      width: 130,
      render: (value: number, record) => money(value, record.currency)
    },
    {
      title: "冻结",
      dataIndex: "held_after_minor",
      key: "held_after_minor",
      width: 130,
      render: (value: number, record) => money(value, record.currency)
    },
    {
      title: "结算",
      dataIndex: "metadata",
      key: "settlement",
      width: 210,
      render: (value: Record<string, unknown>, record) => (
        <BillingSettlementSummary metadata={value} currency={record.currency} />
      )
    },
    {
      title: "原因",
      dataIndex: "reason",
      key: "reason",
      width: 260,
      render: (value: string) => (
        <Typography.Text ellipsis title={value}>
          {value}
        </Typography.Text>
      )
    },
    {
      title: "引用",
      key: "reference",
      width: 220,
      render: (_, record) => (
        <Space direction="vertical" size={0}>
          <Typography.Text>{record.reference_type ?? "-"}</Typography.Text>
          <Typography.Text type="secondary">{shortId(record.reference_id)}</Typography.Text>
        </Space>
      )
    },
    {
      title: "时间",
      dataIndex: "created_at",
      key: "created_at",
      width: 180,
      render: (value: string) => dateTime(value)
    }
  ];

  return (
    <section className="workspace-page">
      <div className="page-heading">
        <div>
          <Typography.Title level={2}>计费流水</Typography.Title>
          <Typography.Text type="secondary">AI 余额充值、预扣、结算、退款和后台调整记录</Typography.Text>
        </div>
        <Space className="page-heading-actions">
          <Input
            allowClear
            placeholder="用户 ID"
            value={draftCustomerId}
            onChange={(event) => setDraftCustomerId(event.target.value)}
            onPressEnter={applyFilters}
            className="audit-filter-input"
          />
          <Select
            options={ledgerTypeOptions}
            value={draftEntryType}
            onChange={setDraftEntryType}
            className="audit-filter-input"
          />
          <Input
            allowClear
            placeholder="引用 ID"
            value={draftReferenceId}
            onChange={(event) => setDraftReferenceId(event.target.value)}
            onPressEnter={applyFilters}
            className="audit-filter-input"
          />
          <Button type="primary" icon={<Search size={16} />} onClick={applyFilters}>
            查询
          </Button>
          <Button icon={<X size={16} />} onClick={resetFilters}>
            清空
          </Button>
          <Button icon={<RefreshCw size={16} />} onClick={refreshLedger} />
        </Space>
      </div>

      {query.error ? <Alert type="error" message={tMessage("ai_wallet_ledger_load_failed")} /> : null}

      <InfiniteTable<AiWalletLedgerEntry>
        rowKey="id"
        loading={query.isLoading}
        columns={columns}
        items={items}
        hasMore={Boolean(query.hasNextPage)}
        isFetchingNextPage={query.isFetchingNextPage}
        itemLabel="计费流水"
        onLoadMore={() => query.fetchNextPage()}
        scroll={{ x: "max-content" }}
        locale={{ emptyText: "暂无数据" }}
      />
    </section>
  );
}

function ledgerTypeLabel(value: string): string {
  const labels: Record<string, string> = {
    credit: "充值",
    debit: "扣减",
    hold: "预扣",
    capture: "结算",
    release: "释放",
    refund: "退款",
    adjustment: "调整"
  };

  return labels[value] ?? value;
}

function money(value: number, currency: string): string {
  const sign = value < 0 ? "-" : "";
  const amount = Math.abs(value) / 100;

  return `${sign}${currency} ${amount.toFixed(2)}`;
}
