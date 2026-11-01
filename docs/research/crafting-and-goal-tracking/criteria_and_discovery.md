# Criteria/trigger-based progression and discovery-based unlocks in crafting and survival games

Research date: 2026-11-01. Minecraft data checked against vanilla data at **26.4 Snapshot 6** (data_version 5126, data pack format 125.1, built 2026-10-27) via misode/mcmeta. Valheim save layout checked against Kakoen/valheim-save-tools (PlayerData version 26, the last version that tool has tested). Several primary sources could not be reached: stardewvalleywiki.com returned Cloudflare 403 to every request, and minecraft.wiki's advancement-storage subpage returned 404. Those gaps are marked below.

## Minecraft advancements: data format, triggers, evaluation, per-player save, recipe-book unlocks, and statistics

### Takeaway
Minecraft advancements are a tree of data-pack JSON definitions. Each one has named **criteria** (a trigger plus conditions) and a **requirements** list that reads as AND-of-ORs. The game evaluates them event-driven: each trigger class keeps a map from player to listeners and checks only the criteria that player has not yet completed. Recipe unlocks are themselves advancements whose reward is a recipe. Statistics are a separate system of monotonic counters with no tree, no rewards, and no completion state.

### Cited Findings
**Definition format (Java Edition)**
- Root fields: `parent` (optional; without it the advancement is a tree root; circular references fail to load), `display` (optional; needs `title`, `description`, `icon`), `criteria` (named map, each entry `{trigger, conditions?}`), `requirements` (optional list of lists), `rewards` (optional), and `sends_telemetry_event` (default false). — [minecraft.wiki: Advancement definition](https://minecraft.wiki/w/Advancement_definition)
- How `requirements` works: "the advancement is granted when every inner list has at least one completed criterion", so it is an AND of ORs. By default every criterion is required, and an empty inner list makes the advancement unobtainable without commands. — [minecraft.wiki: Advancement definition](https://minecraft.wiki/w/Advancement_definition)
- `rewards`: `experience` (int), `recipes` (list of recipe IDs), `loot` (list of loot-table IDs), and `function` (a single function ID; function tags are not allowed). — [minecraft.wiki: Advancement definition](https://minecraft.wiki/w/Advancement_definition)
- `display` options: `frame` is one of `task`, `goal` or `challenge`; `background` applies to roots only (the tab background); `show_toast` and `announce_to_chat` default to true; `hidden` defaults to false. — [minecraft.wiki: Advancement definition](https://minecraft.wiki/w/Advancement_definition)
- Location: `data/<namespace>/advancement/<name>.json`, inside data packs. — [minecraft.wiki: Advancement/JSON format](https://minecraft.wiki/w/Advancement/JSON_format)
- Selected triggers:
  - `inventory_changed`: fires when inventory slots change (not the cursor or crafting slots); can test added items and slot counts.
  - `recipe_crafted`: fires on crafting at a crafting table, stonecutter or smithing table; can test the recipe and its ingredients.
  - `recipe_unlocked`: fires when a recipe is unlocked.
  - `item_used_on_block`, `consume_item`, `placed_block`: fire on the matching player action.
  - `impossible`: never fires; the advancement can be granted only by command.
  - The wiki also lists temporary snapshot triggers (23w13a_or_b, 24w14potato, 25w14craftmine, 26w14a) and removed ones such as `player_damaged`.

  — [minecraft.wiki: Advancement definition](https://minecraft.wiki/w/Advancement_definition)
- Real vanilla example, `story/smelt_iron` in 26.4-snapshot-6: one criterion `iron` = `minecraft:inventory_changed` with `items:[{items:"minecraft:iron_ingot"}]`, `requirements:[["iron"]]`, display text given as translation keys, and `sends_telemetry_event: true`. — [misode/mcmeta data branch](https://github.com/misode/mcmeta/tree/data/data/minecraft/advancement)

**History**
- Advancements replaced achievements in 17w13a (1.12).
- 17w18a: "Advancement loading is now strict JSON".
- 1.13 (17w43a): custom advancements moved into data packs.
- 1.12-pre5: all recipe-unlock advancements got the parent `recipes/root`.
- 1.16-pre1: renaming an advancement file meant "progress in this advancement is not kept". Progress is keyed by advancement ID.
- 1.20 (23w16a): the `recipe_crafted` trigger was added.

  — [minecraft.wiki: Advancement](https://minecraft.wiki/w/Advancement)
- Advancements are "obtained and saved per world" and are independent: a child can be earned without its parent. `/advancement` grants and revokes them. A hidden advancement stays invisible until it is earned, even if its children are earned. — [minecraft.wiki: Advancement](https://minecraft.wiki/w/Advancement)

**How the game evaluates them (Mojang mappings)**
- `SimpleCriterionTrigger<T extends SimpleInstance> implements CriterionTrigger<T>` has a private map from `PlayerAdvancements` to a set of `CriterionTrigger.Listener<T>`. It exposes `addPlayerListener`, `removePlayerListener` and `removePlayerListeners(PlayerAdvancements)`. Subclasses call the protected `trigger(ServerPlayer, Predicate<T>)`, which tests the matching listeners' condition instances. In 1.21.11 the package was renamed from `advancements.critereon` to `advancements.criterion`. — [mappings.dev 1.21.11 SimpleCriterionTrigger](https://mappings.dev/1.21.11/net/minecraft/advancements/criterion/SimpleCriterionTrigger.html); [mappings.dev 1.21.4](https://mappings.dev/1.21.4/net/minecraft/advancements/critereon/SimpleCriterionTrigger.html)

**Per-player save**
- Each player has `<uuid>.json` under the world's advancements directory, documented on a separate "Advancement storage format" page. The 26.1 Snapshot 6 changelog moved `advancements` to `players/advancements` and `stats` to `players/stats`. — [minecraft.wiki: Java Edition level format](https://minecraft.wiki/w/Java_Edition_level_format)

**Recipe book and recipe advancements**
- Real vanilla `recipes/tools/iron_pickaxe` (26.4-snapshot-6):
  - `parent: minecraft:recipes/root`
  - criteria `has_iron_ingot` (`inventory_changed` with item tag `#minecraft:iron_tool_materials`) and `has_the_recipe` (`recipe_unlocked`, `recipes: minecraft:iron_pickaxe`)
  - `requirements: [["has_the_recipe","has_iron_ingot"]]`, a single OR group
  - `rewards.recipes: ["minecraft:iron_pickaxe"]`

  This is the standard discovery-unlock idiom: obtaining an ingredient grants the recipe. The `has_the_recipe` branch means a player who already knows the recipe completes the advancement at once, so it never fires again. — [misode/mcmeta data branch](https://github.com/misode/mcmeta/tree/data/data/minecraft/advancement/recipes)
- Most recipes unlock when the player holds a key item. Exceptions: boats unlock on touching water, and the chest unlocks after ten inventory slots are filled. Each unlock shows a toast. `/recipe` and knowledge books grant or take recipes. — [minecraft.wiki: Recipe book](https://minecraft.wiki/w/Recipe_book)
- Locked recipes can still be crafted by placing the ingredients by hand, unless the `doLimitedCrafting` gamerule is true. On Bedrock, setting `recipesUnlock=false` at world creation shows every recipe. — [minecraft.wiki: Recipe book](https://minecraft.wiki/w/Recipe_book)
- Player NBT has a `recipeBook` compound recording "recipes the player has unlocked". Its fields are defined in a wiki template I could not retrieve. — [minecraft.wiki: Player.dat format](https://minecraft.wiki/w/Player.dat_format)

**Statistics**
- Stat types: `custom` (walk, jump and similar), `mined` (blocks broken with a tool that would drop them; never counted in Creative), `broken` (tool durability below zero), `crafted` (crafted or smelted; counted when the item is taken from the output slot), `used` (placing, eating, tool use), `picked_up`, `dropped`, `killed`, `killed_by`. — [minecraft.wiki: Statistics](https://minecraft.wiki/w/Statistics)
- File: `stats/<uuid>.json` with `DataVersion` plus a `stats` compound of type → {stat name → integer}. Distances are stored in cm and times in ticks. An entry stays absent until it first changes. History: per-world since 1.7.2, UUID filenames since 1.7.6, names instead of IDs since 1.8, namespaced snake_case since 1.13. — [minecraft.wiki: Statistics](https://minecraft.wiki/w/Statistics)

### Inferences
- Statistics and advancements differ in kind. A stat is an unbounded integer counter keyed by (type, id) with no completion. An advancement is a set of boolean criteria with a fixed completion rule, rewards and UI. Counters feed goals indirectly (Minecraft's own threshold goals tend to be predicates over state, or scoreboards).
- The listener-per-trigger design keeps a check O(listeners on that trigger), not O(all advancements), on every event. Removing listeners once criteria complete is what makes completion idempotent and cheap.
- Keying progress by string ID means renaming a definition silently drops progress (as at 1.16-pre1). Stable IDs are a save-compatibility contract.

### Gaps
- I could not confirm the advancement progress-file schema from a reachable source (the storage-format page returned 404). From prior knowledge, unverified for 26.x: the root is a map of advancement ID → `{ "criteria": { "<name>": "<timestamp e.g. 2024-01-01 12:00:00 +0000>" }, "done": bool }`, plus a root `DataVersion`. Treat this as unverified.
- I could not retrieve the `recipeBook` NBT fields (believed to be `recipes`, `toBeDisplayed` for the "new" highlight, and per-book `isGuiOpen`/`isFilteringCraftable`), the `PlayerAdvancements` internals (whether it unregisters listeners on criterion completion and when it flushes to disk), or the `tick` trigger's description.

## Valheim: discovery-based recipe and build-piece unlocks, and how they are saved

### Takeaway
Valheim unlocks nothing through a quest graph. It keeps flat **sets of known names** in the character profile: recipes, materials, stations with their level, texts, biomes, trophies and tutorials. A recipe or piece counts as known once the player has seen all of its materials and its station. Picking up an item adds that item to the known materials.

### Cited Findings
- Character `PlayerData` fields, in save order (version ≤ 26), are:
  - an inventory, then `knownRecipes` (a string set)
  - `knownStations` (a map of station name → int level)
  - `knownMaterials`, `shownTutorials`, `uniques` and `trophies` (string sets)
  - `knownBiomes` (a list of int IDs) and `knownTexts` (a string → string map)
  - appearance fields, foods and skills
  - from v26, `customData` (a string map, which mods use)

  Fields are added behind `if (version >= N)` gates. — [Kakoen/valheim-save-tools PlayerData.java](https://github.com/Kakoen/valheim-save-tools/blob/master/valheim-save-tools-lib/src/main/java/net/kakoen/valheim/save/archive/character/PlayerData.java)
- The `.fch` file is binary and stores character, inventory, skills and the explored map. The `.fch.old` file is an automatic backup of the previous save. A Nexus save editor recalculates the file's signatures so edited files still load. — [physgun.com save location](https://physgun.com/blog/game-hosting/valheim/valheim-save-file-location/); [Nexus mod 3458](https://www.nexusmods.com/valheim/mods/3458)
- In game code (seen through a Harmony patch), `Player` has private fields `HashSet<string> m_knownMaterial` and `Dictionary<string,int> m_knownStations`, and a method `UpdateKnownRecipesList`. `Player.HaveRequirements(Piece, RequirementMode)` takes modes `IsKnown`, `CanAlmostBuild` and `CanBuild`:
  - `IsKnown`: the piece's station must be in `m_knownStations`, and every resource's `m_shared.m_name` must be in `m_knownMaterial`.
  - `CanAlmostBuild`: the player must have each item in inventory.
  - `CanBuild`: the player needs the full counts, with a station in range.

  `HaveRequirementItems(Recipe, bool discover, int qualityLevel, ...)` has a `discover` flag that switches it to known-material checks. — [aedenthorn/ValheimMods CraftFromContainers](https://github.com/aedenthorn/ValheimMods/blob/main/CraftFromContainers/BepInExPlugin.cs)
- Mods expose helpers named `IsRecipeActuallyKnown`, `IsMaterialKnown` and `KnowsRecipeStationRequirement`, which confirms that the gate is three-part: recipe, material and station. — [Thunderstore VeiledRecipes](https://thunderstore.io/c/valheim/p/sighsorry/VeiledRecipes/v/1.0.0/)

### Inferences
- Valheim's model is **derived knowledge**: it stores only primitive discoveries (materials seen, stations seen) and computes recipe and piece availability from the definitions. `knownRecipes` is a cached or persisted result of that computation. A new recipe added by a patch or mod therefore becomes known automatically when its inputs are already known. Nothing needs migrating.
- Station *level* is stored, so recipe tiers gated on station upgrades can be expressed through the same map.

### Gaps
- I did not find a source confirming whether `UpdateKnownRecipesList` runs on a timer in `Player.Update` (polling) or only on pickup and station events. I also did not find the exact pickup hook (believed to be `AddKnownItem`).

## Subnautica: fragment scanning, KnownTech/analysis tech, encyclopedia, and save format

### Takeaway
Subnautica separates three kinds of data:
- **scanner entries**: TechType key → blueprint, the number of fragments required, scan time, and an encyclopedia key
- **KnownTech**: the set of unlocked TechTypes, plus "analysis tech" entries in which unlocking or picking up one TechType unlocks a list of others, and "compound" unlocks that need several TechTypes
- **encyclopedia (PDA databank) entries**: keyed strings with a databank path

### Cited Findings
- Nautilus `KnownTechHandler` (BepInEx 5, the current modding stack; QModManager and SMLHelper are legacy) exposes:
  - `UnlockOnStart(TechType)`; `SetHardLocked(TechType)` (excluded from `unlockall` and from creative defaults)
  - `AddRequirementForUnlock(blueprint, requirement)` ("Unlocks the blueprint when the requirement tech type is unlocked")
  - `SetAnalysisTechEntry(techTypeToBeAnalysed, IEnumerable<TechType> techTypesToUnlock, ...)`, which can add an unlock sound, sprite, message, or a `List<StoryGoal>`, and appends to an existing entry
  - `SetCompoundUnlock(techType, List<TechType>)` ("requiring multiple techtypes to be unlocked before 1 is")
  - `RemoveDefaultUnlock`, `RemoveAllCurrentAnalysisTechEntry`, `RemoveAnalysisTechEntryFromSpecific`

  — [Nautilus KnownTechHandler API](https://subnauticamodding.github.io/Nautilus/api/Nautilus.Handlers.KnownTechHandler.html); [Nexus Ultimate Finder (stack requirements)](https://www.nexusmods.com/subnautica/mods/4045)
- Nautilus `PDAHandler.AddCustomScannerEntry(key, blueprint, isFragment, totalFragmentsRequired, scanTime, destroyAfterScan, encyclopediaKey)`:
  - `key`: the scanned object's TechType.
  - `blueprint`: unlocked "when key is scanned"; may be None.
  - `totalFragmentsRequired`: "the total amount of objects of key that need to be scanned to unlock the blueprint and encyclopediaKey".
  - `destroyAfterScan`: destroys the scanned object.

  `EditFragmentsToScan(TechType, int)` changes the count. `AddEncyclopediaEntry(key, path, ...)` uses localisation keys `Ency_{key}` and `EncyDesc_{key}`. — [Nautilus PDAHandler API](https://subnauticamodding.github.io/Nautilus/api/Nautilus.Handlers.PDAHandler.html)
- Players report that fragment-scan progress made since the last save is lost on reload, so partial counts are saved state. — [Steam discussion](https://steamcommunity.com/app/264710/discussions/0/2333276539607034660)

### Inferences
- The fragment counter is a **counter criterion with a threshold** (n of N) that resolves to a flag: the blueprint enters KnownTech. Analysis tech is a **reverse-dependency trigger** ("obtaining X reveals everything X is used in"), the same idiom as Minecraft's `inventory_changed` recipe advancements.
- Hard-locking and unlock-on-start show that designers need an explicit default policy per tech, separate from the triggers.

### Gaps
- I found no reliable source for the Subnautica save-file format (believed to be a protobuf-serialised `gameinfo` plus scene data holding KnownTech and PDAScanner partial counts), or for whether PDAScanner progress is event-driven. Subnautica 2 guides (fan sites) report different fragment counts, so I did not use them.

## Terraria, Don't Starve, Stardew Valley, No Man's Sky, Astroneer (brief)

### Takeaway
Two other unlock economies show up. **Station-gated prototyping** (Don't Starve): a recipe is learned permanently by crafting it once near a prototyper. **Currency-gated catalogs** (Astroneer): researching objects yields Bytes, which buy schematics. Stardew's Community Center bundles are the clearest data-driven "item list with persisted partial progress" goal. The modding wiki that documents them was unreachable.

### Cited Findings
- Don't Starve: each item or structure generally has to be "prototyped" at a particular prototyper. After that it can be crafted anywhere. A crafting station, by contrast, does not unlock the recipe, so the player must return to it each time. — [Don't Starve wiki: Prototype](https://dontstarve.wiki.gg/wiki/Prototype); [Craftable](https://dontstarve.wiki.gg/wiki/Craftable)
- Astroneer: researching objects in a Research Chamber yields Bytes, which unlock schematics in the Research Catalog. Every module can be unlocked there, and unlocking everything costs 284,050 Bytes in total. — [Astroneer wiki: Research](https://astroneer.wiki.gg/wiki/Research_(Mechanic)); [Bytes](https://astroneer.wiki.gg/wiki/Bytes)
- Stardew Valley bundles (secondary source that summarises the wiki's Modding:Bundles): each required item is a triple of (object ID, count, minimum quality). Object ID −1 means gold, where count and quality both hold the amount. Each bundle has a reward type, reward object ID and count, and a colour index. — [rdew R package docs, citing the Stardew wiki](https://rdrr.io/github/aftonsteps/rdew/man/bundles.html)

### Inferences
- Bundles are AND-of-slots with a "k of n" option, as in Minecraft `requirements`. Persisting per-slot donation flags is what lets partial donations survive. This is the right shape for any "deliver these items" goal.

### Gaps
- stardewvalleywiki.com (Modding:Bundles and Modding:Quest data) returned 403 and a Cloudflare challenge, so the exact `Data/Bundles` key format, the 1.6 randomised-bundle data, the quest schema and the save fields could not be confirmed.
- I did not research Terraria (its crafting is station-proximity with no learned recipes, as far as I know; unverified) or No Man's Sky's milestone and mission log, for lack of tool budget.

## Patterns: event-driven vs polling, idempotence, counters vs flags, data-driven and moddable

### Takeaway
Across these games, goals reduce to a few primitives:
- typed **triggers** (events with payloads)
- **criteria** (a trigger plus a data predicate)
- **combinators** (AND-of-OR, k-of-n, n-count thresholds)
- **rewards** (unlock IDs, items, scripts)

Progress is stored as a set of completed criterion IDs (flags) or as counters keyed by stable string IDs. Event subscription with listener removal after completion is the scalable, idempotent design. Derived-knowledge models (Valheim) avoid storing what can be recomputed.

### Cited Findings
- Event-driven: Minecraft triggers keep per-player listener sets and fire `trigger(player, predicate)` from game events. — [mappings.dev SimpleCriterionTrigger](https://mappings.dev/1.21.11/net/minecraft/advancements/criterion/SimpleCriterionTrigger.html)
- Combinators: Minecraft `requirements` is AND-of-OR. — [Advancement definition](https://minecraft.wiki/w/Advancement_definition). Subnautica has compound unlocks (all of a list) and fragment thresholds (N scans). — [Nautilus KnownTechHandler](https://subnauticamodding.github.io/Nautilus/api/Nautilus.Handlers.KnownTechHandler.html); [PDAHandler](https://subnauticamodding.github.io/Nautilus/api/Nautilus.Handlers.PDAHandler.html)
- Idempotence through an "already have it" branch: Minecraft recipe advancements OR in `recipe_unlocked` for their own reward. — [misode/mcmeta](https://github.com/misode/mcmeta/tree/data/data/minecraft/advancement/recipes)
- Counters vs flags: Minecraft keeps stats (unbounded counters, written lazily) separate from advancements (criteria flags). — [Statistics](https://minecraft.wiki/w/Statistics)
- Data-driven and moddable:
  - Minecraft definitions live in data packs, and the `impossible` trigger plus `rewards.function` let commands and scripts drive goals. — [Advancement definition](https://minecraft.wiki/w/Advancement_definition)
  - Subnautica's tables are patched through Nautilus handlers.
  - Valheim adds a `customData` string map to the save from v26 for extensions. — [valheim-save-tools](https://github.com/Kakoen/valheim-save-tools/blob/master/valheim-save-tools-lib/src/main/java/net/kakoen/valheim/save/archive/character/PlayerData.java)
- Save stability: progress is keyed by ID, so renaming a definition drops progress (Minecraft 1.16-pre1). — [Advancement](https://minecraft.wiki/w/Advancement). Valheim uses version-gated binary fields. — [valheim-save-tools](https://github.com/Kakoen/valheim-save-tools/blob/master/valheim-save-tools-lib/src/main/java/net/kakoen/valheim/save/archive/character/PlayerData.java)

### Inferences
- What this suggests for a Rust/Bevy, recipe-book design like stubby_recipes (my synthesis):
  - Define triggers as an enum of game events (for example ItemCollected{id}, RecipeCrafted{id}, ModulePatched{...}).
  - Have criteria hold a trigger kind plus a data predicate, and have goals hold `requirements: Vec<Vec<CriterionId>>`.
  - Keep an index from trigger kind to open criteria per player, and remove entries on completion.
  - Store the completed criteria (with timestamps) and counters in a save keyed by stable string IDs.
  - Make "recipe-book entry learned" a flag that never reverts when inventory reaches zero. That matches Minecraft and Valheim.
  - For reverse discovery ("having a square wave reveals the recipes that use it"), either author unlock rules per recipe (the Minecraft approach) or derive them from recipe inputs (the Valheim approach). Deriving needs no extra data, and new recipes become discoverable automatically.
- Polling fits only state predicates that have no natural event, such as "stand in a biome" or "light quality ≥ x". Even those can be throttled (for example once per second) or turned into events on cell changes.

### Gaps
- I found no developer post or GDC talk on these systems within budget. Every architectural claim here comes from code mappings, mod APIs, or my own inference.
