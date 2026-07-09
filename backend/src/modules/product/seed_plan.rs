//! Dry-run product initialization plan.
//!
//! This is the backend contract for future one-click initialization. It is
//! intentionally side-effect free: callers can serialize and inspect the plan,
//! but no database records are created from this module.

use serde::Serialize;

use super::defaults::{
    ProductAccessDefaults, ProductAiBillingDefaults, ProductSubscriptionPlan, PRODUCT_DEFAULTS,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProductSeedPlan {
    pub version: u32,
    pub destructive: bool,
    pub idempotency_scope: &'static str,
    pub steps: Vec<ProductSeedStep>,
    pub defaults: ProductSeedDefaults,
    pub warnings: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProductSeedStep {
    pub key: &'static str,
    pub target: &'static str,
    pub source: &'static str,
    pub mode: ProductSeedStepMode,
    pub idempotency_key: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductSeedStepMode {
    PreviewOnly,
    CreateIfMissing,
    ReconcileManually,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProductSeedDefaults {
    pub access: ProductAccessDefaults,
    pub default_subscription_plan: &'static str,
    pub subscription_plans: &'static [ProductSubscriptionPlan],
    pub ai_billing: ProductAiBillingDefaults,
}

pub fn build_product_seed_plan() -> ProductSeedPlan {
    ProductSeedPlan {
        version: 1,
        destructive: false,
        idempotency_scope: "tenant",
        steps: vec![
            ProductSeedStep {
                key: "access.default_application",
                target: "default application access config",
                source: "product::defaults::PRODUCT_DEFAULTS.access",
                mode: ProductSeedStepMode::CreateIfMissing,
                idempotency_key: "product:access:default_application",
            },
            ProductSeedStep {
                key: "access.server_api_key",
                target: "server api key scopes",
                source: "product::defaults::PRODUCT_DEFAULTS.access.server_api_scopes",
                mode: ProductSeedStepMode::PreviewOnly,
                idempotency_key: "product:access:server_api_key",
            },
            ProductSeedStep {
                key: "subscription.plans",
                target: "subscription plan presets",
                source: "product::defaults::PRODUCT_DEFAULTS.subscription_plans",
                mode: ProductSeedStepMode::ReconcileManually,
                idempotency_key: "product:subscription:plans",
            },
            ProductSeedStep {
                key: "ai.billing",
                target: "ai wallet and job operation defaults",
                source: "product::defaults::PRODUCT_DEFAULTS.ai_billing",
                mode: ProductSeedStepMode::PreviewOnly,
                idempotency_key: "product:ai:billing_defaults",
            },
        ],
        defaults: ProductSeedDefaults {
            access: PRODUCT_DEFAULTS.access,
            default_subscription_plan: PRODUCT_DEFAULTS.default_subscription_plan,
            subscription_plans: PRODUCT_DEFAULTS.subscription_plans,
            ai_billing: PRODUCT_DEFAULTS.ai_billing,
        },
        warnings: vec![
            "This plan is side-effect free and does not write database records.",
            "Before implementing execution, confirm tenant, environment, and conflict strategy.",
        ],
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{build_product_seed_plan, ProductSeedStepMode};

    #[test]
    fn seed_plan_is_non_destructive_and_scoped_to_tenant() {
        let plan = build_product_seed_plan();

        assert!(!plan.destructive);
        assert_eq!(plan.idempotency_scope, "tenant");
        assert!(plan
            .steps
            .iter()
            .any(|step| step.mode == ProductSeedStepMode::CreateIfMissing));
    }

    #[test]
    fn seed_plan_serializes_stable_shape() {
        let value = serde_json::to_value(build_product_seed_plan()).expect("seed plan json");

        assert_eq!(value["version"], json!(1));
        assert_eq!(value["destructive"], json!(false));
        assert_eq!(
            value["defaults"]["access"]["auth_mode"],
            json!("subscription")
        );
        assert_eq!(
            value["defaults"]["default_subscription_plan"],
            json!("studio")
        );
        assert_eq!(value["steps"][0]["mode"], json!("create_if_missing"));
    }
}
