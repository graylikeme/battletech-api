# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Build
cargo build -p api
cargo build -p scraper
cargo build --workspace

# Run the API server (requires .env or env vars)
DATABASE_URL=postgres://postgres:pass@localhost:5432/battletech cargo run -p api

# Run the scraper (one-shot data import)
DATABASE_URL=postgres://postgres:pass@localhost:5432/battletech \
  cargo run -p scraper --release -- \
  --zip /path/to/unit_files.zip \
  --version "0.50.11"

# Seed database from dump (alternative to running the scraper)
./seed/load.sh            # uses DATABASE_URL from .env

# Migrations
sqlx migrate run          # apply pending migrations
sqlx migrate revert       # roll back last migration

# After changing any query_as! macro in crates/api, regenerate the offline cache:
cargo sqlx prepare --workspace
```

There is no test suite yet.

```bash
# Terraform (infra/)
cd infra && terraform init      # download providers
cd infra && terraform validate  # check syntax
cd infra && terraform plan      # preview changes
cd infra && terraform apply     # provision infrastructure
```

## Environment

Copy `.env.example` to `.env`. Required vars: `DATABASE_URL`, `PORT` (default 8080), `ALLOWED_ORIGINS`, `EXPECTED_SCHEMA_VERSION`. Optional: `PUBLIC_BASE_URL` (used in `/llms.txt`; defaults to `http://localhost:{PORT}`).

Rust toolchain is pinned to **1.89.0** via `rust-toolchain.toml` (required by async-graphql 7 / edition2024).

## Architecture

Two crates in the workspace:

### `crates/api` — GraphQL HTTP server

**Stack:** axum 0.8 · async-graphql 7 · sqlx 0.8 (compile-time `query_as!`) · moka cache · tower_governor rate limiting · metrics-exporter-prometheus

**Request flow:**
```
HTTP POST /graphql
  → GovernorLayer (100 req burst / 2 per-sec replenishment, per IP)
  → TimeoutLayer (30s)
  → CorsLayer
  → TraceLayer
  → graphql_handler (handlers/graphql.rs)
      → async-graphql executes query against QueryRoot
      → QueryRoot resolvers call db/* functions
      → db/* functions run sqlx queries against Postgres
```

**Key constraints baked into the schema:**
- Depth limit: 20, Complexity limit: 500 (`graphql/schema.rs`). Depth is set to 20 (not lower) because GraphiQL's introspection query needs ~15 levels.
- Expensive fields annotated with `#[graphql(complexity = N)]`: `locations` (5), `loadout` (10), `quirks` (3), `availability` (5), `variants` (5), `mechData` (5)
- `unitsByIds` accepts at most 24 slugs

**GraphQL types live in two layers:**
- `db/models.rs` — `Db*` structs derived from `FromRow`, used only inside the db layer
- `graphql/types/*.rs` — `*Gql` newtypes wrapping `Db*`, with `#[Object]` impls that expose the GraphQL API

**DataLoader** (`graphql/loaders.rs`): `MechDataLoader` uses async-graphql's `dataloader` feature to batch-load `unit_mech_data` rows, preventing N+1 queries when `mechData` is requested in list queries.

**Pagination** (`graphql/pagination.rs`): keyset cursors encoded as `base64("sort_val|id:N")`. The `search` functions in `db/units.rs` and `db/equipment.rs` use `QueryBuilder` for dynamic WHERE clauses and `COUNT(*) OVER()` for total count in a single query.

**sqlx offline mode:** The `.sqlx/` directory contains pre-generated query metadata. All `query_as!` macros in `crates/api` use `"col!"` non-null overrides for PostgreSQL enum columns cast to text (e.g. `u.tech_base::text AS "tech_base!"`). After any schema or query change, run `cargo sqlx prepare --workspace` with a live DB, then commit the updated `.sqlx/` files. The `SQLX_OFFLINE=true` env var (set in Docker builds) disables live-DB checks.

**Static text endpoints:** `GET /schema.graphql` and `GET /llms.txt` serve precomputed strings (SDL and LLM-optimized API reference) generated once at startup. Both return `Content-Type: text/plain` with `Cache-Control: public, max-age=3600`. The handler is a shared `static_text_handler` in `main.rs` that takes a `State<String>`.

**Router state:** Five sub-routers each carry a different state type (`AppSchema`, `String` for SDL, `String` for llms.txt, `AppState`, `PrometheusHandle`) merged into a single `Router` — this is intentional to satisfy axum's type system.

**GraphQL descriptions:** All types, fields, and query parameters have descriptions exposed via introspection. For `#[derive(SimpleObject)]` types, use `///` doc comments on the struct and its fields. For `#[Object]` types, place the type-level `///` doc comment on the `impl` block (not the struct), and field-level `///` doc comments on each resolver method. For query parameters, use `#[graphql(desc = "...")]` inline on the parameter.

### `crates/scraper` — One-shot MegaMek data importer

Reads a MegaMek `unit_files.zip` (found inside the release tarball at `data/mekfiles/unit_files.zip`) and upserts all units into Postgres.

**MegaMek file formats:**
- `.mtf` — custom key:value text format for Mech units (in `meks/` subdirectory)
- `.blk` — XML-like tag format for all other unit types (vehicles, fighters, dropships, etc.)

**Parse → DB pipeline:**
1. `parse.rs` — `parse_mtf()` / `parse_blk()` return `Option<ParsedUnit>`. MTF parsing also extracts mech-specific data (engine, movement, heat sinks, armor/structure types, gyro, cockpit, myomer) into `ParsedMechData`.
2. `db.rs` — runtime `sqlx::query` (no `!` macro) upserts: chassis → unit → locations → loadout → quirks → mech_data. Uses runtime queries (not `query!`) to avoid compile-time issues with custom PG enum types.
3. `seed.rs` — seeds the 10 standard eras, 33 factions, and `dataset_metadata` row

The scraper maintains an in-process `HashMap<slug, equipment_id>` cache to avoid re-inserting the same equipment for every unit that carries it.

## Database

Schema is in `migrations/`. PostgreSQL enums defined: `tech_base_enum`, `rules_level_enum`, `equipment_category_enum`, `location_name_enum`.

`tonnage` columns are `NUMERIC(10,1)` (widened in migration 2 from `NUMERIC(6,1)` to accommodate dropships/jumpships up to ~500,000 tons).

Unit slugs are lowercased, non-alphanumeric characters replaced with hyphens and deduplicated (e.g. `"Atlas AS7-D"` → `"atlas-as7-d"`). Chassis slugs include the unit type suffix to avoid collisions between different unit types sharing a name (e.g. `"atlas-mech"`, `"demolisher-vehicle"`). Unit slugs are derived from `"chassis model"`.
