export interface ProductSubscriptionPlan {
  code: string;
  label: string;
  defaultSeatLimit: number;
  features: string[];
}

export interface ProductFeatureOption {
  label: string;
  value: string;
}

interface ProductSubscriptionConfig {
  defaultPlanCode: string;
  defaultSeatLimit: number;
  defaultFeatures: string[];
  plans: ProductSubscriptionPlan[];
  featureOptions: ProductFeatureOption[];
}

export const productSubscriptionConfig: ProductSubscriptionConfig = {
  defaultPlanCode: "studio",
  defaultSeatLimit: 1,
  defaultFeatures: [
    "ai:invoke",
    "image:generate",
    "video:generate",
    "audio:generate",
    "asset:library"
  ],
  plans: [
    {
      code: "creator",
      label: "创作者版",
      defaultSeatLimit: 1,
      features: ["ai:invoke", "image:generate", "audio:generate"]
    },
    {
      code: "studio",
      label: "工作室版",
      defaultSeatLimit: 1,
      features: [
        "ai:invoke",
        "image:generate",
        "video:generate",
        "audio:generate",
        "asset:library",
        "asset:cache"
      ]
    },
    {
      code: "team_studio",
      label: "团队版",
      defaultSeatLimit: 8,
      features: [
        "ai:invoke",
        "image:generate",
        "video:generate",
        "audio:generate",
        "asset:library",
        "asset:cache",
        "workspace:team",
        "workflow:batch"
      ]
    }
  ],
  featureOptions: [
    { label: "生成 API 调用", value: "ai:invoke" },
    { label: "图片生成", value: "image:generate" },
    { label: "视频生成", value: "video:generate" },
    { label: "音频生成", value: "audio:generate" },
    { label: "素材库", value: "asset:library" },
    { label: "素材缓存", value: "asset:cache" },
    { label: "团队空间", value: "workspace:team" },
    { label: "批量工作流", value: "workflow:batch" }
  ]
};
