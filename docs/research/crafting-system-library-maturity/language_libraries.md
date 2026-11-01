# Engine-agnostic crafting / inventory / recipe / production-chain libraries across ecosystems

Method note: Download counts, versions and dates for crates.io, npm, NuGet and GitHub were pulled live from each registry's public API on 2026-11-01 (crates.io `/api/v1/crates`, npm `/-/v1/search`, NuGet `azuresearch` query, GitHub REST via `gh api`), PyPI from `pypi.org/pypi/<pkg>/json` and pepy.tech. The links cite the human-readable page for each package. "Stars" means GitHub stars on that date.

## Rust (crates.io, beyond Bevy plugins): what exists?

### Takeaway
Rust has no established engine-agnostic crafting or recipe library. Every relevant crate has fewer than about 13k lifetime downloads, and most were last released between 2018 and 2024 or are 2026 single-author first releases with fewer than 300 downloads. Inventory containers exist in small numbers. A data-driven recipe/crafting rules engine, a production-chain solver, and progression/unlock crates are essentially absent. The Factorio-adjacent crates parse game files; none is a reusable recipe-graph solver.

### Cited Findings
**Inventory and crafting crates (sorted by lifetime downloads):**
- `inv-sys`: "A simple and effective inventory system for games." 12,984 downloads, v1.4.1, last release 2021-12-16, 10 versions, MIT OR Apache-2.0. Dormant for about 5 years. — [crates.io/inv-sys](https://crates.io/crates/inv-sys)
- `game_inventory`: "An inventory system independant of item data." 10,477 downloads (32 recent), v2.1.0, last release 2024-10-29, MIT. Repo SoulShardStudios/game_inventory has 11 stars and was last pushed 2024-10-29. — [crates.io/game_inventory](https://crates.io/crates/game_inventory); [GitHub](https://github.com/SoulShardStudios/game_inventory)
- `lootr` ("A simple RPG-like looting system"): 4,030 downloads, v0.7.0, 2023-03-13. — [crates.io/lootr](https://crates.io/crates/lootr)
- `crafting-helper` ("A small generic tool to calculate crafting times and resource usage for games"): 1,908 downloads, a single version v1.0.0 from 2019-02-22, AGPL-3.0-only. — [crates.io/crafting-helper](https://crates.io/crates/crafting-helper)
- `itemgen` ("Generic library for generating items in an RPG"): 1,917 downloads, v0.0.0, 2018, a name placeholder. — [crates.io/itemgen](https://crates.io/crates/itemgen)
- `crafting-recipe`: web search reported v0.1.0, MIT, about four years old, 1,476 downloads, by Lucien Greathouse. The crates.io API returned no record for it on 2026-11-01, so it may have been removed. Treat its status as unconfirmed. — [crates.io/crafting-recipe](https://crates.io/crates/crafting-recipe); [lib.rs](https://lib.rs/crates/crafting-recipe)
- `inventory_kit`: 1,486 downloads, v0.1.2, 2025-05-31. The description ("dynamic inventory management") reads as business or warehouse inventory rather than games. — [crates.io/inventory_kit](https://crates.io/crates/inventory_kit)
- `stdb-inventory`: 476 downloads, v0.1.0, 2025-07-10, MIT. Inventory utilities tied to SpacetimeDB, so it is backend-bound. — [crates.io/stdb-inventory](https://crates.io/crates/stdb-inventory)
- `bevy_inventory` (KBVE monorepo): 201 downloads, a single v0.1.0 release on 2026-05-02, MIT. It describes itself as a "Pure Rust core; optional Bevy ECS integration via the `bevy` feature", but `bevy` 0.18 is a *default* feature. It covers slot-based stacking, transfer and serde snapshots and does **not** mention crafting or recipes. Sibling crate `bevy_items` ("Proto-driven item definitions ... compiles itemdb.proto into typed Rust structs") has 60 downloads. — [lib.rs/bevy_inventory](https://lib.rs/crates/bevy_inventory); [crates.io/bevy_items](https://crates.io/crates/bevy_items)
- `game-utils-inventory`: "Pure inventory primitives - items, slots, shaped grids, equipment, crafting, trade." 81 downloads, 5 versions between 2026-09-11 and 2026-10-15 (v0.4.3), MIT OR Apache-2.0. It depends only on `game-utils-grid` and `serde`, so no engine. The repo mlm-games/game-utils has 0 stars. lib.rs gives no detail on its recipe format. — [lib.rs/game-utils-inventory](https://lib.rs/crates/game-utils-inventory); [GitHub](https://github.com/mlm-games/game-utils)
- `openstranded-common-crafting`: OpenStranded "Shared data types for the OpenStranded crafting system" (recipes, ingredients, results; serde only). 65 downloads, v0.1.0 on 2026-07-11, GPL-3.0-or-later, 0 stars. Companion `openstranded-common-items` (ItemDef, ItemGroup, Material, ItemStack) has 82 downloads. These are one game's data types, not a general engine. — [crates.io](https://crates.io/crates/openstranded-common-crafting/0.1.0); [GitHub](https://github.com/OpenStranded/common-crafting); [crates.io/openstranded-common-items](https://crates.io/crates/openstranded-common-items)
- `valence_inventory` (Minecraft server framework Valence): 2,875 downloads, v0.2.0-alpha.1+mc.1.20.1, 2023. It is Minecraft-protocol-bound. — [crates.io/valence_inventory](https://crates.io/crates/valence_inventory)

**Factorio, Minecraft and other game-adjacent crates (parsers and data, not engines):**
- `factorio-exporter` ("export prototype definitions from Factorio"): 14,621 downloads, v0.9.0, 2022-11-26. Repo MForster/factorio-rust-tools has 4 stars. — [crates.io/factorio-exporter](https://crates.io/crates/factorio-exporter)
- `factorio-blueprint` (read and write blueprint strings): 9,898 downloads, v0.3.0, 2021-11-17, GPL-3.0, 20 stars, about 19% documented. — [crates.io/factorio-blueprint](https://crates.io/crates/factorio-blueprint); [docs.rs](https://docs.rs/crate/factorio-blueprint/latest)
- `gac` ("Generic and Atomic Recipe Calculator for Factorio"): 4,235 downloads, v0.1.2, single day 2023-08-08, MIT. This is the only Rust "recipe calculator" crate found, and it is dormant. — [crates.io/gac](https://crates.io/crates/gac)
- `factorio-prototypes-json` v0.1.0: Rust types for Factorio's prototype JSON (format version 6). — [docs.rs](https://docs.rs/crate/factorio-prototypes-json/latest)
- `mcdata-rs` (access to PrismarineJS/minecraft-data): 1,926 downloads, v0.1.2, 2025-04-15. — [crates.io/mcdata-rs](https://crates.io/crates/mcdata-rs)
- Name trap: `inventory` (152.6M downloads, v0.3.25, 2026-10-07) is "Typed distributed plugin registration", unrelated to game inventories. — [crates.io/inventory](https://crates.io/crates/inventory)
- Progression and tech trees: crates.io searches for "tech tree", "progression unlock", "recipe book", "factory planner" and "production planner" returned no game-progression or production-planner crate (results were unrelated tools). — [crates.io search](https://crates.io/search?q=tech%20tree)

### Inferences
- The largest "game inventory" crates (about 10 to 13k lifetime downloads) are roughly four orders of magnitude below foundational crates, and both are dormant. Rust game devs either roll their own or stay inside Bevy plugins.
- 2026 brought a cluster of fresh single-author attempts (`game-utils-inventory`, `bevy_inventory`, `openstranded-common-*`, `bevy_items`). Interest is rising, but none has adoption, docs or a recipe model yet.
- The specific niche this project needs is unoccupied in Rust: data-driven recipes, a recipe book that persists learned entries, mines with replenishment, and a production-chain rate solver.

### Gaps
- I could not read the actual recipe API of `game-utils-inventory` (README unavailable via API, and lib.rs gives no detail), so whether it supports data-driven recipes is unknown.
- The status of `crafting-recipe` is unconfirmed (web index vs. API miss).

## Other ecosystems: C#/.NET, C++, JS/TS, Python, Lua, Haxe, Go, Java/Kotlin

### Takeaway
No ecosystem has a widely adopted, engine-agnostic crafting or recipe engine. The mature, high-star projects in this space are all **engine-bound plugins** (Godot: GLoot 990★, Expresso inventory-system 739★) or **game-bound data libraries** (Minecraft: PrismarineJS minecraft-data 957★, prismarine-recipe about 62k npm downloads/week; FFXIV crafting solvers). Standalone, generic attempts exist in every ecosystem, but all have tiny usage (two- or three-digit downloads, or single-digit stars).

### Cited Findings
**C#/.NET (NuGet):**
- `GameLib` 0.1.5: "Library providing basic gaming related functionality such as inventory management and craf[ting]". 33,786 total downloads, the largest generic hit on NuGet. — [NuGet GameLib](https://www.nuget.org/packages/GameLib)
- `InventoryFramework.SDK` 1.0.8: "Engine-agnostic client SDK for InventoryFramework: server-authoritative inventory and crafting", 2,395 downloads. It is a client for a gRPC server, licensed through RSA-signed license files (commercial). Separate Godot, Unreal and Unity adapters have about 1.5k downloads each. — [NuGet InventoryFramework.SDK](https://www.nuget.org/packages/inventoryframework.sdk); [Unity adapter](https://www.nuget.org/packages/InventoryFramework.UnityAdapter/1.0.1)
- `Variable.Inventory` 1.0.13 ("inventory management, capacity checks, and item transfers"): 1,904 downloads. — [NuGet](https://www.nuget.org/packages/Variable.Inventory)
- `SetNet.Crafting` 1.2.0: MIT, server-authoritative crafting tied to the SetNet networking framework, 127 downloads. `SetNet.Inventory` has 424. — [NuGet SetNet.Crafting](https://www.nuget.org/packages/SetNet.Crafting/)
- `RaphaelSharp` 1.1.11: C# bindings to the Rust FFXIV crafting solver `raphael-rs`, 519 downloads. It is game-specific. — [NuGet RaphaelSharp](https://www.nuget.org/packages/RaphaelSharp)
- `TheChest.Core`: simple container library with Unity and Godot companions, last updated October 2024, no crafting. — [GitHub The-Chest](https://github.com/The-Chest)
- PlayFab SDK (4.9M downloads) includes inventory, but as a hosted backend service, not a rules library. — [NuGet PlayFabAllSDK](https://www.nuget.org/packages/PlayFabAllSDK)

**GDScript/Godot (engine-bound, the reference for "mature"):**
- peter-kish/GLoot ("A universal inventory system for the Godot game engine"): 990★, MIT, pushed 2026-07-14. — [GitHub](https://github.com/peter-kish/gloot)
- expressobits/inventory-system ("Modular inventory system for godot 4 ... separate logic", includes crafting): 739★, C++ GDExtension, MIT, pushed 2026-02-20. — [GitHub](https://github.com/expressobits/inventory-system)
- zcaliptium/gdinv (data-driven inventory plugin): 77★, last pushed 2023. Menkos/inventory-forge (Godot item-database editor): 27★. — [gdinv](https://github.com/zcaliptium/gdinv); [inventory-forge](https://github.com/Menkos/inventory-forge)

**JavaScript/TypeScript (npm):**
- `prismarine-recipe` 1.5.0 ("Represent minecraft recipes"): about 62,550 weekly downloads, published 2026-03-30. Repo 11★. Minecraft-specific, driven by mineflayer bots. — [npm](https://www.npmjs.com/package/prismarine-recipe); [GitHub](https://github.com/PrismarineJS/prismarine-recipe)
- `@jsprismarine/recipes` 0.16.0 ("Java Edition recipe evaluator: all 22 recipe serializers, a crafting matcher"): about 1.4k weekly, 2026-10-31. Minecraft-specific. — [npm](https://www.npmjs.com/package/@jsprismarine/recipes)
- `craftingrecipes` (amorphous and positional recipes plus an item "thesaurus"; `computeOutput`/`matches`/`craft` on an inventory) and `voxel-recipes` 1.0.0 (2016, about 19 weekly). Both are generic but abandoned voxel.js-era code. — [npm craftingrecipes](https://npmjs.com/package/craftingrecipes); [npm voxel-recipes](https://www.npmjs.com/package/voxel-recipes)
- `@ffxiv-teamcraft/simulator` 3.7.3 (about 495 weekly). The parent ffxiv-teamcraft app has 1,968★. Game-specific. — [npm](https://www.npmjs.com/package/@ffxiv-teamcraft/simulator); [GitHub](https://github.com/ffxiv-teamcraft/ffxiv-teamcraft)
- `@kgrech/factorio-calculator-core` 1.0.15 ("core algorithm of the factorio calculator"): 0 weekly downloads, last published 2021. This is the only npm package that tries to extract a calculator core as a library, and it is abandoned. — [npm](https://www.npmjs.com/package/@kgrech/factorio-calculator-core)
- Generic game-inventory packages are effectively unused: `game-inventory` 0.0.1 (2022, 3 weekly), `encumber` 0.0.1 (2017, 4 weekly), `miaoda-game-inventory-react` (20 weekly). — [npm search](https://www.npmjs.com/search?q=game%20inventory)
- Game-data typings rather than engines: `typed-factorio` 4.8.0 (Factorio modding API types, 2026-10-30) and `@satisfactory-dev/docs.json.ts` (types for Satisfactory's shipped Docs.json). — [npm typed-factorio](https://www.npmjs.com/package/typed-factorio); [npm docs.json.ts](https://www.npmjs.com/package/@satisfactory-dev/docs.json.ts)

**Python (PyPI):**
- `factoriocalc` 0.3.0 (last upload 2025-04-04, about 10.6k total downloads on pepy) "uses a custom simplex solver written in pure python" and handles multi-output recipes (oil, uranium). Factorio-specific. — [FactorioCalc docs](https://factoriocalc.readthedocs.io/); [PyPI](https://pypi.org/project/factoriocalc/)
- `factorio-draftsman` 4.0.1 (2026-10-04, about 70.7k total downloads) manipulates blueprint strings. It is a blueprint tool, not crafting. — [PyPI](https://pypi.org/project/factorio-draftsman/)
- `minecraft-data` (Python) 3.20.0, last release 2022-12-30. — [PyPI](https://pypi.org/project/minecraft-data/)
- GitHub search for Python crafting/recipe libraries found only `svioletg/mc-recipe-img` (0★). — [GitHub](https://github.com/svioletg/mc-recipe-img)

**Lua:**
- Lua crafting code is overwhelmingly FiveM/QBCore server scripts, for example swkeep/keep-crafting (98★, 2023) and the AlexJamesDean `ajd-*` resources. ComputerCraft has egor-oleg/aegis-autocraft (31★). — [keep-crafting](https://github.com/swkeep/keep-crafting); [aegis-autocraft](https://github.com/egor-oleg/aegis-autocraft)
- The awesome-love2d curated list has **no** entry matching inventory, crafting, recipe or item. — [awesome-love2d README](https://github.com/love2d-community/awesome-love2d)

**Java/Kotlin, Go, Haxe, C++:**
- Java: liplum/MultiCrafterLib (Mindustry multi-recipe crafter library, 62★) and several Minecraft/Bukkit/Hytale crafting libs at 0 to 2★. All are game-bound. — [MultiCrafterLib](https://github.com/liplum/MultiCrafterLib); [GitHub search](https://github.com/search?q=language%3AJava+recipe+library+crafting)
- Go: only game-specific tools (an Albion market profit tool, a Minecraft recursive-recipe helper, each 0 to 1★). — [GitHub search](https://github.com/search?q=language%3AGo+crafting+recipe)
- Haxe: nothing relevant found. Haxe's notable adjacent tool is CastleDB (ncannasse/castle, 611★, pushed 2026-10-25), "a structured database" for game data, which is a generic item-catalog editor rather than a crafting engine. — [CastleDB](https://github.com/ncannasse/castle)
- C++: a GitHub search for "inventory crafting library" in C++ returned no results. The only notable C++ entry is the Godot GDExtension above. — [GitHub search](https://github.com/search?q=language%3AC%2B%2B+inventory+crafting+library)
- GitHub topics: `topic:crafting-system` has only 12 repos in total (top: 3★), `topic:recipe-system` has 4, and `topic:item-system` has 6. `topic:inventory-system` has 438, but its top entries are business inventory apps (Part-DB 1,824★) plus GLoot. — [topic:crafting-system](https://github.com/topics/crafting-system); [topic:inventory-system](https://github.com/topics/inventory-system)

### Inferences
- The pattern is consistent across ecosystems: maturity comes from coupling to a host (a Godot plugin, or the Minecraft and FFXIV data ecosystems), not from standing alone. Generic standalone crafting libraries exist in every language, but none has reached meaningful adoption.
- Unity is not covered here (Asset Store assets are engine-bound and out of scope). The Unity Asset Store's many commercial "Inventory and Crafting" packs show demand, but they do not show a standalone category.

### Gaps
- Unity Asset Store, vcpkg/conan and LuaRocks were not queried directly. LuaRocks has no public search API that I used, and the conclusions for those come from GitHub search only.
- npm weekly-download figures come from the npm search API's `downloads.weekly` field and were not cross-checked against npm-stat.

## Factory/production-chain tools: do they expose reusable recipe-graph or solver libraries?

### Takeaway
The major planners are mature, actively maintained **applications**, not libraries. FactorioLab (881★) and Kirk McDonald's calculator (754★) embed their solvers in app code. The one attempt to publish a Factorio calculator core on npm is abandoned. The reusable layer people actually share is the **generic LP solver** (GLPK, HiGHS, javascript-lp-solver/YALPS, scipy linprog) plus per-game data dumps. FactorioLab shows a multi-game recipe/solver core is viable: it supports 14 games with one engine, but only as an app.

### Cited Findings
- FactorioLab: 881★, MIT, TypeScript/Angular, created 2020-03-30, pushed 2026-10-30. Supports 14 games with one calculator: Factorio, Dyson Sphere Program, Satisfactory, Captain of Industry, Techtonica, Final Factory, Factor Y, Foundry, Outworld Station, Belts of Iron, Mindustry, Motemancer, SkyFormer and StarRupture. It "is intended to build on the features of the Kirk McDonald Factorio Calculator". — [GitHub factoriolab](https://github.com/factoriolab/factoriolab)
- FactorioLab's solver dependency is `glpk-ts` (^0.0.11), GLPK compiled for TypeScript, as listed in its package.json. Game data lives in per-game datasets with a schema folder (`src/data/schema`). — [package.json](https://github.com/factoriolab/factoriolab/blob/main/package.json); [src/data](https://github.com/factoriolab/factoriolab/tree/main/src/data)
- An Alt-F4 article by the FactorioLab author says simplex is "the method used by both FactorioLab and the Kirk McDonald Factorio Calculator", after two earlier non-simplex solver iterations. — [Alt-F4 #47](https://alt-f4.blog/ALTF4-47/)
- Kirk McDonald calculator: 754★, Apache-2.0, created 2015-08-24, pushed 2025-07-02. Static JS with in-repo `simplex.js`, `matrix.js` and `rational.js` (its own exact-rational simplex), plus `dump.lua`. Datasets are dumped from the game with a separate `factoriodump` tool in KirkMcDonald/factorio-tools, which also offers an experimental standalone `factoriocalc`. — [GitHub](https://github.com/KirkMcDonald/kirkmcdonald.github.io)
- YAFC-CE (Factorio calculator and analyser that works with mods): 203★, GPL-3.0, C#, pushed 2026-10-31. Original ShadowTheAge/yafc: 201★, last pushed 2024-10-21. Foreman2 (visual Factorio planner): 343★, C#, pushed 2026-08-03. All are desktop apps. — [yafc-ce](https://github.com/Yafc-CE/yafc-ce); [yafc](https://github.com/ShadowTheAge/yafc); [Foreman2](https://github.com/DanielKote/Foreman2)
- SatisfactoryTools (greeny): 538★, TypeScript, pushed 2026-10-16, license NOASSERTION. Its package.json lists no LP-solver dependency, so its solver is in-repo or server-side (unverified). — [GitHub](https://github.com/greeny/SatisfactoryTools)
- Other Satisfactory planners are all apps: satisfactory-factories/application (53★, AGPL-3.0), AnthorNet/SC-ProductionPlanner (51★), adepierre/ficsit-companion (49★, C++, node-based, whose README mentions lpsolve), Scott1903/satisfactory_planner (38★, Python), and Zistack/Satisfactory-Optimizer (scipy `linprog`). satisfactory-dev/Satisfactory-Production-Calculator (2★, Apache-2.0) bills itself as "The guts of the Marv-compatible production planner", a rare attempt at a core. — [satisfactory-factories](https://github.com/satisfactory-factories/application); [ficsit-companion](https://github.com/adepierre/ficsit-companion); [Satisfactory wiki community tools](https://satisfactory.wiki.gg/wiki/Community_tools); [satisfactory-dev calculator](https://github.com/satisfactory-dev/Satisfactory-Production-Calculator)
- doubleaxe/daxfb-calculator ("Calculator/Factory Planner for factory management games", Evospace, Captain of Industry and others): 24★, MIT, pushed 2026-10-26. It is a second multi-game planner. — [GitHub](https://github.com/doubleaxe/daxfb-calculator)
- Factorio's in-game "Factory solver based on linear programming" exists as a mod. Helmod is a well-known in-game planner mod; its GitHub repo was not found at Helmod/Helmod. — [mods.factorio.com factory_solver](https://mods.factorio.com/mod/factory_solver/discussion/671745f1387691b8015f3ca4)
- Generic LP solvers these tools lean on: `javascript-lp-solver` 1.0.3 (about 126.8k weekly), `yalps` 0.6.4 (about 51k weekly; IanManske/YALPS 106★), HiGHS (ERGO-Code/HiGHS 1,903★, MIT; Python `highspy` 1.15.1), COIN-OR Clp (499★), PuLP 4.2.0. — [npm javascript-lp-solver](https://www.npmjs.com/package/javascript-lp-solver); [YALPS](https://github.com/IanManske/YALPS); [HiGHS](https://github.com/ERGO-Code/HiGHS); [Clp](https://github.com/coin-or/Clp); [PuLP](https://pypi.org/project/PuLP/)

### Inferences
- The production-rate math is a solved problem, typically an LP with a generic solver. What has never been packaged is the **domain layer** above the solver: items, recipes, machines, rates, modules and multi-output handling. Every planner re-implements it in-app. FactorioLab running 14 games on one engine is the strongest evidence that a game-agnostic recipe-graph model is feasible.
- For Rust, `good_lp`, `microlp` and HiGHS bindings would cover the solver. A domain layer on top would be new. This is inferred, and those Rust LP crates were not surveyed here.

### Gaps
- I did not confirm SatisfactoryTools' solver implementation, and did not open FactorioLab's schema files to describe its data model in detail.

## Data-format standards: is there a common item/recipe interchange format?

### Takeaway
There is **no cross-game standard**. What exists is per-ecosystem de facto formats: Factorio's official `--dump-data` JSON, Satisfactory's shipped `Docs.json`, PrismarineJS minecraft-data, and the JEI API inside Minecraft modding. FactorioLab's internal per-game dataset schema is the closest thing to a multi-game format, and it is app-internal.

### Cited Findings
- Factorio added a `--dump-data` command-line option (reported as 1.1.77) that writes raw prototype data to `script-output/data-raw-dump.json` without launching the game. Factorio staff say the dump flags are mutually exclusive by design. — [Factorio forum](https://forums.factorio.com/viewtopic.php?p=658340); [Factorio forum, data-dump mode](https://forums.factorio.com/viewtopic.php?p=449096)
- Rust typings for Factorio's machine-readable prototype JSON support "version 6 of the JSON format". — [docs.rs factorio-prototypes-json](https://docs.rs/crate/factorio-prototypes-json/latest)
- Satisfactory ships a `Docs.json`, which community tools type and consume (`@satisfactory-dev/docs.json.ts`). — [npm](https://www.npmjs.com/package/@satisfactory-dev/docs.json.ts)
- PrismarineJS minecraft-data is a "Language independent module providing minecraft data for minecraft clients, servers and libraries": 957★, created 2015, pushed 2026-10-29. It is consumed by JS, Python (`minecraft-data`) and Rust (`mcdata-rs`) wrappers. This is the clearest example of a cross-*language* (not cross-game) item/recipe data standard. — [GitHub](https://github.com/PrismarineJS/minecraft-data); [crates.io mcdata-rs](https://crates.io/crates/mcdata-rs)
- JEI (Just Enough Items): 1,052★, MIT, active since 2014, pushed 2026-11-01. Mods register recipe categories and catalysts through its plugin API. The API became a de facto standard: EMI loads JEI plugins via its built-in JEMI layer, and bridges like TooManyRecipeViewers map JEI API calls onto EMI's API. REI (RoughlyEnoughItems) has 464★. — [JEI GitHub](https://github.com/mezz/JustEnoughItems); [EMI on Modrinth](https://modrinth.com:443/mod/emi); [TMRV](https://www.mado-hosting.com/minecraft-mods/tmrv); [REI](https://github.com/shedaniel/RoughlyEnoughItems); [JEI wiki](https://github.com/mezz/JustEnoughItems/wiki/Recipe-Category-Crafting-Items)
- KubeJS exposes `RecipeViewerEvents` that abstract over the recipe viewers. — [KubeJS wiki](https://kubejs.com/wiki/events/RecipeViewerEvents)

### Inferences
- Both standards that took hold (JEI API, minecraft-data) emerged from **one huge modding ecosystem with many independent producers and consumers** of recipe data. Without that pressure, every game invents its own schema.
- A Rust crate could borrow the generalisable shape of these formats: items, recipes with typed input/output stacks, crafting stations or "catalysts", categories and tags, and multi-output recipes. No existing schema is a ready-made cross-game standard to adopt.

### Gaps
- I found no JSON Schema or other published spec for a cross-game recipe format, and nothing resembling an IETF/W3C-style effort.

## Overall: established category or gap? What made adjacent categories succeed?

### Takeaway
A "data-driven crafting rules engine as a standalone library" is **not an established category in any ecosystem**. It is a real gap, but a gap with a history of small, abandoned attempts, which suggests the hard part is design and adoption rather than code. The adjacent categories that did succeed engine-agnostically (dialogue, ECS, behavior trees) share a few traits: a portable core plus thin per-engine integrations, an authoring tool or language, and in several cases a shipped commercial game or company behind them.

### Cited Findings
- Yarn Spinner: the core repo is "The core compiler and engine-agnostic components for Yarn Spinner" (2,868★, MIT, created 2015, pushed 2026-10-31). The core compiler and runtime are C# and "None of it is Unity specific". In July 2026 every Unity sample had a Godot GDScript equivalent, and the team reports the GDScript port is the most-used non-Unity version (self-reported). Unreal is in alpha as "native integration ... not just a Unity port". The 2026 roadmap commits the core to staying free and open source. — [GitHub YarnSpinner](https://github.com/YarnSpinnerTool/YarnSpinner); [Darner's Digest 2022](https://www.blog.radiator.debacle.us/2022/01/darners-digest-vol-3-on-yarn-spinner.html); [Monthly Jul '26](https://www.yarnspinner.dev/blog/monthly_jul_26/); [Monthly Jun '26](https://www.yarnspinner.dev/blog/monthly_jun_26); [Yarn Spinner in 2026](https://yarnspinner.dev/blog/yarn-spinner-in-2026/); [other engines docs](https://docs.yarnspinner.dev/yarn-spinner-for-other-engines)
- ink (inkle): 4,999★, MIT, pushed 2026-05-05. It has a language plus the Inky editor, a Unity integration that "includes the full Ink engine source", and a community-maintained JS port (inkjs) that is "usually one major version behind". — [GitHub ink](https://github.com/inkle/ink)
- ECS: EnTT 13,237★ (MIT, C++) and flecs 8,784★ (C/C++), both pushed 2026-11-01. — [EnTT](https://github.com/skypjack/entt); [flecs](https://github.com/SanderMertens/flecs)
- Behavior trees: BehaviorTree.CPP ("Batteries included") has 4,281★, MIT, pushed 2026-10-26. — [GitHub](https://github.com/BehaviorTree/BehaviorTree.CPP)
- Engine-bound comparison: Dialogic (Godot dialogue) has 6,137★, and GLoot (Godot inventory) has 990★. — [Dialogic](https://github.com/dialogic-godot/dialogic); [GLoot](https://github.com/peter-kish/gloot)
- Game-data authoring, an adjacent established category: CastleDB, 611★, active. — [CastleDB](https://github.com/ncannasse/castle)

### Inferences
- Why dialogue and ECS succeeded standalone while crafting did not (my reasoning, not sourced):
  1. **A content language or authoring tool**: ink, Yarn, Inky and the Yarn editors give writers something engine-independent to author in. Crafting data is usually just a spreadsheet or JSON, so a library adds little over a struct.
  2. **The domain is uniform across games**: branching dialogue and ECS look the same everywhere. Crafting rules vary enormously (grid-shaped vs. shapeless, quality, timers, stations, rates, unlocks, spoilage), so a generic library is either too thin to matter or too opinionated to fit.
  3. **Shipped-game provenance**: ink came from inkle's own commercial games, and BehaviorTree.CPP and EnTT came out of real robotics and game production.
  4. **A portable core plus thin adapters** (Yarn Spinner's Unity, Godot and Unreal pattern).
- What could make a Rust crafting crate fill the gap: focus on the parts that *are* uniform and hard. Candidates are (a) a typed item/recipe/catalog data model with serde and validation, (b) a persistent recipe-book/discovery and unlock graph, (c) a production-rate solver over the recipe graph (the FactorioLab problem, which nobody has packaged), and (d) resource nodes with replenishment policies. Leave inventories and UI to hosts. Pair it with an authoring or validation tool to copy the ink and Yarn success factor.
- Risk signal: many dead or near-zero-adoption generic attempts across npm, NuGet, crates.io and GitHub suggest that demand for *generic* crafting libraries is weak. Teams treat crafting as game-specific design. A Rust crate would likely succeed as a production-chain or recipe-graph toolkit (the planner niche, where 14-game FactorioLab shows commonality) more than as a "generic crafting system".

### Gaps
- No usage surveys quantify how many games roll their own crafting systems vs. adopt a library. The "weak demand" inference rests on download and star counts alone.
- I did not survey Rust LP crates (`good_lp`, `microlp`, `highs`), which a Rust production solver would build on.
