import { Button, Popover, Space, Typography } from "antd";

interface BillingSettlement {
  requested_minor: number;
  captured_minor: number;
  released_minor: number;
  additional_minor: number;
  shortfall_minor: number;
}

interface BillingSettlementSummaryProps {
  metadata?: Record<string, unknown> | null;
  currency: string;
}

export function BillingSettlementSummary({
  metadata,
  currency
}: BillingSettlementSummaryProps) {
  const settlement = billingSettlementFromMetadata(metadata);

  if (!settlement) {
    return <Typography.Text type="secondary">-</Typography.Text>;
  }

  const detailItems = [
    ["应扣", settlement.requested_minor],
    ["实扣", settlement.captured_minor],
    ["释放", settlement.released_minor],
    ["补扣", settlement.additional_minor],
    ["缺口", settlement.shortfall_minor]
  ] as const;

  return (
    <Space direction="vertical" size={0}>
      <Typography.Text>{money(settlement.captured_minor, currency)}</Typography.Text>
      <Space size={4}>
        <Typography.Text type="secondary">
          应扣 {money(settlement.requested_minor, currency)}
        </Typography.Text>
        <Popover
          title="结算明细"
          trigger="click"
          content={
            <Space direction="vertical" size={4}>
              {detailItems.map(([label, value]) => (
                <Typography.Text
                  key={label}
                  type={label === "缺口" && value > 0 ? "danger" : "secondary"}
                >
                  {label} {money(value, currency)}
                </Typography.Text>
              ))}
            </Space>
          }
        >
          <Button type="link" size="small">
            明细
          </Button>
        </Popover>
      </Space>
    </Space>
  );
}

function billingSettlementFromMetadata(
  metadata?: Record<string, unknown> | null
): BillingSettlement | undefined {
  const root = objectValue(metadata);
  const settlement = objectValue(root?.billing_settlement) ?? root;
  const captured = numberValue(settlement?.captured_minor);
  const requested = numberValue(settlement?.requested_minor);

  if (captured == null && requested == null) {
    return undefined;
  }

  return {
    requested_minor: requested ?? 0,
    captured_minor: captured ?? 0,
    released_minor: numberValue(settlement?.released_minor) ?? 0,
    additional_minor: numberValue(settlement?.additional_minor) ?? 0,
    shortfall_minor: numberValue(settlement?.shortfall_minor) ?? 0
  };
}

function objectValue(value: unknown): Record<string, unknown> | undefined {
  return value && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : undefined;
}

function numberValue(value: unknown): number | undefined {
  const number = typeof value === "number" ? value : Number(value);

  return Number.isFinite(number) ? number : undefined;
}

function money(value: number, currency: string): string {
  const sign = value < 0 ? "-" : "";
  const amount = Math.abs(value) / 100;

  return `${sign}${currency} ${amount.toFixed(2)}`;
}
