//! Product-layer boundary for project-specific SaaS customization.
//!
//! Keep shared SaaS and AI billing capabilities in the existing core modules
//! (`auth`, `iam`, `customer`, `subscription`, `ai`, `server_api`,
//! `web_assets`, `web_works`, `audit`, and `system`). Concrete products can add
//! product-only routes, repositories, and models under this module without
//! changing those core contracts.

pub mod admin;
pub mod defaults;
pub mod execute;
pub mod history;
pub mod preflight;
pub mod seed_plan;
