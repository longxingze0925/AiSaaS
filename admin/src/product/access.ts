export interface ProductServerApiScopeOption {
  label: string;
  value: string;
}

interface ProductAccessConfig {
  applicationDefaults: {
    name: string;
    slug: string;
    authMode: "both" | "license" | "subscription";
    heartbeatIntervalSeconds: number;
    offlineToleranceSeconds: number;
    seatLimit: number;
  };
  serverApiKeyDefaults: {
    name: string;
    scopes: string[];
  };
  serverApiScopeOptions: ProductServerApiScopeOption[];
}

export const productAccessConfig: ProductAccessConfig = {
  applicationDefaults: {
    name: "Ai SaaS API 接入",
    slug: "media-studio-default",
    authMode: "subscription",
    heartbeatIntervalSeconds: 3600,
    offlineToleranceSeconds: 86400,
    seatLimit: 1
  },
  serverApiKeyDefaults: {
    name: "影像生成生产 Key",
    scopes: ["ai:invoke"]
  },
  serverApiScopeOptions: [{ label: "调用生成 API", value: "ai:invoke" }]
};
