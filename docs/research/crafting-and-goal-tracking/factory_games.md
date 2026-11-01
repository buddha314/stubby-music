# Goal / progression tracking and crafting unlocks in shipped factory games (data model and implementation)

Researched 2026-11-01. Primary sources used where possible: the Satisfactory Mod Loader (SML) headers on GitHub (`master`, which tracks the current Satisfactory 1.x release), the Factorio Lua API docs (labelled **2.1.24**), and Mindustry source on GitHub (`master`). Coverage of Dyson Sphere Program, Shapez 2 and Captain of Industry is thinner. Those games are closed-source and their wikis are sparse, so many details there are listed as gaps.

## Satisfactory: schematics, the schematic manager, unlocks, dependencies, game phases, to-do list, save data

### Takeaway
Satisfactory models every unlock as a **schematic** data asset (`UFGSchematic`). A schematic has a type enum, an item cost, a list of instanced `UFGUnlock` objects and a list of instanced availability-dependency objects. A single world actor, `AFGSchematicManager`, holds the save state: purchased schematic classes, the one active schematic, and a per-schematic array of **paid-off item amounts** (`FSchematicCost`). Partial payment therefore persists independently for every schematic. The Space Elevator uses the same pattern in `AFGGamePhaseManager`: current phase, target phase, and paid-off costs toward the target phase. Delivery consumes items: they are "paid off" into the HUB or Space Elevator.

