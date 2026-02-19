pub fn generate_llms_txt(base_url: &str) -> String {
    format!(
        r#"# BattleTech Data API

> GraphQL API for BattleTech tabletop game data: units (mechs, vehicles, fighters, dropships), equipment (weapons, armor, engines), construction reference tables, factions, and eras. Data sourced from MegaMek 0.50.11 (~6,500 units, ~2,875 equipment items) enriched with Master Unit List (MUL) data (BV, roles, availability). Includes construction reference data for unit builders (engine/armor/structure/heatsink/gyro/cockpit/myomer types with weights and crit slots).

## Endpoint

POST {base_url}/graphql
Content-Type: application/json
Body: {{"query": "...", "variables": {{...}}}}

## Full Schema (SDL)

GET {base_url}/schema.graphql

## Key Concepts

- **Slugs**: lowercase, hyphen-separated identifiers. Examples: "atlas-as7-d", "clan-wolf", "medium-laser". Chassis slugs include the unit type suffix: "atlas-mech", "demolisher-vehicle"
- **Years**: in-universe BattleTech timeline years (e.g. 3025, 3055), not real-world dates
- **Tech base** values (snake_case): inner_sphere, clan, mixed, primitive
- **Rules level** values (snake_case): introductory, standard, advanced, experimental, unofficial
- **Equipment category** values (snake_case): energy_weapon, ballistic_weapon, missile_weapon, ammo, physical_weapon, equipment, armor, structure, engine, targeting_system, myomer, heat_sink, jump_jet, communications
- **Faction type** values: great_house, clan, periphery, mercenary, other
- **BV** (Battle Value): composite combat effectiveness score used for game balancing
- **MUL ID**: numeric identifier from the official Master Unit List (masterunitlist.info). Null for units not in MUL
- **Role**: tactical role from MUL (e.g. "Juggernaut", "Sniper", "Striker", "Brawler", "Missile Boat", "Scout"). Null if unassigned
- **Clan name**: alternate Clan/IS reporting name for dual-name units (e.g. "Fire Moth A" for "Dasher A"). Null for units without dual names. The `nameSearch` filter matches both `fullName` and `clanName`
- **Tonnage**: weight in metric tons (20–100 for mechs, up to 500,000+ for jumpships)
- **Range values**: measured in tabletop hexes
- **Crits**: number of critical hit slots an equipment item occupies
- **Resolved component types**: `mechData` provides both raw MegaMek strings (e.g. `engineTypeRaw`) and resolved references (e.g. `engine`) with full construction properties (weight multipliers, crit slots, etc.)
- **Construction reference**: prescriptive data for unit builders — component types with weights, crit slots, and rules; engine weight table; internal structure table

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
- Query complexity limit: 500 (expensive fields: loadout=10, locations=5, availability=5, variants=5, mechData=5, quirks=3, eras=5)
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
        clanName
        tonnage
        techBase
        rulesLevel
        bv
        introYear
        mulId
        role
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

### Get a single unit with full loadout, armor, and resolved component types
```graphql
{{
  unit(slug: "atlas-as7-d") {{
    fullName
    clanName
    tonnage
    techBase
    rulesLevel
    bv
    cost
    introYear
    mechData {{
      config
      isOmnimech
      engineRating
      walkMp
      runMp
      jumpMp
      heatSinkCount
      engine {{ name weightMultiplier ctCrits stCrits }}
      armor {{ name pointsPerTon crits }}
      structure {{ name weightFraction crits }}
      heatsink {{ name dissipation crits weight }}
      gyro {{ name weightMultiplier crits }}
      cockpit {{ name weight crits }}
      myomer {{ name properties }}
      engineTypeRaw
      armorTypeRaw
      structureTypeRaw
      heatSinkTypeRaw
      gyroTypeRaw
      cockpitTypeRaw
      myomerTypeRaw
    }}
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

### Filter OmniMechs with jump capability
```graphql
{{
  units(first: 10, isOmnimech: true, hasJump: true) {{
    edges {{
      node {{
        slug
        fullName
        tonnage
        mechData {{
          config
          engine {{ name }}
          walkMp
          runMp
          jumpMp
        }}
      }}
    }}
    pageInfo {{
      totalCount
    }}
  }}
}}
```

### Filter units by tactical role
```graphql
{{
  units(first: 10, role: "Juggernaut") {{
    edges {{
      node {{
        slug
        fullName
        tonnage
        role
        bv
      }}
    }}
    pageInfo {{
      totalCount
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

### Search equipment with builder filters
```graphql
{{
  allEquipment(first: 10, nameSearch: "laser", category: "energy_weapon", techBase: "clan") {{
    edges {{
      node {{
        slug
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
      }}
    }}
    pageInfo {{
      totalCount
    }}
  }}
}}
```

### Filter equipment by weight, crits, and location
```graphql
{{
  allEquipment(first: 20, maxTonnage: 2.0, maxCrits: 3, observedLocation: "right_arm") {{
    edges {{
      node {{
        slug
        name
        tonnage
        crits
        observedLocations
      }}
    }}
  }}
}}
```

### Find ammo types for a weapon
```graphql
{{
  equipment(slug: "autocannon-10") {{
    name
    ammoTypes {{
      slug
      name
      tonnage
      bv
    }}
  }}
}}
```

### Construction reference — fetch all data for builder initialization
```graphql
{{
  constructionReference {{
    engineTypes {{ slug name techBase weightMultiplier ctCrits stCrits }}
    armorTypes {{ slug name techBase pointsPerTon crits }}
    structureTypes {{ slug name techBase weightFraction crits }}
    heatsinkTypes {{ slug name techBase dissipation crits weight }}
    gyroTypes {{ slug name weightMultiplier crits }}
    cockpitTypes {{ slug name weight crits }}
    myomerTypes {{ slug name properties }}
    engineWeights {{ rating standardWeight }}
    internalStructure {{ tonnage head centerTorso sideTorso arm leg }}
  }}
}}
```

### Look up engine weight for a specific rating
```graphql
{{
  engineWeights(rating: 300) {{
    rating
    standardWeight
  }}
}}
```

### Look up internal structure for a tonnage
```graphql
{{
  internalStructure(tonnage: 75) {{
    head
    centerTorso
    sideTorso
    arm
    leg
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
  chassis(slug: "atlas-mech") {{
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
