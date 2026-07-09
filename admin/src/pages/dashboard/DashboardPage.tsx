import { Col, Row, Table, Tag, Typography } from "antd";
import { Activity, FileClock, ShieldCheck, Users } from "lucide-react";

import { productConfig } from "../../product/config";
import { useAuthStore } from "../../stores/authStore";
import { tPermissionLabel, tRoleLabel } from "../../utils/i18n";

const metrics = [
  {
    label: "运营实例",
    value: "1",
    icon: <ShieldCheck size={20} />
  },
  {
    label: "权限点",
    value: "permissions",
    icon: <Activity size={20} />
  },
  {
    label: "后台角色",
    value: "roles",
    icon: <Users size={20} />
  },
  {
    label: "审计状态",
    value: "enabled",
    icon: <FileClock size={20} />
  }
];

export function DashboardPage() {
  const { user, tenant, roles, permissions } = useAuthStore();
  const roleLabels = roles.map((code) => tRoleLabel({ code }, { includeCode: true }));
  const rows = [
    {
      key: "user",
      item: "当前管理员",
      value: user ? `${user.name} / ${user.email}` : "-"
    },
    {
      key: "tenant",
      item: "运营空间",
      value: tenant?.name ?? "-"
    },
    {
      key: "roles",
      item: "后台角色",
      value: roleLabels.length > 0 ? roleLabels.join(", ") : "-"
    }
  ];

  return (
    <section className="workspace-page">
      <div className="page-heading">
        <div>
          <Typography.Title level={2}>SaaS 运营仪表盘</Typography.Title>
          <Typography.Text type="secondary">
            {productConfig.dashboardSubtitle}
          </Typography.Text>
        </div>
      </div>

      <Row gutter={[12, 12]} className="metric-grid">
        {metrics.map((metric) => {
          const value =
            metric.value === "permissions"
              ? permissions.length
              : metric.value === "roles"
                ? roles.length
                : metric.value === "enabled"
                  ? "已启用"
                  : metric.value;

          return (
            <Col xs={24} sm={12} lg={6} key={metric.label}>
              <div className="metric-tile">
                <span className="metric-icon">{metric.icon}</span>
                <span className="metric-label">{metric.label}</span>
                <strong>{value}</strong>
              </div>
            </Col>
          );
        })}
      </Row>

      <Table
        rowKey="key"
        columns={[
          { title: "项目", dataIndex: "item", key: "item", width: 180 },
          { title: "值", dataIndex: "value", key: "value" }
        ]}
        dataSource={rows}
        pagination={false}
      />

      <div className="permission-strip">
        {permissions.slice(0, 18).map((permission) => (
          <Tag key={permission}>
            {tPermissionLabel({ code: permission }, { includeCode: true })}
          </Tag>
        ))}
      </div>
    </section>
  );
}
