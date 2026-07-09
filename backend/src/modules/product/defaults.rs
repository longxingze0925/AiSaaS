//! Product-specific backend defaults.
//!
//! This module is intentionally data-only. It gives concrete SaaS products a
//! stable place for default access, subscription, and AI billing policy without
//! changing shared core modules or database migrations.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ProductAccessDefaults {
    pub application_name: &'static str,
    pub application_slug: &'static str,
    pub auth_mode: &'static str,
    pub heartbeat_interval_seconds: i32,
    pub offline_tolerance_seconds: i32,
    pub max_devices_default: i32,
    pub server_api_key_name: &'static str,
    pub server_api_scopes: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ProductSubscriptionPlan {
    pub code: &'static str,
    pub max_devices: i32,
    pub features: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ProductWalletAdjustmentDefaults {
    pub credit_reason: &'static str,
    pub debit_reason: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ProductAiJobActionReasons {
    pub retry_poll: &'static str,
    pub retry_cache: &'static str,
    pub fail_release: &'static str,
    pub refund: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ProductAiBillingDefaults {
    pub currency: &'static str,
    pub wallet_adjustment: ProductWalletAdjustmentDefaults,
    pub job_action_reasons: ProductAiJobActionReasons,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ProductDefaults {
    pub access: ProductAccessDefaults,
    pub default_subscription_plan: &'static str,
    pub subscription_plans: &'static [ProductSubscriptionPlan],
    pub ai_billing: ProductAiBillingDefaults,
}

pub const AI_INVOKE_SCOPE: &str = "ai:invoke";
pub const IMAGE_GENERATE_FEATURE: &str = "image:generate";
pub const VIDEO_GENERATE_FEATURE: &str = "video:generate";
pub const AUDIO_GENERATE_FEATURE: &str = "audio:generate";
pub const ASSET_LIBRARY_FEATURE: &str = "asset:library";
pub const ASSET_CACHE_FEATURE: &str = "asset:cache";
pub const WORKSPACE_TEAM_FEATURE: &str = "workspace:team";
pub const WORKFLOW_BATCH_FEATURE: &str = "workflow:batch";

pub const PRODUCT_SUBSCRIPTION_PLANS: &[ProductSubscriptionPlan] = &[
    ProductSubscriptionPlan {
        code: "creator",
        max_devices: 1,
        features: &[
            AI_INVOKE_SCOPE,
            IMAGE_GENERATE_FEATURE,
            AUDIO_GENERATE_FEATURE,
        ],
    },
    ProductSubscriptionPlan {
        code: "studio",
        max_devices: 1,
        features: &[
            AI_INVOKE_SCOPE,
            IMAGE_GENERATE_FEATURE,
            VIDEO_GENERATE_FEATURE,
            AUDIO_GENERATE_FEATURE,
            ASSET_LIBRARY_FEATURE,
            ASSET_CACHE_FEATURE,
        ],
    },
    ProductSubscriptionPlan {
        code: "team_studio",
        max_devices: 8,
        features: &[
            AI_INVOKE_SCOPE,
            IMAGE_GENERATE_FEATURE,
            VIDEO_GENERATE_FEATURE,
            AUDIO_GENERATE_FEATURE,
            ASSET_LIBRARY_FEATURE,
            ASSET_CACHE_FEATURE,
            WORKSPACE_TEAM_FEATURE,
            WORKFLOW_BATCH_FEATURE,
        ],
    },
];

pub const PRODUCT_DEFAULTS: ProductDefaults = ProductDefaults {
    access: ProductAccessDefaults {
        application_name: "灵感影像工坊 API 接入",
        application_slug: "media-studio-default",
        auth_mode: "subscription",
        heartbeat_interval_seconds: 3600,
        offline_tolerance_seconds: 86400,
        max_devices_default: 1,
        server_api_key_name: "影像生成生产 Key",
        server_api_scopes: &[AI_INVOKE_SCOPE],
    },
    default_subscription_plan: "studio",
    subscription_plans: PRODUCT_SUBSCRIPTION_PLANS,
    ai_billing: ProductAiBillingDefaults {
        currency: "CNY",
        wallet_adjustment: ProductWalletAdjustmentDefaults {
            credit_reason: "影像生成余额充值",
            debit_reason: "影像生成余额扣减",
        },
        job_action_reasons: ProductAiJobActionReasons {
            retry_poll: "重新查询生成任务状态",
            retry_cache: "重新缓存生成素材",
            fail_release: "生成失败释放预扣",
            refund: "生成异常人工退款",
        },
    },
};

pub fn subscription_plan(code: &str) -> Option<ProductSubscriptionPlan> {
    PRODUCT_DEFAULTS
        .subscription_plans
        .iter()
        .copied()
        .find(|plan| plan.code == code)
}

#[cfg(test)]
mod tests {
    use super::{subscription_plan, AI_INVOKE_SCOPE, PRODUCT_DEFAULTS};

    #[test]
    fn default_subscription_plan_exists_and_enables_ai() {
        let plan =
            subscription_plan(PRODUCT_DEFAULTS.default_subscription_plan).expect("default plan");

        assert!(plan.features.contains(&AI_INVOKE_SCOPE));
        assert!(plan.max_devices >= 1);
    }

    #[test]
    fn access_defaults_are_saas_oriented() {
        assert_eq!(
            PRODUCT_DEFAULTS.access.application_slug,
            "media-studio-default"
        );
        assert_eq!(PRODUCT_DEFAULTS.access.auth_mode, "subscription");
        assert!(PRODUCT_DEFAULTS
            .access
            .server_api_scopes
            .contains(&AI_INVOKE_SCOPE));
    }
}
