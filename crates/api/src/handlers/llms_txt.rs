pub fn generate_llms_txt(base_url: &str) -> String {
    format!(
        r#"# BattleTech Data API

> GraphQL API for BattleTech tabletop game data: units (mechs, vehicles, fighters, dropships), equipment (weapons, armor, engines), factions, and eras. Data sourced from MegaMek 0.50.11 (~6,500 units, ~2,875 equipment items).

## Endpoint

POST {base_url}/graphql
Content-Type: application/json
Body: {{"query": "...", "variables": {{...}}}}

## Full Schema (SDL)

GET {base_url}/schema.graphql

## Key Concepts

- **Slugs**: lowercase, hyphen-separated identifiers. Examples: "atlas-as7-d", "clan-wolf", "medium-laser"
- **Years**: in-universe BattleTech timeline years (e.g. 3025, 3055), not real-world dates
- **Tech base** values (snake_case): inner_sphere, clan, mixed, primitive
- **Rules level** values (snake_case): introductory, standard, advanced, experimental, unofficial
- **Equipment category** values (snake_case): energy_weapon, ballistic_weapon, missile_weapon, ammo, physical_weapon, equipment, armor, structure, engine, targeting_system, myomer, heat_sink, jump_jet, communications
- **Faction type** values: great_house, clan, periphery, mercenary, other
- **BV** (Battle Value): composite combat effectiveness score used for game balancing
- **Tonnage**: weight in metric tons (20–100 for mechs, up to 500,000+ for jumpships)
- **Range values**: measured in tabletop hexes
- **Crits**: number of critical hit slots an equipment item occupies

## Pagination

Keyset cursor pagination on `units` and `allEquipment` queries.

Parameters:
- `first`: items per page (default 20, max 100)
- `after`: opaque cursor string from a previous `pageInfo.endCursor`

Response shape:
```graphql
{{
  edges {{ cursor, node {{ ... }} }}
  pageInfo {{ hasNextPage, hasPreviousPage, startCursor, endCursor, totalCount }}
}}
```

To paginate: pass `endCursor` from the previous response as `after` in the next request.

## Limits

- Query depth limit: 20
- Query complexity limit: 500 (expensive fields: loadout=10, locations=5, availability=5, variants=5, quirks=3, eras=5)
- `unitsByIds`: max 24 slugs per call
- Pagination: max 100 items per page
- Rate limit: 100 request burst / ~120 requests/min sustained (per IP)

## Example Queries

### Search units by name
```graphql
{{
  units(first: 10, nameSearch: "Atlas") {{
    edges {{
      node {{
        slug
        fullName
        tonnage
        techBase
        rulesLevel
        bv
        introYear
      }}
    }}
    pageInfo {{
      totalCount
      hasNextPage
      endCursor
    }}
  }}
}}
```

### Get a single unit with full loadout and armor
```graphql
{{
  unit(slug: "atlas-as7-d") {{
    fullName
    tonnage
    techBase
    rulesLevel
    bv
    cost
    introYear
    loadout {{
      equipmentName
      location
      quantity
      isRearFacing
    }}
    locations {{
      location
      armorPoints
      rearArmor
      structurePoints
    }}
    quirks {{
      name
      isPositive
      description
    }}
  }}
}}
```

### Filter units by faction and era
```graphql
{{
  units(first: 20, factionSlug: "clan-wolf", eraSlug: "clan-invasion", techBase: "clan") {{
    edges {{
      node {{
        slug
        fullName
        tonnage
        bv
      }}
    }}
    pageInfo {{
      totalCount
      hasNextPage
      endCursor
    }}
  }}
}}
```

### Paginate through results using cursor
```graphql
{{
  units(first: 20, after: "QXRsYXMgQVM3LUR8aWQ6NDI=", nameSearch: "Atlas") {{
    edges {{
      node {{
        slug
        fullName
      }}
    }}
    pageInfo {{
      hasNextPage
      endCursor
    }}
  }}
}}
```

### Batch lookup by slugs
```graphql
{{
  unitsByIds(slugs: ["atlas-as7-d", "mad-cat-prime", "hunchback-hbk-4g"]) {{
    slug
    fullName
    tonnage
    techBase
    bv
  }}
}}
```

### Search equipment
```graphql
{{
  allEquipment(first: 10, nameSearch: "laser", category: "energy_weapon", techBase: "clan") {{
    edges {{
      node {{
        slug
        name
        tonnage
        damage
        heat
        rangeShort
        rangeMedium
        rangeLong
        bv
      }}
    }}
    pageInfo {{
      totalCount
    }}
  }}
}}
```

### List all Clan factions
```graphql
{{
  allFactions(isClan: true) {{
    slug
    name
    shortName
    foundingYear
  }}
}}
```

### Get era by year
```graphql
{{
  eraByYear(year: 3055) {{
    slug
    name
    startYear
    endYear
  }}
}}
```

### Get chassis with all its variants
```graphql
{{
  chassis(slug: "atlas") {{
    name
    unitType
    techBase
    tonnage
    variants {{
      slug
      variant
      fullName
      techBase
      bv
      introYear
    }}
  }}
}}
```
"#,
        base_url = base_url,
    )
}
