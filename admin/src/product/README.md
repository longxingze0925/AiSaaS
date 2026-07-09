# Product Layer

This directory is the customization boundary for concrete SaaS products.

For the full project customization workflow, start with
`../../../docs/internal/development/产品魔改清单.md`.

Use it for:

- Product name, short name, brand mark, and high-level copy in `config.ts`.
- Product API access defaults, server API key scopes, and scope labels in `access.ts`.
- Product subscription presets, default features, and seat defaults in `subscription.ts`.
- Product AI model templates and manual billing operation reasons in `ai.ts`.
- Product-specific menu entries in `menu.tsx`.
- Product-specific protected routes in `routes.tsx`.
- Future product-only pages, API wrappers, and types.
- `ProductSetupPage.tsx` is the initialization console. It previews Product
  Layer defaults, shows backend preflight checks, can run the controlled
  initializer, and lists seed execution history.

Core SaaS capabilities should stay outside this directory: auth, RBAC, users,
subscriptions, wallet, AI gateway, usage, assets, audit, and system settings.
When customizing a project, prefer adding product modules here instead of
editing core routes and menus directly.

For a concrete AI SaaS project, start with:

1. `subscription.ts` for plan codes, default seat limits, and feature flags.
2. `access.ts` for default access config and server API key scopes.
3. `ai.ts` for wallet adjustment presets, project-specific model templates, and billing operation reasons.
4. `config.ts` for brand copy.
5. `menu.tsx` and `routes.tsx` for product-only surfaces.

The built-in `/product/setup` page is the first stop after project
customization:

1. Check frontend and backend defaults in the dry-run snapshot.
2. Resolve any preflight `conflict` before writing data.
3. Execute initialization only in the intended tenant and environment.
4. Copy one-time `app_secret` and server key values before leaving the page.
5. Use execution history for resource IDs and request tracing; historical
   secrets are intentionally not recoverable.
