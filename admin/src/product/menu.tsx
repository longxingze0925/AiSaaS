import { Boxes, ClipboardList } from "lucide-react";

import type { MenuRoute } from "../routes/menu";

// Add project-specific menu items here instead of editing the core SaaS menu.
export const productMenuRoutes: MenuRoute[] = [
  {
    key: "product",
    path: "/product/setup",
    label: "产品配置",
    permission: "system:read",
    icon: <Boxes size={18} />,
    children: [
      {
        key: "product-setup",
        path: "/product/setup",
        label: "初始化蓝图",
        permission: "system:read",
        icon: <ClipboardList size={18} />
      }
    ]
  }
];
