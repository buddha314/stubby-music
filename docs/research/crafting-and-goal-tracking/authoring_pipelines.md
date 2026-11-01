# Authoring pipelines: getting gameplay rules from designers to engineers

Research date: 2026-11-01. Budget-limited pass (~16 tool calls). Several primary pages fetched did not contain what their titles promised; those gaps are recorded explicitly. Claims without a source are under Inferences or Gaps, never Cited Findings.

## 1. Spreadsheet-driven pipelines (Sheets/Excel -> CSV/JSON -> game; CastleDB; GDC practice)

### Takeaway
Spreadsheets remain a mainstream front end for rules data, and current practice (GDC 2026) still includes building game data directly from Google Sheets. Engine vendors document the spreadsheet → CSV → engine-table round trip as a sanctioned workflow. The weak points are drift between the sheet and the build, and merging. CastleDB-style typed, JSON-backed "static databases" exist to keep the sheet ergonomics while making the data diffable in git.

### Cited Findings
- Unreal's official docs describe designers keeping data in Excel `.xlsm` workbooks whose macro buttons export `.csv`, which is then imported as a Data Table. The docs say "These documents reside in a single location to help make data easier to find and modify". Once reimported, changes apply without code changes. — [Unreal: Data Driven Gameplay Elements](https://dev.epicgames.com/documentation/en-us/unreal-engine/data-driven-gameplay-elements-in-unreal-engine)
- In that Unreal workflow a programmer defines the row schema first: "a programmer has to create the row container telling the engine how to interpret the data." The first CSV column must be `Name`, the lookup key. Import options `Ignore Extra Fields` and `Ignore Missing Fields` control whether a mismatch between schema and sheet raises a warning. **Reimport** reuses the original source file path. — [Unreal docs](https://dev.epicgames.com/documentation/en-us/unreal-engine/data-driven-gameplay-elements-in-unreal-engine)
- GDC 2026 "Spreadsheets Microtalks" (Game & Production Technology track) included these practitioner talks:
  - Tomo Moriwaki: Epic Tavern "built game data directly from the sheets", collaborating on story in Google Sheets.
  - Guillaume Pierre (Magic Fuel Games/Playstack): an economy spreadsheet that simulates the procedurally generated loot, rooms, outposts and districts of *Darkwater*.
  - Mike Rossmassler (Turtle Rock): spreadsheet-guided paper prototypes.
  - Cheryl Platz: converting Figma to implementation via Sheets.
  — [GDC Vault: Spreadsheets Microtalks (2026)](https://gdcvault.com/play/1035719/Spreadsheets); [GDC schedule entry](https://schedule.gdconf.com/session/spreadsheets-microtalks/913682)
- Other GDC sessions in the search results: a Tools Summit talk on a level-design tool built inside spreadsheets for *Full Bloom* ("quick iteration on hundreds of levels"); a team that built *Merchant Guilds* content in spreadsheets with Apps Script for direct import; Tyler Coleman's IGS "Spreadsheet Smarter" with templates. These come from search snippets; I did not open each page and their years are unverified. — [Tools Summit (1035550)](https://gdcvault.com/play/1035550/Tools-Summit-A-Series-of); [Tools Summit (1035299)](https://gdcvault.com/play/1035299/Tools-Summit-A-Series-of); [Spreadsheet Smarter](https://gdcvault.com/play/1027128/Independent-Games-Summit-Spreadsheet-Smarter)
- Slay the Spire's balance was metrics-driven, using community feedback and play data alongside design (GDC 2019, Anthony Giovannetti). — [GDC news](https://gdconf.com/news/learn-slay-spires-successful-metrics-driven-approach-game-balancing-gdc-2019)
- A 2015 deck (GDS 2015, not GDC) says spreadsheet balancing suits content-heavy games and games-as-a-service, and suits small games less. — [SlideShare: GDS2015 spreadsheet design](https://www.slideshare.net/slideshow/gds2015-spreadsheetdesign/55773761)
- CastleDB:
  - It is a "structured static database" by Nicolas Cannasse (Haxe creator, Shiro Games), from about 2012.
  - It stores its data model (schema) and its rows as human-readable JSON, so Git/SVN can diff and merge it.
  - Editing is spreadsheet-like, with typed columns.
  - A later rewrite integrates it into the HIDE editor (Heaps).
  — [lib.haxe.org: castle](https://lib.haxe.org/p/castle); [dbdb.io: CastleDB](https://dbdb.io/db/castledb/revisions/3); [gamefromscratch](https://gamefromscratch.com/?p=6895); [KnightNine/castle fork README](https://github.com/KnightNine/castle)

### Inferences
- The Unreal workflow is the canonical split of ownership: **engineers own the row struct (schema), designers own the rows (data)**. A schema/data mismatch is surfaced at import time, and the "ignore" flags decide whether it warns or is silent. A small team should not copy the silent option: unknown or missing fields should be errors.
- Wherever the sheet is the source and the engine asset is a derived copy, there are two copies that can drift. A reimport only happens if someone does it, and nothing proves the build matches the sheet. CastleDB's design answers this by making the editable artifact itself the JSON in version control.

### Gaps
- CastleDB's use on *Dead Cells* (Motion Twin) is widely repeated, but I found **no primary source** in this pass. The castledb.org site refused connections, and searches tied it only to Shiro Games/Haxe. Treat "Dead Cells used CastleDB" as unverified.
- I found no primary source on Riot's or Supercell's internal balance-sheet or live-ops data practice, and none on "Depot" (the Unity/Godot sheet tool) or Unity Google Sheets importers.
- The bugnet.io "balancing with spreadsheets" article returned only a blog index, so its claims about drift are unverified.

## 2. Engine-native data authoring (Unreal, Unity, Godot, Satisfactory, Factorio, Paradox, Minecraft, RimWorld, Dwarf Fortress)

### Takeaway
Every long-lived systemic game in this survey separates **rules data** from the code that interprets it, though in different forms:

| Form | Examples |
| ---- | -------- |
| Engine assets | Unreal Data Tables, Satisfactory `FGRecipe`/`FGSchematic`, Unity ScriptableObjects |
| Declarative text | RimWorld XML Defs, Minecraft data-pack JSON, Dwarf Fortress raws, Paradox script |
| Staged, executable data definition | Factorio's Lua data stage |

Recipes and unlocks are nearly always separate objects: a recipe says what is made, and a schematic, advancement or research says when the player gets it.

### Cited Findings
- **Satisfactory** (Coffee Stain, Unreal), recipe assets:
  - A recipe is a Blueprint class with parent `FGRecipe`, named `Recipe_*`.
  - **Ingredients** are "An array of structs, each of which contains the information for one crafting ingredient". **Product** is a separate array.
  - **Produced In** is "An array of IFGRecipeProducerInterface implementers (machines) that are allowed to use the recipe."
  - **Manufacturing Duration** is in seconds, and a **Manual Manufacturing Multiplier** scales craft-bench time.
  - The docs warn that a quantity above an item's stack size means the machine never runs: a semantic constraint the data format does not enforce.
  — [ficsit.app: Recipe vs Schematic](https://docs.ficsit.app/satisfactory-modding/latest/Development/BeginnersGuide/SimpleMod/recipe.html)
- **Satisfactory** schematics (unlocks):
  - `FGSchematic` assets (`Schematic_*`) carry Type (`Milestone`), Cost (an item array), and **Unlocks**: "the rewards the player will get when purchasing this schematic", for example a `BP Unlock Recipe` that references a recipe asset.
  - They also carry Tech Tier, Menu Priority, Time to Complete, and `Include In Builds`.
  - A schematic is invisible until it is **registered** in the Game World Module's `M Schematics` array, and that module must be the Root Module.
  — [ficsit.app recipe page](https://docs.ficsit.app/satisfactory-modding/latest/Development/BeginnersGuide/SimpleMod/recipe.html)
- **Satisfactory** recipes are also the build-gun cost mechanism ("recipes are used to define both crafting recipes... and construction prices when using the build gun"). The community ContentLib mod lets recipes be written as **JSON** that uses internal item names, with one patch file per modified asset. — [ficsit.app Crafting (2.1.1)](https://docs.ficsit.app/satisfactory-modding/2.1.1/Development/Satisfactory/Crafting.html); [ContentLib: Recipes](https://docs.ficsit.app/contentlib/latest/Features/Recipes.html); [ContentLib: Patching](https://docs.ficsit.app/contentlib/latest/Features/Patching.html)
- **Factorio** data lifecycle:
  - It has distinct stages: settings, then data/prototype, then control (runtime). The data stage runs `data.lua`, `data-updates.lua`, `data-final-fixes.lua` in rounds across all mods. Each round runs every mod's file before the next round starts, so a mod can modify another's prototypes without depending on its load order.
  - The Lua state is discarded after each stage. At runtime "prototypes can no longer be modified, and the global `data` variable is no longer accessible."
  - Load-time validation: "Missing properties will either fall back to a default value or give an error that indicates what is missing." Unrecognized properties are **ignored**.
  — [Factorio Lua API: Data lifecycle](https://lua-api.factorio.com/latest/auxiliary/data-lifecycle.html)
- **Factorio migrations**:
  - Migrations run once per save, in the control stage.
  - "saves will remember the file names of the migrations that have already been applied to it."
  - JSON migrations run first, typically to rename prototypes. Lua migrations follow, with full game-state access.
  — [Factorio data lifecycle](https://lua-api.factorio.com/latest/auxiliary/data-lifecycle.html)
- **Minecraft data packs**:
  - The path is the ID. A file at `data/<namespace>/<registry>/<path>.json` loads into that registry as `<namespace>:<path>`, and "The data is organized into namespaces to avoid files from different packs unintentionally interfering with each other."
  - Recipes, advancements, loot tables, tags and predicates are JSON.
  - Tags merge across packs unless they set `"replace": true`.
  - `pack.mcmeta` declares a format range (`min_format`/`max_format`). Since 1.21.9 there are minor format versions for non-breaking changes.
  — [Minecraft Wiki: Data pack](https://minecraft.wiki/w/Data_pack)
- **Minecraft hot reload and failure**:
  - `/reload` hot-reloads tags, loot tables, recipes, advancements, functions and predicates.
  - "If data pack reload fails, changes are not applied and the game continues using previous data."
  - A broken pack shows an error on world open and offers Safe Mode (vanilla only).
  — [Minecraft Wiki: Data pack](https://minecraft.wiki/w/Data_pack)
- **Minecraft datagen**: on the modding side, Fabric's data generation is "an API for programmatically generating recipes, advancements, tags, item models". Code-side providers write JSON into `src/main/generated` via `./gradlew runDatagen`, so the shipped JSON is a build artifact of typed code. — [Fabric docs: Data generation setup](https://docs.fabricmc.net/develop/data-generation/setup)
- **RimWorld XML Defs**:
  - New content (weapons, apparel, animals, plants) is added as XML "Defs" in a mod's `Defs` folder. File paths "do not matter except for LoadFolders". Vanilla Defs in `Data/` are the intended templates.
  — [RimWorld Wiki: Defs](https://rimworldwiki.com/wiki/Modding_Tutorials/Defs)
- **RimWorld Def mechanics**:
  - Each `<Def>` tag must match a C# type.
  - Inheritance uses `Name`/`ParentName`, and `Abstract="True"` parents are discarded after inheritance. Lists **append** to the parent while scalar tags overwrite, and `Inherit="False"` opts out.
  - Failure modes: an incomplete parent "will crash the game"; keyword case matters ("abstract is NOT the same as Abstract"); redefining core abstracts "is common cause for mod compatibility issues."
  — [RimWorld Wiki: XML file structure](https://rimworldwiki.com/wiki/Modding_Tutorials/XML_file_structure)
- **Dwarf Fortress raws**:
  - Raws are plain-text `[TOKEN:VALUES]` files (e.g. `creature_*`) with templated `CREATURE_VARIATION` and per-caste overrides.
  - Load order is set by each file's first line, not its filename, which matters for `[COPY_TAGS_FROM]`.
  - Duplicated raw files "can cause strange, difficult-to-diagnose problems, and even crashes".
  — [DF Wiki: Raw](https://dwarffortresswiki.org/index.php/DF2014:Raw)
- **Unity ScriptableObjects**:
  - They are project assets "independent of GameObjects", and their "main value... is as a data store". Shared instances mean "there is one copy of the data in memory".
  - Script edits in Edit mode need `EditorUtility.SetDirty` or "changes may not persist". In a standalone build "you can only read saved data".
  — [Unity Manual: ScriptableObject](https://docs.unity.com/en-us/engine/current/manual/scripting/get-started/fundamental-unity-types/class-scriptable-object)
- **Paradox (Clausewitz)**:
  - Script files under `common/` are validated by the community VS Code extension **CWTools**, which gives live validation. One of its roadmap items is checking that mod-overridden defines exist in vanilla.
  - The engine reports problems to `error.log` with messages such as `Malformed token: 95, near line: 4`.
  — [Paradox forum: CWTools](https://admin-forum.paradoxplaza.com/forum/threads/tool-cwtools-a-mod-validating-extension-for-vs-code.1066033); [Paradox forum: malformed token example](https://admin-forum.paradoxplaza.com/forum/threads/modding-quick-questions.589686/post-22328223)

### Inferences
- **Recipe ≠ unlock.** Satisfactory (recipe vs schematic) and Minecraft (recipe vs advancement) both keep "what can be made" separate from "when the player gets it", and the unlock references the recipe by ID. For stubby-music this maps to: the recipe-book entry (discovery) and the recipe definition stay distinct objects in `catalog.json`. That is consistent with AGENTS.md's rule that learning must not disappear when inventory reaches zero.
- **Explicit registration is a known footgun.** Satisfactory's "data exists but is invisible until registered" is a failure mode worth designing out. Prefer discovery by file or catalog presence, plus a validator that flags any unlock pointing at nothing, or any recipe that no unlock references.
- **Factorio's leniency costs typo detection.** Ignoring unknown properties suits a mod ecosystem but hides typos (e.g. `ingrediants`). A single-author game should reject unknown fields, as serde's `deny_unknown_fields` does.
- **Datagen** (typed code generating JSON) is the middle path when hand-written JSON grows repetitive. The JSON stays the reviewed, diffable artifact, but its source is type-checked.
- **Inheritance and templates** (RimWorld abstracts, DF variations) cut repetition, but they introduce order and merge semantics (append vs overwrite) that are a frequent source of bugs. Defer them until repetition actually hurts.

### Gaps
- Unreal Gameplay Tags, Primary Asset Labels, Data Assets and the Asset Manager were not fetched in this pass.
- Godot Resources were not fetched.
- RimWorld's `defName` cross-reference resolution and its load-time "Config error" reporting, and the XPath `Patches` system: the pages fetched were placeholders. The behaviour is well known in the community but not cited here.
- Path of Exile, Warframe and Stardew Valley data structure: not researched in this pass.
- Factorio's `data.raw` itself is documented on the community wiki, not on the fetched page.

## 3. Schema and validation conventions (typed schemas, referential integrity, CI, hot reload, IDs, migrations)

### Takeaway
The cited systems converge on five conventions:

1. **A stable, namespaced string ID that is separate from the display name**: Minecraft `namespace:path`, Unreal's `Name` column, RimWorld/Satisfactory internal names.
2. **A schema owned by code**: Unreal row structs, RimWorld C# Def classes, Unity SO classes.
3. **Validation at load, with the line or field named.**
4. **Atomic hot reload** that keeps the old data if the new data fails (Minecraft).
5. **Explicit, once-only, rename-first save migrations** (Factorio).

### Cited Findings
- **IDs:**
  - Minecraft derives the ID from the file path and namespaces it to avoid collisions. — [Minecraft Wiki](https://minecraft.wiki/w/Data_pack)
  - Unreal requires a `Name` key column used for in-game lookup. — [Unreal docs](https://dev.epicgames.com/documentation/en-us/unreal-engine/data-driven-gameplay-elements-in-unreal-engine)
  - ContentLib's recipe file name becomes the internal recipe name. — [search summary of ContentLib docs](https://docs.ficsit.app/contentlib/latest/Tutorials/CreateRecipe.html)
- **Load validation:**
  - Factorio errors on missing required properties, naming what is missing, and ignores unknown ones. — [Factorio](https://lua-api.factorio.com/latest/auxiliary/data-lifecycle.html)
  - Unreal warns on extra or missing CSV fields unless told to ignore them. — [Unreal](https://dev.epicgames.com/documentation/en-us/unreal-engine/data-driven-gameplay-elements-in-unreal-engine)
  - Paradox writes `error.log` entries with line numbers, and CWTools moves that validation into the editor. — [Paradox forum](https://admin-forum.paradoxplaza.com/forum/threads/tool-cwtools-a-mod-validating-extension-for-vs-code.1066033)
- **Safe hot reload:** Minecraft `/reload` keeps the previous data on failure. Some data types still need a restart. — [Minecraft Wiki](https://minecraft.wiki/w/Data_pack)
- **Versioning:** Minecraft's `pack_format` (min/max, with minor versions for non-breaking changes) is an explicit data-schema version. — [Minecraft Wiki](https://minecraft.wiki/w/Data_pack)
- **Migrations:** Factorio saves record which migration files have been applied. JSON rename migrations run before Lua migrations. — [Factorio](https://lua-api.factorio.com/latest/auxiliary/data-lifecycle.html)
- **Semantic constraints the format cannot express:** a quantity above stack size silently stalls a Satisfactory machine. — [ficsit.app](https://docs.ficsit.app/satisfactory-modding/latest/Development/BeginnersGuide/SimpleMod/recipe.html)

### Inferences
- Validation has two layers:
  - **Shape**: types, required and unknown fields. A typed schema handles this (serde structs, optionally JSON Schema for editor autocomplete).
  - **Semantics**: every referenced item ID exists; quantity ≤ stack size; every unlock target exists; no unreachable recipes; no cycles where forbidden. Only the second layer catches the Satisfactory-style bugs. For stubby-music, both belong in the Bevy-free `stubby_recipes` crate, run in `cargo test` (CI) and at game load, with errors naming the file, entry ID and field.
- **Treat IDs as save-game API.** Renaming a `square_wave`-style ID needs a migration table, as Factorio's JSON rename migrations show. Display names and localization keys can change freely.
- **Adopt the Minecraft rule: on a failed reload, keep the old data and report.** This fits Bevy asset hot-reload for `catalog.json`.

### Gaps
- No studio-level source was found for CI checks on data (e.g. a blog describing data validation in a build pipeline), localization-key conventions, or golden and simulation tests for balance.

## 4. Process conventions (ownership, data-as-spec, PR review, content-only patches, tooling budget)

### Takeaway
The sourced pattern is consistent: **engineers own the schema and the interpreter; designers own the rows**. Data lives where it can be diffed (text, or CSV/JSON exports). Teams that cannot merge (binary assets) split files by function and lock. Live games that expose their data (Factorio, Minecraft, RimWorld, Satisfactory) let content ship and change without engine code changes.

### Cited Findings
- **Ownership:**
  - In Unreal, a programmer creates the row container, and designers then iterate in Excel and reimport "without code changes". — [Unreal docs](https://dev.epicgames.com/documentation/en-us/unreal-engine/data-driven-gameplay-elements-in-unreal-engine)
  - RimWorld Defs must map to C# types (code-owned schema) while content is XML. — [RimWorld XML file structure](https://rimworldwiki.com/wiki/Modding_Tutorials/XML_file_structure)
- **Locking vs merging:**
  - Alex Kurilin's "Why Game Devs Don't Merge Files" (reported June 2026) explains that game teams **lock** binary assets because they cannot be merged. Conflicts are "avoided through file design and team communication".
  - He recommends **splitting monolithic files by function** (e.g. Unreal Actor Components) to reduce contention.
  - Blueprint diffs show changes but merging still needs manual reconciliation.
  — [GIGAZINE summary of kuril.in post](https://www.gigazine.net/gsc_news/en/20260619-game-dev-dont-merge-files)
- **Merge tools are limited even for text:** Unity's UnityYAMLMerge cannot resolve conflicts where two people edited the same object. — [search summary: Diversion blog](https://diversion.dev/blog/resolving-conflicts-in-game-development-projects); [gist on UnityYAMLMerge](https://gist.github.com/maxahn/3b4c78d276d5f409b69da6e8610b2ef2)
- **Visual diffing:** Ink & Switch's Backstitch argues git's line diff loses the structure of scene files, and shows changes in the editor viewport instead. — [Ink & Switch Backstitch notebook](https://www.inkandswitch.com/project/backstitch/notebook/)
- **Spreadsheets as the collaboration surface across roles** (narrative, economy, UX): GDC 2026 microtalks. — [GDC Vault](https://gdcvault.com/play/1035719/Spreadsheets)

### Inferences
- **Split data files by domain** rather than keeping one monolithic `catalog.json`: items, resources, recipes, unlocks/recipe-book, and mines/replenishment. This reduces merge conflicts between parallel agents, following the Kurilin file-splitting advice applied to text. A loader can concatenate the files and validate cross-file references.
- **"The data is the spec" and "the spec is the requirement" can coexist.**
  - OpenSpec SHALL/WHEN-THEN scenarios state the *rule types and invariants*, e.g. "a recipe SHALL reference only known item IDs" and "learning SHALL persist at zero inventory".
  - The JSON holds the *values*.
  - Engineers (agents) change code only when a new *rule type* is needed. A designer changing a number or adding a recipe of an existing type is a data-only PR with no spec change.
- **Tooling budget:** large studios build custom editors (Unreal and Unity inspectors, CastleDB/HIDE). A 1–3 person team gets most of the value from the following, and should defer custom editors until the data volume makes hand-editing JSON the bottleneck:
  - text JSON/RON in git;
  - a JSON Schema for editor autocomplete and inline errors (VS Code supports `$schema`), analogous to CWTools;
  - a validator with good messages;
  - hot reload.
  This last point is inference; no source on indie tooling budgets was found.

### Gaps
- No primary sources were found for:
  - live-ops content-only patch practice (Riot, Supercell, Warframe, Path of Exile);
  - "design doc vs data as spec" debates in studio postmortems;
  - formal PR review of data changes at large studios.

## 5. Trade-offs and failure modes; small-team fit

### Takeaway
The documented failure modes are:

- **silent acceptance of bad data**: ignored unknown fields; "ignore missing fields" import flags;
- **semantically invalid but well-formed data**: stack-size stalls;
- **load-order and inheritance surprises**: RimWorld abstracts, DF first-line load order, duplicated raws;
- **invisible unregistered content**: Satisfactory schematics;
- **unmergeable or contended files.**

For a one-designer plus AI-agent team, the strongest-evidenced design is Minecraft/Factorio-like:

- namespaced stable IDs;
- text data in git, split per domain;
- a strict code-owned schema;
- a semantic validator run in tests and at load;
- atomic hot reload;
- explicit ID migrations.

### Cited Findings
- **Duplicated raw definitions** cause hard-to-diagnose problems and crashes (Dwarf Fortress). — [DF Wiki](https://dwarffortresswiki.org/index.php/DF2014:Raw)
- **Inheritance pitfalls** (RimWorld): an incomplete parent crashes the game; a case error in `Abstract` breaks things; overriding core abstracts breaks compatibility. — [RimWorld XML file structure](https://rimworldwiki.com/wiki/Modding_Tutorials/XML_file_structure)
- **Unregistered schematics** never appear in Satisfactory, and over-stack quantities stall machines forever. — [ficsit.app](https://docs.ficsit.app/satisfactory-modding/latest/Development/BeginnersGuide/SimpleMod/recipe.html)
- **Unknown prototype properties** are silently ignored by Factorio. — [Factorio](https://lua-api.factorio.com/latest/auxiliary/data-lifecycle.html)
- **Logic in data:** Factorio's data stage is executable Lua, while its runtime is locked, so code-as-data is confined to load time. — [Factorio](https://lua-api.factorio.com/latest/auxiliary/data-lifecycle.html)
- **Data generated from typed code:** Fabric datagen generates JSON rather than hand-writing it. — [Fabric](https://docs.fabricmc.net/develop/data-generation/setup)
- **Spreadsheet economy simulation** as a balance test before data ships (*Darkwater*). — [GDC Vault 2026](https://gdcvault.com/play/1035719/Spreadsheets)

### Inferences
- **Logic creeping into data** shows up in two forms:
  - Factorio confines it to a load stage.
  - Paradox script and Minecraft functions embed condition and effect logic in data.
  For stubby-music, keep data **declarative** (quantities, IDs, enums of known rule types), and add a new rule *type* via an OpenSpec change plus Rust code. Do not let a mini-language grow in `catalog.json`.
- **"Designers need engineers for every new rule type"** is the accepted cost of a code-owned schema. With AI agents as engineers that cost is low, so a strict schema is cheaper here than at a studio.
- **Tests worth adopting**, by analogy to the cited validators and the *Darkwater* simulation sheet:
  1. A validator test over the real shipped catalog.
  2. Golden tests for a few canonical recipes, e.g. a lightning bug yields 2 `square_wave`.
  3. A small deterministic economy simulation: given seed X and N nights, assert production rates stay within bounds.
  The project's seeded-determinism rule makes the third practical.
- **Mitigating merge conflicts** for JSON in git: one entry per object, sorted keys, stable formatting (enforce with a formatter check in CI), and per-domain files.

### Gaps
- No sourced studio examples of:
  - golden or balance simulation tests run in CI;
  - spreadsheet-to-build drift incidents;
  - quantified merge-conflict costs on data files.
- Indie-specific guidance (1–3 people) on schema and validators was not found. The recommendations above are inference from the larger-system evidence.
