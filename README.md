# BattleTech Data API

A GraphQL API serving BattleTech unit, equipment, faction, and era data sourced from [MegaMek](https://github.com/MegaMek/megamek).

## Stack

- **API:** Rust · axum 0.8 · async-graphql 7 · sqlx 0.8 · PostgreSQL 16
- **Scraper:** reads MegaMek's `unit_files.zip` (MTF + BLK formats), upserts into Postgres
- **Ops:** Prometheus metrics at `/metrics`, Dockerfile (musl/Alpine), IP rate limiting

## Quick start

**1. Start Postgres**

```bash
docker run -d --name bt-postgres \
  -e POSTGRES_PASSWORD=pass \
  -p 5432:5432 \
  postgres:16
```

**2. Configure environment**

```bash
cp .env.example .env
# Edit DATABASE_URL if needed
```

**3. Install tooling and run migrations**

```bash
cargo install sqlx-cli --no-default-features --features postgres
sqlx migrate run
```

**4. Seed the database**

**Option A — Use the included seed dump (fastest):**

```bash
./seed/load.sh
```

This loads a pre-exported snapshot (MegaMek 0.50.11 — ~6,500 units, ~2,875 equipment) in seconds.

**Option B — Import from MegaMek source files:**

Download a MegaMek release tarball (e.g. `MegaMek-0.50.11.tar.gz`) and extract it. The unit data is at `data/mekfiles/unit_files.zip` inside the extracted directory.

```bash
cargo run -p scraper --release -- \
  --zip /path/to/MegaMek-0.50.11/data/mekfiles/unit_files.zip \
  --version "0.50.11"
```

This seeds reference data (eras, factions) and imports ~6,500 units with loadout, armor locations, and quirks.

**5. Run the API**

```bash
cargo run -p api
```

The server starts on `http://localhost:8080`. In debug builds, GraphiQL is available at `GET /graphql`.

## Endpoints

| Endpoint | Description |
|----------|-------------|
| `POST /graphql` | GraphQL API |
| `GET /graphql` | GraphiQL playground (debug builds only) |
| `GET /health` | Liveness check — always 200 |
| `GET /ready` | Readiness check — verifies DB connectivity and schema version |
| `GET /metrics` | Prometheus metrics |

## GraphQL API

### Example queries

```graphql
# Paginated unit search
{
  units(first: 20, nameSearch: "Atlas", techBase: "inner_sphere") {
    edges {
      node {
        id
        fullName
        tonnage
        introYear
        rulesLevel
      }
    }
    pageInfo {
      totalCount
      hasNextPage
      endCursor
    }
  }
}

# Single unit with loadout
{
  unit(slug: "atlas-as7-d") {
    fullName
    tonnage
    techBase
    loadout {
      equipmentName
      location
      quantity
    }
    locations {
      location
      armorPoints
      rearArmor
    }
  }
}

# All Clan factions
{
  allFactions(isClan: true) {
    slug
    name
    shortName
  }
}

# Era by year
{
  eraByYear(year: 3055) {
    slug
    name
    startYear
    endYear
  }
}
```

### Limits

- Query depth: 12
- Query complexity: 500
- `unitsByIds`: max 24 slugs per call
- Pagination: max 100 per page
- Rate limit: 100 req burst / ~120 req/min sustained per IP

## Docker

```bash
docker build -t battletech-api .
docker run -p 8080:8080 \
  -e DATABASE_URL=postgres://postgres:pass@host.docker.internal:5432/battletech \
  -e ALLOWED_ORIGINS=https://yourdomain.com \
  -e EXPECTED_SCHEMA_VERSION=1 \
  battletech-api
```

The image is a statically-linked musl binary on Alpine (~10 MB).

## Environment variables

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | — | PostgreSQL connection string |
| `PORT` | `8080` | Listen port |
| `ALLOWED_ORIGINS` | — | Comma-separated CORS origins; use `*` to allow all |
| `EXPECTED_SCHEMA_VERSION` | `1` | Schema version checked by `/ready` |
| `RUST_LOG` | `info` | Log filter (e.g. `debug`, `warn`, `api=debug`) |

## Deployment (timeweb.cloud)

Infrastructure is managed with Terraform in the `infra/` directory. It provisions a managed Kubernetes cluster and a managed PostgreSQL instance on [timeweb.cloud](https://timeweb.cloud), connected via a private VPC.

**1. Configure variables**

```bash
cd infra
cp terraform.tfvars.example terraform.tfvars
# Edit terraform.tfvars — set twc_token, db_password, and other values
```

**2. Provision infrastructure**

```bash
terraform init
terraform plan
terraform apply
```

**3. Get kubeconfig and apply K8s manifests**

```bash
# Save kubeconfig from Terraform output
terraform output -raw kubeconfig > ~/.kube/battletech.yaml
export KUBECONFIG=~/.kube/battletech.yaml

# Create namespace and deploy the API
kubectl apply -f k8s/namespace.yaml
kubectl create secret generic battletech-api-secrets \
  --namespace=battletech \
  --from-literal=DATABASE_URL='postgres://USER:PASS@DB_HOST:5432/battletech'
kubectl apply -f k8s/api.yaml
```

**4. Run migrations and seed the database**

Connect to the managed PostgreSQL using the VPC-internal address (from `terraform output db_host`), then:

```bash
DATABASE_URL='postgres://USER:PASS@DB_HOST:5432/battletech' sqlx migrate run
DATABASE_URL='postgres://USER:PASS@DB_HOST:5432/battletech' ./seed/load.sh
```

## Data

Units, chassis, equipment, locations, loadout, and quirks are imported from MegaMek release files. Reference data (eras, factions) is hardcoded in `crates/scraper/src/seed.rs`. Re-running the scraper is idempotent — all inserts use `ON CONFLICT ... DO UPDATE`.

MegaMek 0.50.11 produces approximately:

| Table | Rows |
|-------|------|
| `unit_chassis` | 1,638 |
| `units` | 6,535 |
| `equipment` | 2,875 |
| `unit_loadout` | 70,549 |
| `unit_locations` | 33,156 |
| `eras` | 10 |
| `factions` | 33 |
