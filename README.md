# BattleTech Data API

A GraphQL API serving BattleTech unit, equipment, faction, and era data sourced from [MegaMek](https://github.com/MegaMek/megamek) and the [Master Unit List](http://masterunitlist.info) (MUL).

## Stack

- **API:** Rust · axum 0.8 · async-graphql 7 · sqlx 0.8 · PostgreSQL 16
- **Scraper:** imports from MegaMek unit files (MTF + BLK formats), the Master Unit List (BV, roles, availability, clan names), and equipment stats seed data
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

This loads a pre-exported snapshot with all MegaMek + MUL data in seconds.

**Option B — Import from source:**

Download a MegaMek release tarball (e.g. `MegaMek-0.50.11.tar.gz`) and extract it. The unit data is at `data/mekfiles/unit_files.zip` inside the extracted directory.

```bash
# Step 1: Import MegaMek unit files
cargo run -p scraper@0.1.0 --release -- megamek \
  --zip /path/to/MegaMek-0.50.11/data/mekfiles/unit_files.zip \
  --version "0.50.11"

# Step 2 (optional): Enrich with MUL data (BV, cost, role, availability, clan names)
# First fetch MUL data to local files:
cargo run -p scraper@0.1.0 --release -- mul-fetch \
  --output-dir ./mul-data --delay-ms 1000

# Then import into DB:
cargo run -p scraper@0.1.0 --release -- mul-import \
  --data-dir ./mul-data
```

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
| `GET /schema.graphql` | Full GraphQL schema in SDL format |
| `GET /llms.txt` | LLM-optimized API reference document |

## GraphQL API

### Example queries

```graphql
# Paginated unit search with filters
{
  units(first: 20, nameSearch: "Atlas", techBase: "inner_sphere") {
    edges {
      node {
        slug
        fullName
        clanName
        tonnage
        bv
        cost
        introYear
        rulesLevel
        role
      }
    }
    pageInfo {
      totalCount
      hasNextPage
      endCursor
    }
  }
}

# Single unit with full detail and resolved component types
{
  unit(slug: "atlas-as7-d") {
    fullName
    clanName
    tonnage
    techBase
    bv
    cost
    role
    mechData {
      config
      isOmnimech
      engineRating
      walkMp
      runMp
      jumpMp
      heatSinkCount
      # Resolved component types with construction properties
      engine { name weightMultiplier ctCrits stCrits }
      armor { name pointsPerTon crits }
      structure { name weightFraction crits }
      heatsink { name dissipation crits weight }
      gyro { name weightMultiplier crits }
      cockpit { name weight crits }
      myomer { name }
      # Raw MegaMek strings (always available as fallback)
      engineTypeRaw
      armorTypeRaw
    }
    loadout {
      equipmentName
      location
      quantity
      isRearFacing
    }
    locations {
      location
      armorPoints
      rearArmor
      structurePoints
    }
    quirks {
      name
      isPositive
      description
    }
    availability {
      factionSlug
      factionName
      eraSlug
      eraName
      availabilityCode
    }
  }
}

# Search Clan units by alternate name
# nameSearch matches both fullName and clanName
{
  units(first: 5, nameSearch: "Fire Moth") {
    edges {
      node { slug fullName clanName }
    }
  }
}

# Filter by faction, era, and role
{
  units(first: 20, factionSlug: "clan-wolf", eraSlug: "clan-invasion", role: "Striker") {
    edges {
      node {
        slug
        fullName
        tonnage
        bv
        role
      }
    }
    pageInfo { totalCount }
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

# Chassis with all variants
{
  chassis(slug: "atlas-mech") {
    name
    unitType
    tonnage
    variants {
      slug
      fullName
      bv
      introYear
    }
  }
}

# Equipment with stats and ammo relationships
{
  equipment(slug: "autocannon-10") {
    name
    tonnage
    crits
    damage
    heat
    rangeShort
    rangeMedium
    rangeLong
    bv
    observedLocations
    ammoTypes { slug name }
  }
}

# Equipment search with builder filters
{
  allEquipment(maxTonnage: 2.0, maxCrits: 3, observedLocation: "right_arm") {
    edges {
      node { slug name tonnage crits }
    }
  }
}

# Construction reference — all component types in one request
{
  constructionReference {
    engineTypes { slug name techBase weightMultiplier ctCrits stCrits }
    armorTypes { slug name pointsPerTon crits }
    structureTypes { slug name weightFraction crits }
    heatsinkTypes { slug name dissipation crits weight }
    gyroTypes { slug name weightMultiplier crits }
    cockpitTypes { slug name weight crits }
    myomerTypes { slug name }
    engineWeights { rating standardWeight }
    internalStructure { tonnage head centerTorso sideTorso arm leg }
  }
}
```

### Filters

The `units` query supports the following filters:

| Filter | Type | Description |
|--------|------|-------------|
| `nameSearch` | String | Case-insensitive substring match on `fullName` and `clanName` |
| `techBase` | String | `inner_sphere`, `clan`, `mixed`, `primitive` |
| `rulesLevel` | String | `introductory`, `standard`, `advanced`, `experimental`, `unofficial` |
| `tonnageMin` / `tonnageMax` | Float | Weight range in metric tons |
| `factionSlug` | String | Units available to this faction (e.g. `"clan-wolf"`) |
| `eraSlug` | String | Units available in this era (e.g. `"clan-invasion"`) |
| `isOmnimech` | Bool | OmniMechs only (`true`) or non-OmniMechs (`false`) |
| `config` | String | Chassis config: `Biped`, `Quad`, `Tripod`, `LAM` |
| `engineType` | String | Engine type (e.g. `"XL Engine"`, `"Fusion Engine"`) |
| `hasJump` | Bool | Jump-capable mechs only |
| `role` | String | Tactical role (e.g. `"Juggernaut"`, `"Sniper"`, `"Striker"`) |

The `allEquipment` query supports additional builder-oriented filters:

| Filter | Type | Description |
|--------|------|-------------|
| `nameSearch` | String | Case-insensitive substring match on equipment name |
| `category` | String | Equipment category in snake_case (e.g. `"energy_weapon"`) |
| `techBase` | String | `inner_sphere`, `clan`, `mixed`, `primitive` |
| `rulesLevel` | String | `introductory`, `standard`, `advanced`, `experimental`, `unofficial` |
| `maxTonnage` | Float | Equipment weighing at most this many tons |
| `maxCrits` | Int | Equipment consuming at most this many critical slots |
| `observedLocation` | String | Equipment observed at this location (e.g. `"right_arm"`) |
| `ammoForSlug` | ID | Ammo types compatible with this weapon slug |

### Limits

- Query depth: 20
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
| `PUBLIC_BASE_URL` | `http://localhost:{PORT}` | Base URL used in `/llms.txt` and `/schema.graphql` references |

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

## Data sources

### MegaMek

Units, chassis, equipment, locations, loadout, quirks, and mech-specific data are imported from [MegaMek](https://github.com/MegaMek/megamek) release files. The scraper reads `.mtf` (mech) and `.blk` (vehicle, aerospace, etc.) formats from MegaMek's `unit_files.zip`.

### Master Unit List (MUL)

The scraper enriches MegaMek data with information from the official [Master Unit List](http://masterunitlist.info):

- **Battle Value (BV)** and **C-bill cost** for game balancing
- **Tactical roles** (Juggernaut, Sniper, Striker, Brawler, etc.)
- **MUL ID** linking to the official entry
- **Clan names** — alternate IS/Clan reporting names for dual-name OmniMechs (e.g. "Fire Moth" for "Dasher")
- **Faction/era availability** — which factions field each unit in which eras

MUL data is fetched via `mul-fetch` (saves to local files, resume-safe) and imported via `mul-import`. A pre-fetched archive is included at `mul-data.zip`. Units are matched by slug (~95% match rate for BattleMechs/vehicles).

### Data overview

All imports are idempotent — inserts use `ON CONFLICT ... DO UPDATE`.

| Table | Rows | Source |
|-------|------|--------|
| `unit_chassis` | ~1,670 | MegaMek |
| `units` | ~6,535 | MegaMek |
| `unit_mech_data` | ~4,225 | MegaMek |
| `equipment` | ~2,875 | MegaMek |
| `unit_loadout` | ~70,550 | MegaMek |
| `unit_locations` | ~33,150 | MegaMek |
| `unit_availability` | ~100,000+ | MUL |
| `eras` | 10 | seed + MUL |
| `factions` | ~70 | seed + MUL |
| `engine_types` | 9 | construction ref |
| `armor_types` | 9 | construction ref |
| `structure_types` | 6 | construction ref |
| `heatsink_types` | 4 | construction ref |
| `gyro_types` | 5 | construction ref |
| `cockpit_types` | 6 | construction ref |
| `myomer_types` | 4 | construction ref |
| `engine_weight_table` | 79 | construction ref |
| `mech_internal_structure` | 17 | construction ref |
