import {
  BellRing,
  Boxes,
  ClipboardList,
  CreditCard,
  Cpu,
  FileClock,
  Gauge,
  Inbox,
  KeyRound,
  ListTree,
  Settings,
  Shield,
  ShieldCheck,
  Users
} from "lucide-react";
import type { ReactNode } from "react";

import { productMenuRoutes } from "../product/menu";

export interface MenuRoute {
  key: string;
  path: string;
  label: string;
  permission?: string;
  icon?: ReactNode;
  children?: MenuRoute[];
}

export const menuRoutes: MenuRoute[] = [
  {
    key: "dashboard",
    path: "/",
    label: "仪表盘",
    icon: <Gauge size={18} />
  },
  {
    key: "users-billing",
    path: "/customers",
    label: "用户与计费",
    icon: <Users size={18} />,
    children: [
      {
        key: "customers",
        path: "/customers",
        label: "用户管理",
        permission: "customer:read",
        icon: <Users size={18} />
      },
      {
        key: "subscriptions",
        path: "/subscriptions",
        label: "套餐订阅",
        permission: "subscription:read",
        icon: <CreditCard size={18} />
      },
      {
        key: "ai-billing-wallets",
        path: "/ai-billing/wallets",
        label: "余额账户",
        permission: "ai:read",
        icon: <CreditCard size={18} />
      },
      {
        key: "logs-billing-ledger",
        path: "/logs/billing-ledger",
        label: "计费流水",
        permission: "ai:read",
        icon: <FileClock size={18} />
      }
    ]
  },
  {
    key: "ai-billing",
    path: "/ai-billing/providers",
    label: "AI 能力",
    permission: "ai:read",
    icon: <Cpu size={18} />,
    children: [
      {
        key: "ai-billing-providers",
        path: "/ai-billing/providers",
        label: "渠道配置",
        permission: "ai:read",
        icon: <Cpu size={18} />
      },
      {
        key: "ai-billing-models",
        path: "/ai-billing/models",
        label: "模型商品",
        permission: "ai:read",
        icon: <ClipboardList size={18} />
      },
      {
        key: "logs-ai-jobs",
        path: "/logs/ai-jobs",
        label: "生成任务",
        permission: "ai:job:read",
        icon: <ClipboardList size={18} />
      },
      {
        key: "logs-ai-usage",
        path: "/logs/ai-usage",
        label: "调用日志",
        permission: "ai:read",
        icon: <FileClock size={18} />
      },
      {
        key: "logs-ai-assets",
        path: "/logs/ai-assets",
        label: "素材缓存",
        permission: "ai:read",
        icon: <Boxes size={18} />
      },
      {
        key: "api-access",
        path: "/api-access",
        label: "API 接入",
        permission: "app:read",
        icon: <KeyRound size={18} />
      }
    ]
  },
  ...productMenuRoutes,
  {
    key: "operations",
    path: "/tasks",
    label: "运营系统",
    icon: <ListTree size={18} />,
    children: [
      {
        key: "tasks-center",
        path: "/tasks",
        label: "任务中心",
        permission: "security:view_events",
        icon: <Inbox size={18} />
      },
      {
        key: "logs-audit",
        path: "/logs/audit",
        label: "审计日志",
        permission: "audit:read",
        icon: <FileClock size={18} />
      },
      {
        key: "team",
        path: "/team",
        label: "团队成员",
        permission: "member:read",
        icon: <Users size={18} />
      },
      {
        key: "roles",
        path: "/roles",
        label: "角色权限",
        permission: "role:read",
        icon: <Shield size={18} />
      },
      {
        key: "system-settings",
        path: "/system-settings",
        label: "系统配置",
        permission: "system:read",
        icon: <Settings size={18} />
      },
      {
        key: "notification-channels",
        path: "/notification-channels",
        label: "通知渠道",
        permission: "notification:read",
        icon: <BellRing size={18} />
      },
      {
        key: "security",
        path: "/security",
        label: "安全状态",
        icon: <ShieldCheck size={18} />
      }
    ]
  }
];

export const flatMenuRoutes = flattenMenuRoutes(menuRoutes);

function flattenMenuRoutes(routes: MenuRoute[]): MenuRoute[] {
  return routes.flatMap((route) => [
    route,
    ...(route.children ? flattenMenuRoutes(route.children) : [])
  ]);
}