### Cited Findings
- `ESchematicType` values: `EST_Custom`, `EST_Cheat`, `EST_Tutorial`, `EST_Milestone`, `EST_Alternate`, `EST_Story`, `EST_MAM`, `EST_ResourceSink`, `EST_HardDrive`, `EST_Prototype`, `EST_Customization`. The enum carries a dev comment: "@todo [MODSUPPORT] This should maybe be implemented the same way as UFGBuildCategories?" — [SML FGSchematic.h](https://raw.githubusercontent.com/satisfactorymodding/SatisfactoryModLoader/master/Source/FactoryGame/Public/FGSchematic.h)
- `UFGSchematic` fields:
  - `mType`, `mDisplayName`, `mDescription`
  - `mSchematicCategory` and `mSubCategories`
  - `mMenuPriority` (float; lower sorts earlier)
  - `mTechTier` (int32, AssetRegistrySearchable)
  - `mCost` (`TArray<FItemAmount>`)
  - `mTimeToComplete` (float, the time the schematic takes to finish after it is acquired)
  - `mUnlocks` (`TArray<TObjectPtr<UFGUnlock>>`, Instanced)
  - `mSchematicDependencies` (`TArray<TObjectPtr<UFGAvailabilityDependency>>`, Instanced: "Is this schematic dependant on anything to be available for purchase?")
  - `mDependenciesBlocksSchematicAccess`, `mHiddenUntilDependenciesMet`
  - `mIsPlayerSpecific`: per-player rather than session-wide purchase. It is only valid for `EST_ResourceSink`.
  - `mStatisticGameplayTag`: an online stat incremented on unlock.
  - `mRelevantEvents`, `mGameplayTags`, `mSchematicUnlockTag`
  - Deprecated: `mDependsOnSchematic` and `mAdditionalSchematicDependencies`, replaced by availability dependencies.

  — [SML FGSchematic.h](https://raw.githubusercontent.com/satisfactorymodding/SatisfactoryModLoader/master/Source/FactoryGame/Public/FGSchematic.h)
- Dependency types exposed to modders are `BPSchematicPurchasedDependency` and `BPItemPickedUpDependency`. Every dependency entry must be satisfied. Each group has a "Require All" toggle; when it is off, any one match is enough. Item-pickup dependencies only work if the item descriptor has the `HasPickedUpItem` flag set. — [docs.ficsit.app Schematic](https://docs.ficsit.app/satisfactory-modding/latest/Development/Satisfactory/Schematic.html)
- Unlock types (FGUnlock wrappers):
  - `UnlockRecipe`, `UnlockSchematic`
  - `UnlockInventorySlot`, `UnlockArmEquipmentSlot`
  - `UnlockGiveItem`
  - `UnlockMap`, `UnlockScannableResource`
  - `UnlockBuildEfficiency`: shows per-second production and consumption in machine UIs.
  - `UnlockBuildOverclock`
  - `UnlockSubsystem`

  — [docs.ficsit.app Schematic](https://docs.ficsit.app/satisfactory-modding/latest/Development/Satisfactory/Schematic.html). That page carries "outdated/development" banners.
- Tech tier 0 is the tutorial stage. In tier 0 the HUB shows "Upgrade HUB" instead of "Launch". `mTimeToComplete` is the freighter flight time for milestones and the processing time for MAM research. — [docs.ficsit.app Schematic](https://docs.ficsit.app/satisfactory-modding/latest/Development/Satisfactory/Schematic.html)
- `AFGSchematicManager` save state:
  - `mPurchasedSchematics` (`TArray<TSubclassOf<UFGSchematic>>`, SaveGame)
  - `mPaidOffSchematic` (`TArray<FSchematicCost>`, SaveGame, replicated): "This keeps track of what players have paid off on different schematics"
  - `mActiveSchematic` (SaveGame): "The active schematic the resources is being sold towards"
  - `mLastActiveSchematic`
  - `mShipLandTimeStampSave` (float, SaveGame): the freighter return time
  - `mMaxAllowedTechTier` (default 6)

  `mAllSchematics` is not saved; it is populated at startup. — [SML FGSchematicManager.h](https://raw.githubusercontent.com/satisfactorymodding/SatisfactoryModLoader/master/Source/FactoryGame/Public/FGSchematicManager.h)
- `FSchematicCost` = { `Schematic` (class), `ItemCost` (`TArray<FItemAmount>`, comment "Amount paid off") }. This is how **partial payment is stored**: one row per schematic that has received anything. — [SML FGSchematicManager.h](https://raw.githubusercontent.com/satisfactorymodding/SatisfactoryModLoader/master/Source/FactoryGame/Public/FGSchematicManager.h)
- Schematic manager API:
  - `PayOffOnSchematic(schematic, TArray<FItemAmount>& amount)` → bool. The amount is passed by reference, which suggests it is decremented by what was accepted.
  - `GetRemainingCostFor`, `GetPaidOffCostFor`, `IsSchematicPaidOff`
  - `SetActiveSchematic`, `CanSetAsActiveSchematic`
  - `GetAvailableSchematicsOfTypes` (dependencies met), `GetAvailableNonPurchasedSchematicsOfTypes`
  - `GiveAccessToSchematic(class, instigator, unlockFlags)`
  - `LaunchShip`, `IsShipAtTradingPost`, `GetTimeUntilShipReturn`
  - Delegates: `PurchasedSchematicDelegate`, `PaidOffOnSchematicDelegate`, `mOnActiveSchematicChanged`.
  - `GetCostFor` is deprecated: "Get the cost from the Schematic directly".

  — [SML FGSchematicManager.h](https://raw.githubusercontent.com/satisfactorymodding/SatisfactoryModLoader/master/Source/FactoryGame/Public/FGSchematicManager.h)
- Space Elevator (`AFGGamePhaseManager`) save state:
  - `mCurrentGamePhase` and `mTargetGamePhase` (`UFGGamePhase*` data assets; target is null at end of game)
  - `mTargetGamePhasePaidOffCosts` (`TArray<FItemAmount>`): "The amounts we paid off for the current game phase"
  - `mPALaunchSequenceValues`
  - `mIsGameCompleted`
  - Deprecated, kept for save compatibility: `mGamePhase` (an `EGamePhase` enum) and `mGamePhaseCosts`

  API: `PayOffOnTargetGamePhase(FItemAmount)` returns the amount accepted. `GetRemainingPhaseCosts` returns `FRemainingPhaseCost{ItemClass, TotalCost, RemainingCost}`. Also `ReadyToGoToNextGamePhase`, `GoToNextGamePhase` and `GetLastTechTierForGamePhase`. Phase assets are sorted by `mLastTierOfPhase`, then `mPriority`. — [SML FGGamePhaseManager.h](https://raw.githubusercontent.com/satisfactorymodding/SatisfactoryModLoader/master/Source/FactoryGame/Public/FGGamePhaseManager.h)
- To-do list and recipe pinning live on the player state, `AFGPlayerState`:
  - `mPrivateTodoList` (FString, SaveGame). The public to-do list is set through `SetPublicTodoList` and tracks `GetPublicTodoListLastEditedBy`.
  - `mShoppingListComponent` (`UFGShoppingListComponent`, SaveGame), which holds the pinned-recipe list.
  - `mShoppingListSettings` = visibility flags for the public, private and recipe lists, plus size.
  - A migration hook, `GetAndClearShoppingListForMigration()`, returns `TArray<FShoppingListRecipeEntry>`.
  - `mCentralStoragePinnedItems` (an array of item classes) is used for the central storage UI.

  — [SML FGPlayerState.h](https://raw.githubusercontent.com/satisfactorymodding/SatisfactoryModLoader/master/Source/FactoryGame/Public/FGPlayerState.h)
- Player-reported behaviour: pinning several recipes sums their total material cost, and a pin is removed automatically when the recipe is built. — [Icarus feature request citing Satisfactory](https://icarus.featureupvote.com/suggestions/255040/suggestion-ingame-todo-list-and-recipe-pin)

### Inferences
- Save state is **minimal and derived-friendly**. Only purchased class refs, paid-off amounts and the active pointer are saved. Costs, unlocks and dependency graphs stay in immutable data assets, and "available" is computed on demand from dependencies. That is a clean split between static definitions and mutable progress.
- Partial payment is keyed per schematic, so switching the active milestone keeps the progress already paid toward the old one.
- Dependencies are polymorphic, instanced objects (schematic purchased, item picked up). That makes "first time you hold X" an unlock trigger without a separate event system, provided the item flags its pickups.
- The to-do list stores pinned **recipes** (and their amounts, presumably) in a component, not as goals. "Remaining" is computed in the UI.

### Gaps
- The `FShoppingListRecipeEntry` fields and the `UFGShoppingListComponent` internals were not fetched.
- I did not fetch the `UFGGamePhase` asset field list (cost and last-tier fields are implied by comments only).
- The MAM research tree node data (`UFGResearchTree`, nodes with parent links) was not fetched.
- How the HUB terminal chooses which inventory items to pay (all-or-partial per stack) is not confirmed. The by-reference `amount` parameter only suggests partial acceptance.

## Factorio 2.0 / 2.1: technologies, research triggers, per-force progress, flow statistics

### Takeaway
A technology prototype has **either** a `unit` (science-pack cost: count or `count_formula` × ingredients × time) **or** a `research_trigger`, a single event condition. Its other fields are `prerequisites` and a list of `effects` modifiers such as `unlock-recipe`. Progress is per force: the current research fraction, a queue, and `saved_progress` per technology. Production goals read `LuaFlowStatistics`, which records cumulative **input (produced)**, **output (consumed)** and **storage** counts per prototype, per surface, with time-windowed samples.

### Cited Findings
- TechnologyPrototype fields:
  - `unit` (TechnologyUnit): "Required if `research_trigger` is not defined"
  - `research_trigger` (TechnologyTrigger): "Required if `unit` is not defined"
  - `prerequisites`, `effects` (list of Modifier, e.g. `unlock-recipe`)
  - `max_level` (uint32 or "infinite"), `upgrade`
  - `enabled`, `essential`, `visible_when_disabled`, `ignore_tech_cost_multiplier`, `allows_productivity`

  A trailing `-<number>` in the name is the level, and levels must be contiguous. — [TechnologyPrototype (API 2.1.24)](https://lua-api.factorio.com/latest/prototypes/TechnologyPrototype.html)
- TechnologyUnit: `count` (uint64 >0) or `count_formula` (MathExpression per level), never both. `time` is the seconds per unit in a speed-1 lab. `ingredients` is the list of research ingredients per unit. — [TechnologyUnit](https://lua-api.factorio.com/latest/types/TechnologyUnit.html)
- TechnologyTrigger union, selected by `type`: `mine-entity`, `craft-item`, `craft-fluid`, `send-item-to-orbit`, `capture-spawner`, `build-entity`, `create-space-platform`, `scripted`. — [TechnologyTrigger](https://lua-api.factorio.com/latest/types/TechnologyTrigger.html)
- `CraftItemTechnologyTrigger` = { `type`="craft-item", `item`: ItemIDFilter (required), `count`: ItemCountType (default 1) }. — [CraftItemTechnologyTrigger](https://lua-api.factorio.com/latest/types/CraftItemTechnologyTrigger.html)
- `LuaForce.script_trigger_research(technology)` fires a technology's "scripted" trigger. It does nothing if the technology has no such trigger. — [LuaForce](https://lua-api.factorio.com/latest/classes/LuaForce.html)
- FFF #376 (Sept 2023) introduced triggered techs. The cited examples: Steel Axe requires actually crafting steel plates, and Steam Power might require crafting 50 iron plates. Landing on a planet lets you "discover" its resources naturally. — [FFF #376](https://direct.factorio.com/blog/post/fff-376). Player reception was mixed ("Why would I want to craft random amounts of res to unlock random tech?"). — [FFF #376 forum thread](https://forums.factorio.com/viewtopic.php?p=591287)
- Edge case: the `mine-entity` trigger on a resource entity also fires when that resource is collected as a side effect of mining some other entity, by hand or by drill. — [Forum bug report 2.0.32](https://forums.factorio.com/viewtopic.php?p=655779). Modders have asked for multiple triggers per technology, which the API did not support. — [Mod interface request](https://forums.factorio.com/viewtopic.php?p=649378)
- Per-force research state on `LuaForce`:
  - `current_research`, `research_progress` (double in [0,1])
  - `research_queue` (first entry active; already-researched or unavailable entries are dropped silently)
  - `previous_research`, `technologies` (indexed by name)
  - `add_research`, `cancel_current_research`, `reset_technologies`
  - Events: `on_research_started`, `on_research_finished`, `on_research_cancelled`, `on_research_queued`, `on_research_moved`

  — [LuaForce](https://lua-api.factorio.com/latest/classes/LuaForce.html)
- `LuaTechnology` fields:
  - `researched` (r/w; setting it applies or reverses the effects)
  - `enabled`, `visible_when_disabled`
  - `saved_progress`: "Saved technology progress fraction as a value in range [0, 1)"
  - `level`, `research_unit_count` (multiplied by the cost multiplier unless that is ignored)
  - `prerequisites`, `successors`

  — [LuaTechnology](https://lua-api.factorio.com/latest/classes/LuaTechnology.html)
- Flow statistics are per force **and per surface**: `get_item_production_statistics(surface)`, plus fluid, kill-count and entity-build-count (built and mined) statistics. — [LuaForce](https://lua-api.factorio.com/latest/classes/LuaForce.html)
- `LuaFlowStatistics`:
  - Totals: `input_counts`, `output_counts`, `storage_counts`, each also split by quality.
  - `get_flow_count{name, category, precision_index, sample_index?, count?}`: each precision level holds 300 samples.
  - `on_flow(id, count)`: the sign of `count` picks input or output.
  - For items, "`output` describes the item consumption numbers", so input = production.

  — [LuaFlowStatistics](https://lua-api.factorio.com/latest/classes/LuaFlowStatistics.html)

### Inferences
- Factorio separates **cost-based** progress (a fraction toward N units) from **event-based** progress (a trigger with a count). A trigger tech is a one-off goal, and progress toward `count` lives in the engine; the API exposes no per-trigger counter that I found.
- Flow statistics are a general, cumulative, windowed ledger keyed by prototype. Achievements and goals can query "produced N total" or "rate over the last minute" without each goal subscribing to events. A comparable ledger (produced / consumed / held, keyed by item ID) would let us express many goals as pure queries.
- Storing progress as a fraction (`saved_progress` in [0,1)) instead of raw counts makes cost-multiplier changes easy, but loses the per-ingredient detail Satisfactory and Mindustry keep. Factorio can do this because labs consume full "units" of all ingredients in lockstep.

### Gaps
- Fields of the other trigger types (e.g. `send-item-to-orbit`'s item, `build-entity`'s entity and count) were not fetched individually.
- How achievement prototypes (e.g. `produce-achievement` with amount and `limited_to_one_game`) read flow statistics was not fetched. I can't cite their fields.
- The tips-and-tricks prototype (`TipsAndTricksItem` with `trigger` / `skip_trigger` types such as `craft-item`, `build-entity`, `use-pipette`) was not fetched this session. Verify at lua-api.factorio.com/latest/prototypes/TipsAndTricksItem.html.

## Mindustry (v7, GitHub master): tech tree, research spending, sector objectives and map objectives

### Takeaway
Each `TechNode` holds `requirements` (an ItemStack cost, scaled by per-item multipliers), a parallel `finishedRequirements` array of **per-item partial progress**, and a list of `objectives` that must also hold. Progress is persisted per item under settings keys `req-<content>-<item>`. Campaign objectives such as `Research` and `Produce` both check only `content.unlockedHost()`. Map objectives offer a menu of goal kinds that separate items **held** (team items or core item count) from blocks **built** (cumulative `placedBlockCount`) and units **alive**.

### Cited Findings
- `TechNode`:
  - `requirements` are scaled by `researchCostMultipliers` (a per-item float map shared down the subtree).
  - `finishedRequirements` has the same length as `requirements`. It is loaded with `Core.settings.getInt("req-" + content.name + "-" + item.name)`, and `save()` writes only the values that changed.
  - `objectives`: one `Research` objective per content dependency, a `SectorComplete` objective when the parent is a SectorPreset, and `nodeProduce` appends a `Produce` objective.
  - Costs come from `content.researchRequirements()`.

  — [TechTree.java](https://raw.githubusercontent.com/Anuken/Mindustry/master/core/src/mindustry/content/TechTree.java)
- Campaign objectives:
  - `Research` and `Produce`: `complete()` returns `content.unlockedHost()`. Produce differs only in display; unrevealed content shows `???`.
  - `SectorComplete`: the sector has a save, is captured and has a base.
  - `OnSector`, `OnPlanet`

  — [Objectives.java](https://raw.githubusercontent.com/Anuken/Mindustry/master/core/src/mindustry/game/Objectives.java)
- Map objectives (`MapObjectives.java`):
  - `ItemObjective`: `state.rules.defaultTeam.items().has(item, amount)`, i.e. **held**.
  - `CoreItemObjective`: `state.stats.coreItemCount.get(item) >= amount`.
  - `BuildCountObjective`: `state.stats.placedBlockCount.get(block) >= count`, i.e. **built, cumulative**.
  - `UnitCountObjective`: units alive of a type.
  - `DestroyUnitsObjective`, `DestroyBlock(s)Objective`, `DestroyCoreObjective`
  - `TimerObjective`, `FlagObjective`, `CommandModeObjective`
  - `ResearchObjective` and `ProduceObjective`: both `content.unlocked()`

  — [MapObjectives.java](https://raw.githubusercontent.com/Anuken/Mindustry/master/core/src/mindustry/game/MapObjectives.java)
- Map objectives chain through parents. An objective is `qualified()` only when it is not completed and all its parents are completed. Only qualified objectives are updated, by polling `update()` each frame. Completion is server-authoritative (`Call.completeObjective(index)`) and can add or remove flags and run logic code. — [MapObjectives.java](https://raw.githubusercontent.com/Anuken/Mindustry/master/core/src/mindustry/game/MapObjectives.java)

### Inferences
- Mindustry uses **polled predicates** over global stats and state, not event counters. That is simple and robust, but needs the stats to exist (`placedBlockCount`, `coreItemCount`).
- Per-item partial research progress, as in Satisfactory, lets players contribute materials in any order.
- "Produce X" is implemented as "X is unlocked". The unlock happens elsewhere, as a side effect of obtaining the item.

### Gaps
- Where `finishedRequirements` gets incremented, i.e. whether research pulls items out of the core (consuming them), was not shown in this file. I recall that campaign research spends core items, but did not verify it this session.
- Exactly when `unlock()` fires for an item (e.g. on first entering a core) was not verified.

## Dyson Sphere Program, Shapez 2, Captain of Industry

### Takeaway
All three are closed-source and sparsely documented at the data-model level. What I could confirm:
- **Shapez 2** counts shapes **delivered to the Vortex** toward milestones and toward per-line, exponentially scaling "goal lines", which feed a single Operator Level.
- **DSP** "Metadata" is a cross-save currency derived from matrix production or consumption during research, used to buy out techs.
- **Captain of Industry** research is RP-based, with labs converting consumed items into RP and a FIFO queue.

### Cited Findings
- Shapez 2: the Vortex is where all shapes must be delivered to count toward a Milestone or Job. — [Shapez 2 wiki: Vortex](https://shapez2.wiki.gg/wiki/Vortex)
- Shapez 2 goal lines:
  - Each milestone from "Fluids & Painting" onward unlocks a goal line whose shape is that milestone's final shape.
  - Each line levels independently, with required counts growing exponentially from a per-scenario start amount at a growth percentage, capped at 2,147,482,600.
  - Two "Random Operator Shape" lines are seeded from the savegame.
  - Operator Level (one number per save) rises by 1 for every goal-line level completed. Each level awards Platform Units, Blueprint Points and Research Points, scaled by difficulty settings such as Goal Multiplier.

  — [Shapez 2 wiki: Operator Level (rev 2840, May 2026)](https://shapez2.wiki.gg/wiki/Operator_Level)
- Shapez 2: one Steam post says an upgrade (around build 0.0.9) makes each delivered shape count as 2×, 3× and so on toward tasks. That is a **delivery-count multiplier** decoupled from physical items. — [Steam discussion](https://steamcommunity.com/app/2162800/discussions/0/6929437211302478001) (player report, unverified)
- DSP Metadata (patch 0.9.25.11985 FAQ):
  - It is obtained by mass-producing matrices and is used for "technology instant unlocking (buyout)".
  - It is shared across saves.
  - Contributions from saves at different cluster addresses add up; from the same cluster address only the maximum counts (the 360+180 → 540 example).
  - A save that has used metadata cannot upload to the Milky Way.

  — [DevTrackers copy of DSP patch notes](https://devtrackers.gg/dyson-sphere-program/p/476c8c62-dyson-sphere-program-patch-notes-0-9-25-11985-faq). A player claims metadata derives from the matrix **consumption rate in research**, not from surplus production. — [Steam thread](https://steamcommunity.com/app/1366540/discussions/0/3588834930881334620) (anecdotal, conflicts with "mass-producing")
- DSP research cost is in hashes: e.g. Information Matrix data volume is 288k hashes, and per-lab hash rate is 60 + 60 × Research Speed level. — [DSP wiki Information Matrix](https://dev.dsp-wiki.com/Purple_cube); [TrueAchievements guide](https://www.trueachievements.com/a373268/data-rules-everything-around-me-achievement)
- Captain of Industry:
  - Labs produce research points (RP) equal to the items they consume (3 RP for the first lab), with bonuses from population and edicts.
  - Each research has a fixed RP cost.
  - The queue runs FIFO, and cancelling the current research clears the whole queue.
  - Repeatable research needs space research (v0.8.5).

  — [CoI wiki Research](https://wiki.coigame.com/Research) (stub page)
- Captain of Industry modding registers research nodes as prototype data through `RegisterDataWithInterface<IResearchNodesData>()`. — [CoI modding README](https://raw.githubusercontent.com/MaFi-Games/Captain-of-industry-modding/main/README.md)

### Inferences
- Shapez 2 is the purest "deliver-and-consume" design: the Vortex is a sink, and every goal is a count of a shape into that sink. Endless goal lines with exponential targets turn the sink into an infinite progression without a separate stats system.
- DSP layers a meta-progression currency on top of normal consumption-based research. That is relevant only if we want cross-world progression, which conflicts with our seeded-world simulator stance only if it is allowed to bypass discovery.

### Gaps
- The DSP tech data model (TechProto fields, hash progress storage) and its achievement/"milestone" system: no primary source found.
- Whether shapes delivered to the Shapez 2 Vortex are destroyed (strongly implied, not stated on the fetched page), and the internal goal-line data model.
- The Captain of Industry goals/tutorial "tasks" system: no source found (search returned noise). Whether CoI goals count produced or delivered items is unanswered.
- No developer GDC talks or devlogs on goal-tracker design were found within budget.

## Produced vs delivered/consumed vs held: who counts what, and trade-offs

### Takeaway
There are three basic accounting modes:
1. **Deliver and consume** into a sink: Satisfactory HUB and Space Elevator, Shapez 2 Vortex, research labs in Factorio, CoI and DSP, Mindustry research.
2. **Cumulative produced/built counter**: Factorio `craft-item` / `build-entity` triggers and flow statistics, Mindustry `placedBlockCount`.
3. **Held or present snapshot**: Mindustry `ItemObjective` and `CoreItemObjective`, Satisfactory's item-picked-up dependency.

Consume-on-delivery stores partial progress per item. Production counters need a global ledger.

### Cited Findings
- Satisfactory: delivered and consumed, with per-item partial payments stored (`FSchematicCost.ItemCost` "Amount paid off"; `mTargetGamePhasePaidOffCosts`). — [FGSchematicManager.h](https://raw.githubusercontent.com/satisfactorymodding/SatisfactoryModLoader/master/Source/FactoryGame/Public/FGSchematicManager.h); [FGGamePhaseManager.h](https://raw.githubusercontent.com/satisfactorymodding/SatisfactoryModLoader/master/Source/FactoryGame/Public/FGGamePhaseManager.h)
- Satisfactory also uses **first pickup** (held once) as a visibility gate, through `BPItemPickedUpDependency` and the descriptor flag `HasPickedUpItem`. — [docs.ficsit.app](https://docs.ficsit.app/satisfactory-modding/latest/Development/Satisfactory/Schematic.html)
- Factorio: science units are consumed in labs (`unit.ingredients` per unit). Trigger techs count **crafted, mined or built** events without consuming anything. — [TechnologyUnit](https://lua-api.factorio.com/latest/types/TechnologyUnit.html); [TechnologyTrigger](https://lua-api.factorio.com/latest/types/TechnologyTrigger.html)
- Factorio flow statistics separate produced (input), consumed (output) and storage, per surface. — [LuaFlowStatistics](https://lua-api.factorio.com/latest/classes/LuaFlowStatistics.html)
- Mindustry offers both held (`items().has`, `coreItemCount`) and cumulative built (`placedBlockCount`) goals. — [MapObjectives.java](https://raw.githubusercontent.com/Anuken/Mindustry/master/core/src/mindustry/game/MapObjectives.java)
- Shapez 2: delivered to the Vortex. — [Vortex](https://shapez2.wiki.gg/wiki/Vortex)
- Designer and player trade-off on trigger ("produce") goals: Wube's FFF #376 framed crafting triggers as natural discovery, e.g. craft steel to unlock steel-related tech, or land on a planet to discover its resources. Some players found arbitrary craft counts odd. — [FFF #376](https://direct.factorio.com/blog/post/fff-376); [forum](https://forums.factorio.com/viewtopic.php?p=591287)
- Ambiguity hazard with produced counters: Factorio's `mine-entity` also fires on side-effect collection. — [forum](https://forums.factorio.com/viewtopic.php?p=655779)

### Inferences
- **Consume-on-delivery** is unambiguous and exploit-resistant: items cannot be counted twice. It creates a resource sink, and the persistent state is small (paid per item per goal). It requires a physical delivery point (HUB, Vortex, lab).
- **Produced counters** reward building the chain rather than stockpiling, and need no sink. They do need a global ledger and careful definitions of "produced" (crafted by hand vs by machine, side-effect yields).
- **Held snapshots** are cheap to poll, but can be satisfied then lost, and are gameable by moving items around. They suit "first discovery" gates (Satisfactory pickup dependency) better than quantity goals.
- For stubby-music: a recipe-book "learned" state that survives inventory reaching zero (as AGENTS.md requires) maps to Satisfactory's pickup dependency or Mindustry's `unlocked()`. Milestone-style costs map to a per-goal paid-off `Vec<(ItemId, u32)>` à la `FSchematicCost`. Throughput goals (e.g. "stream delivers N beats") map to a Factorio-style flow ledger keyed by item ID, windowed by sample time.

### Gaps
- I found no explicit designer quotes (GDC or devlog) from Coffee Stain, Wube or tobspr arguing for consumption against production accounting, beyond FFF #376's framing.
