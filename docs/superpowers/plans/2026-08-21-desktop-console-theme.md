# Desktop Console Theme Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the web-like management interface with a dense desktop management console while preserving all current relay management behavior.

**Architecture:** Split the global stylesheet into tokens, base rules, application layout, and semantic component rules. Generic controls consume semantic variants; business views consume those controls and only define domain structure. The application shell owns navigation and connection context, while pages own data loading and command invocation.

**Tech Stack:** Nuxt 4, Vue 3, Tailwind CSS 4, TypeScript, lucide-vue-next, Bun.

---

## Theme Tokens

**Files:**
- Create: `app/assets/css/tokens.css`
- Create: `app/assets/css/base.css`
- Modify: `app/assets/css/main.css`
- Test: `tests/app-shell.test.ts`

- [ ] Add `--pr-*` semantic tokens for application canvas, panels, text, borders, focus, success, warning, danger, spacing, radii, shadows, layout sizes, and z-index levels.
- [ ] Move document sizing, font inheritance, reset rules, focus-visible treatment, and reduced-motion handling into `base.css`.
- [ ] Reduce `main.css` to the Tailwind entrypoint and ordered stylesheet imports.
- [ ] Assert that the stylesheet entrypoint imports tokens, base, layout, and components rules.

## Application Layout

**Files:**
- Create: `app/assets/css/layout.css`
- Modify: `app/components/layout/AppLayout.vue`
- Modify: `app/components/navigation/Sidebar.vue`
- Modify: `app/components/navigation/Topbar.vue`
- Modify: `app/components/layout/PageLayout.vue`
- Modify: `app/components/layout/PageHeader.vue`
- Test: `tests/legacy-web-layout.test.ts`

- [ ] Replace the current card-like canvas with a fixed desktop frame: navigation rail, compact connection bar, and one scrolling work surface.
- [ ] Give the side navigation stable dimensions, clear selected state, keyboard focus, and no marketing caption treatment.
- [ ] Make page headers a compact title-and-command row; use a separate toolbar slot for list actions and filters.
- [ ] Assert that the application shell is composed from `AppLayout`, `Sidebar`, `Topbar`, `PageLayout`, and `PageHeader`.

## Generic Components

**Files:**
- Create: `app/assets/css/components.css`
- Modify: `app/components/base/Button.vue`
- Modify: `app/components/base/IconButton.vue`
- Modify: `app/components/input/Select.vue`
- Modify: `app/components/container/Card.vue`
- Modify: `app/components/display/Table.vue`
- Modify: `app/components/display/StatCard.vue`
- Modify: `app/components/overlay/Drawer.vue`
- Modify: `app/components/feedback/ErrorState.vue`
- Create: `app/components/feedback/EmptyState.vue`
- Create: `app/components/feedback/LoadingState.vue`
- Test: `tests/legacy-web-layout.test.ts`

- [ ] Define component classes only with semantic tokens; eliminate direct color literals and utility overrides from component templates.
- [ ] Provide `Button` variants for primary, secondary, danger, and text actions with compact and regular sizes.
- [ ] Make `Table` responsible for sticky headers, row density, selected rows, and horizontal scrolling.
- [ ] Make `Card` an explicit framed work surface, not a generic page-section wrapper.
- [ ] Add reusable empty and loading feedback states alongside the existing generic error state.

## Business Work Surfaces

**Files:**
- Modify: `app/components/business/dashboard/StatsOverview.vue`
- Modify: `app/components/business/dashboard/RequestOutcomeChart.vue`
- Modify: `app/components/business/dashboard/RankingBarChart.vue`
- Modify: `app/components/business/activity/RequestTable.vue`
- Modify: `app/components/business/providers/ProviderList.vue`
- Modify: `app/components/business/providers/ProviderForm.vue`
- Modify: `app/components/business/interfaces/InterfaceList.vue`
- Modify: `app/components/business/interfaces/InterfaceForm.vue`
- Modify: `app/pages/index.vue`
- Modify: `app/pages/providers.vue`
- Modify: `app/pages/interfaces.vue`
- Modify: `app/pages/stats.vue`
- Modify: `app/pages/settings.vue`
- Modify: `app/pages/setup.vue`
- Test: `tests/workspace-flow.test.ts`, `tests/provider-flow.test.ts`, `tests/interface-flow.test.ts`, `tests/stats-flow.test.ts`

- [ ] Render the dashboard as a statistics surface with a compact summary, result distribution, and rankings; do not duplicate activity data.
- [ ] Render providers, interfaces, and activity as dense operational tables with a dedicated toolbar and feedback state.
- [ ] Keep forms in the existing right drawer but use shared controls and consistent field spacing.
- [ ] Keep all data commands, routes, labels, and connection behavior unchanged.

## Verification

**Files:**
- Modify: structural test files only when a moved responsibility requires a new source path.

- [ ] Run `bun run typecheck` and resolve every Vue and TypeScript error.
- [ ] Run `bun test` and update only assertions tied to moved component responsibilities.
- [ ] Run `git diff --check` and inspect the changed stylesheet and component boundaries before handoff.
