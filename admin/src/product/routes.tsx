import type { ReactNode } from "react";

import { ProductSetupPage } from "./ProductSetupPage";

export interface ProductRoute {
  path: string;
  element: ReactNode;
}

// Add concrete SaaS product pages here instead of editing the core app routes.
export const productRoutes: ProductRoute[] = [
  {
    path: "product/setup",
    element: <ProductSetupPage />
  }
];
