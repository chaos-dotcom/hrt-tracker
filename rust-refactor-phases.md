# rust-refactor-phases

## Phase 0 — Compatibility Contract
- Lock API surface: `/api/data`, `/api/settings`, `/api/ics`, `/api/ics/{secret}`, `/api/convert`, `/api/dosage-photo/{entryId}`, `/api/dosage-photo/{entryId}/{filename}`.
- Lock UI routes: `/`, `/create/dosage`, `/create/blood-test`, `/create/measurement`, `/view`, `/stats`, `/backup`, `/calc`, `/vials`, `/vials/create`, `/vials/{id}`, `/estrannaise`.
- Lock data locations and formats:
  - `data/hrt-data.json` (JSON; exclude `settings` on write).
  - `data/hrt-settings.yaml` (YAML).
  - `data/dosage-photos/{entryId}/{filename}` (binary).
- Lock response semantics:
  - ICS content type/headers and query params.
  - Image response content types.
  - Error/status code behavior for missing/invalid files.
  - `/api/data` GET/POST parity (strip `settings`, atomic write, `{}` on missing/invalid).
  - `/api/settings` GET/POST parity (YAML read/write, `{}` on missing).
  - `/api/convert` parity (conversion math + error handling).
  - `/api/dosage-photo` parity (upload behavior + GET/DELETE semantics).
- Lock state rules:
  - Blood test fudge-factor migration.
  - Auto-backfill scheduled doses.
  - Snap-to-next injection boundary logic.
- Non-goals for Phase 0:
  - No DB migration or file format changes.
  - No route renames or behavioral changes.

## Phase 1 — Workspace + Build Skeleton
- Create Rust workspace: `crates/shared`, `crates/server`, `crates/web`.
- Configure `cargo-leptos` for CSR build pipeline.
- Configure `crates/server` static asset serving for the CSR bundle.
- Add baseline lint/format (rustfmt + clippy) and shared dev scripts.

## Phase 2 — Domain Model Port (Shared Crate)
- Port enums and types from TypeScript:
  - Use `serde` with string enums and `#[serde(rename_all = "camelCase")]` where needed.
  - Implement tagged enum for `DosageHistoryEntry` via `#[serde(tag = "medicationType")]`.
- Implement derived defaults (`#[serde(default)]`) for arrays to avoid `null`/missing.
- Implement domain logic:
  - Fudge-factor migration for blood tests.
  - Snap-to-next injection boundary.
  - Auto-backfill dose schedules.
- Port hormone unit conversion logic.

## Phase 3 — Persistence Layer (Server Crate)
- JSON read/write for `hrt-data.json` with atomic write (`.tmp` + rename).
- YAML read/write for `hrt-settings.yaml`.
- Photo filesystem storage:
  - Safe filename generation.
  - Content-type inference from extension.
  - Directory creation per `entryId`.

## Phase 4 — API Parity (Server Crate)
- `/api/data`
  - GET: read JSON, return `{}` if missing, strip invalid top-level, exclude `settings`.
  - POST: accept JSON, strip `settings`, atomic save.
- `/api/settings`
  - GET: parse YAML, return `{}` if missing.
  - POST: accept JSON, write YAML.
- `/api/ics` and `/api/ics/{secret}`
  - Implement identical calendar generation, scheduling rules, and headers.
- `/api/convert`
  - JSON input/output with validation.
- `/api/dosage-photo/*`
  - Multipart upload, list of filenames in response.
  - GET and DELETE by filename.

## Phase 5 — Leptos CSR UI (Web Crate)
- Rebuild routing and pages:
  - `/`, `/create/*`, `/view`, `/stats`, `/backup`, `/calc`, `/vials`, `/estrannaise`.
- Recreate state store (equivalent of `storage.svelte.ts`):
  - Initialization with server data.
  - Local changes reflected in UI.
  - Explicit save with debounce.
- Rebuild modal, forms, and validation behavior.

## Phase 6 — Charting (Pure Rust)
- Integrate `plotters` + `plotters-canvas` in CSR.
- Implement chart rendering functions that map to existing chart semantics.
- Wire to controls so chart re-renders on user changes.
- Ensure axes, units, and dataset selections match existing behavior.

## Phase 7 — End-to-End Parity Verification
- Load existing `hrt-data.json` and `hrt-settings.yaml` without loss.
- Validate API outputs against current app:
  - ICS generation (compare outputs).
  - Conversion results.
  - Photo upload/download flow.
- UI parity checks for main flows:
  - Create/record dosage.
  - Record blood tests.
  - Backup/restore.
  - Stats view.

## Phase 8 — Packaging + Deployment
- Build CSR assets via `cargo-leptos`.
- Serve assets through Rust server.
- Update Dockerfile to build single binary + assets.
- Update run/deploy instructions.
