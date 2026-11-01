# Crafting / inventory / recipe systems: engine plugin maturity (as of 2026-11-01)

Note on method: about 22 search/fetch calls. Figures come from store pages and GitHub, fetched on 2026-11-01 unless the note says otherwise. Asset Store star averages often did not render in the extracted text. Where a number comes from a third-party tracker, the note says so.

## Unity: Asset Store and open-source systems

### Takeaway
Unity has the deepest market. **Opsive Ultimate Inventory System (UIS)** is the most clearly maintained commercial framework (v1.3.10, Sep 2026). It separates its data (Categories, Definitions, Attributes, Recipes) from its rules (Crafting Processors) and its UI, but it is tied to Unity. Many older "Inventory + Crafting" assets have been deprecated or open-sourced and then abandoned (Devdog Inventory Pro, Ultimate Inventory & Crafting 5 Pro, "Inventory System Full/Lite"). That high churn is itself a maturity signal.

### Cited Findings
**Opsive Ultimate Inventory System**
- $90 per seat; 73 reviews; 1,293 favourites. Latest version 1.3.10, released Sep 12, 2026. Supports Unity 2022.1 to Unity 6000.6 on Built-in, URP and HDRP — [Asset Store](https://assetstore.unity.com/packages/tools/game-toolkits/ultimate-inventory-system-166053)
- A search snippet showed 59 ratings and v1.3.8 (Jun 25, 2026), and listed prices between $45 and $90 (probably a sale). This conflicts with the live page above (73 reviews, 1.3.10), so treat the live page as current — [Asset Store reviews](https://assetstore.unity.com/packages/tools/game-toolkits/ultimate-inventory-system-166053/reviews)
- Data model: Item Categories, Item Definitions and Item instances, with inheritance. Attributes are named values that can be inherited or overridden. An Inventory is made of Item Collections (Bag, Equipment). Also covers currency and shops, a save system with a save-system integration, and a UI Designer — [Opsive docs](https://opsive.com/support/documentation/ultimate-inventory-system/)
- Integrations: Behavior Designer Pro, Bolt, Dialogue System, Input System, Master Audio, PlayMaker, Quest Machine, State Designer, Ultimate Character Controller and Unity Localization — [Opsive docs](https://opsive.com/support/documentation/ultimate-inventory-system/)
- Crafting: a Recipe holds ingredients and outputs. An ingredient can be an Item Definition, an Item Category (interchangeable substitutes) or a specific Item (whose stored values must match). Recipes are grouped into Crafting Categories, which can be abstract. A **Crafter** offers categories. A **Crafting Processor** (Simple, or Simple With Currency) checks the inventory, consumes ingredients and adds outputs. Custom processors are the intended extension point for "skill levels, station state, ingredient quality, delayed completion". **There is no built-in timed crafting.** The API is `CanCraft` / `Craft`, returning `Success` and `CraftingOutput`. The UI "only presents and binds" — [Opsive crafting docs](https://opsive.com/support/documentation/ultimate-inventory-system/crafting/)
- Reviews: critics say "couldn't even get the menus to work", "not intuitive menus and easy setup" and "multiple ways this asset can easily break". Fans say it is "very stable and highly capable" and saves writing inventory plus save from scratch. One reviewer called it "a flexible, modular framework rather than a plug-and-play solution", with a learning curve — [Asset Store reviews](https://assetstore.unity.com/packages/tools/game-toolkits/ultimate-inventory-system-166053/reviews)

**Inventory 2 for Game Creator 2 (Catsoft Works)**
- Offers crafting and dismantling, bags, weight, economics and merchants. Requires Unity 6 and Game Creator 2, and is "tightly integrated" with other GC2 modules such as Melee 2. 32 reviews, 406 favourites, about $45 (Gumroad). One reviewer called it "one of the most complex modules" of GC2 — [Asset Store reviews](https://assetstore.unity.com/packages/tools/utilities/inventory-2-game-creator-2-208668/reviews); [Gumroad](https://tigerlily157.gumroad.com/l/Inventory2)

**Devdog Inventory Pro (open-sourced)**
- Devdog says it sold more than 100,000 licences before release under MIT on GitHub. The repo has 711 stars, 167 forks, 28 commits and 9 open issues. The README says the tool is "entirely in the hands of the community", and the authors are "moving on" — [GitHub](https://github.com/devdogio/Inventory-Pro)
- Crafting is inheritable and overridable. "Item Layouts" give multi-slot items (2x2 shield, 1x1 apple). Blueprints come up only as a currency use — [GitHub](https://github.com/devdogio/Inventory-Pro)
- Devdog's reason: it spent more supporting these tools than it earned from them, and wanted to focus on Odin. The tools were withdrawn from the store, with refunds for purchases in the previous 30 days — [GameFromScratch](https://gamefromscratch.com/?p=11047)

**Deprecated store assets (pattern of churn)**
- "Inventory System (Full)" and "(Lite)": deprecated, no longer sold or supported; owners keep access. The search did not confirm that the publisher is Devion Games — [Full](https://assetstore.unity.com/packages/tools/gui/inventory-system-full-126053); [Lite](https://assetstore.unity.com/packages/tools/gui/inventory-system-lite-123129)
- "Ultimate Inventory & Crafting 5 PRO": also deprecated — [Asset Store](https://assetstore.unity.com/packages/tools/gui/ultimate-inventory-crafting-5-pro-19900)
- An unofficial GitHub mirror of a "inventory system Unity" (v2.1.4 preview) lists crafting, currency, vendors and visual scripting. Its authorship is unverified — [GitHub](https://github.com/CyberSys/inventory-system-Unity)

**Survival templates (inventory and crafting bundled into a whole game)**
- Survival Template PRO: $49.99; depends on Input System and Editor Coroutines; integrates Emerald AI 3.0. Reviews are split, from "one of the most well-structured" to "an overcomplicated asset that causes more problems than it solves". The search did not confirm that the publisher is Polymind — [Asset Store](https://assetstore.unity.com/packages/slug/207954); [reviews](https://assetstore.unity.com/packages/templates/systems/survival-template-pro-207954/reviews)
- Survival Engine (Indie Marc): a template with inventory, crafting, building, farming, hunting and a save system. v1.32 (Dec 30, 2024), per a third-party mirror. There is an online (multiplayer) edition with a $49 upgrade discount, and a Steam demo — [Asset Store](https://assetstore.unity.com/packages/templates/systems/survival-engine-crafting-building-farming-178160/reviews); [Unity Discussions](https://discussions.unity.com/t/released-survival-engine-complete-project-template-for-survival-games/809254); [Steam demo](https://store.steampowered.com/app/1507910/Survival_Engine)

### Inferences
- In Unity, "mature" means: ScriptableObject-style databases with category and definition inheritance, typed attributes, recipes as data with pluggable processors, a save integration, a UI designer, and integrations with the publisher's other assets. Lock-in is to Unity and, often, to the vendor's ecosystem (GC2, Opsive UCC).
- Even the leading asset leaves timed crafting, queues and station state to custom code. Factory or automation crafting, which is what Satisfactory-style play needs, is not a first-class feature anywhere in Unity's inventory market.
- The "complete survival template" products draw the most "overcomplicated" and "breaks easily" complaints.

### Gaps
- Average star ratings for UIS, GC2 Inventory and Survival Template PRO did not render. Prices for Survival Template PRO and Survival Engine are unverified.
- UFPS has no confirmed current status. No Unity open-source crafting repos with significant star counts turned up beyond Inventory Pro.

## Unreal Engine: Fab plugins and Epic frameworks

### Takeaway
Epic ships no crafting framework. Lyra gives a **fragment-composition** pattern for item definitions: a data-only definition plus extensible fragments, with equipment kept separate. Third-party systems build on Unreal's replication and data assets. Arc Inventory refines the fragment model (static and dynamic fragments, sub-stacks). The survival templates on Fab (Games by Hyper MST, Easy Survival RPG V5) bundle crafting with whole-game scaffolding, and maturity claims for them are mostly vendor-sourced.

### Cited Findings
- Lyra: an item's array of Inventory Fragments drives its gameplay and presentation. The definition is "composed from extensible inventory fragments and is limited to data properties". Built-in fragments include `InventoryFragment_EquippableItem`, `InventoryFragment_SetStats` (Gameplay-tag stacks such as ammo) and `InventoryFragment_ReticleConfig`. You add new behaviour by writing a new fragment type. Equipment definitions do not reference inventory definitions — [Epic docs](https://dev.epicgames.com/documentation/en-us/unreal-engine/lyra-inventory-and-equipment-in-unreal-engine)
- Community "fixes" to Lyra add a subsystem helper to look up fragments by class, which suggests the stock API is thin — [DTF (Russian)](https://dtf.ru/gamedev/3379073-lyra-inventory-fix-02-podsistema-dlya-fragmentov-inventarya-reshaem-todo-s-umom)
- Arc Inventory (Puny Human): Item Stacks are lightweight UObjects replicated inline. Item Definitions are optional Data Assets holding static fragments, replicated as a pointer. Dynamic fragments live on the stack and can replicate. Stat Tags map Tag to Integer. Sub Item Stacks support attachments and perks, and World Stacks place items in the world. The docs carry a 2020–2023 copyright; licence and UE version are not stated — [Arc docs](https://docs.punyhuman.com/arcinventory/concepts/itemstack/)
- Narrative Inventory (Reuben Ward): now 100% free on Fab. Items are Blueprints; the inventory is a component on PlayerState with networking built in. It integrates Narrative 3 quest tasks (Find, Equip, Use Item). No built-in crafting was found — [Fab](https://www.fab.com/listings/12f51c14-0c13-4ca8-b9d5-96ea1d09cd72)
- One developer built a custom C++ crafting module on Narrative Pro (UE 5.3) with data-asset recipes (ingredients, outputs, craft time). This shows that crafting is commonly a bolt-on — [itch devlog](https://bardwynn.itch.io/bardwynn/devlog/909976/crafting-update)
- Games by Hyper MST (Starter, Plus, Pro V4): MST Starter includes inventory, crafting, hotbar, vendors and loot chests. Crafting checks the inventory for ingredients. World stations put finished items into their own inventory component, which the player collects; player crafting adds directly to the player inventory. Full replication. Sources are vendor-only — [MST Starter](https://gamesbyhyper.com/product/mst-starter/); [Hyper docs](https://gamesbyhyper.com/docs/project-foundations/template-logic-and-flexibility/); [Hyper List Inventory on Fab](https://www.fab.com/listings/5635f78b-0e27-43d0-ab45-38f8e3ea5b32)
- Easy Survival RPG V5: multiplayer, with **queue-based crafting**, durability, containers and backpacks, and dynamic loot and resource spawning. No reviews or UE version were visible — [Fab](https://www.fab.com/listings/bc8e1ee8-5b82-40df-ad2c-226524ca6253)
- Several older Marketplace crafting plugins ("Survival RPG Inventory & Equipment System", "VR Inventory Plugin") have **not been migrated to Fab** and are usable only from Epic Launcher libraries. This is a visible attrition signal — [UE Marketplace](https://www.unrealengine.com/marketplace/zh-CN/product/survival-rpg-inventory-equipment-system/reviews); [VR Inventory](https://www.unrealengine.com/marketplace/en-US/product/vr-inventory-plugin/reviews)
- Other crafting listings exist ("Multiplayer Crafting System", "Survival Crafting System"), but no details were retrieved — [Multiplayer Crafting System](https://www.unrealengine.com/marketplace/en-US/product/multiplayer-crafting-system/reviews); [Survival Crafting System](https://www.unrealengine.com/marketplace/en-US/product/survival-crafting-system)

### Inferences
- In Unreal, "mature" means replicated, server-authoritative inventory, data-asset or DataTable definitions, and composition by fragments or Gameplay Tags (GAS-adjacent). Crafting stations are modelled as actors that own their own inventory (MST). That pattern maps well onto factory machines.
- Multiplayer replication is the main differentiator in Unreal, and it is also why the logic is entangled with UObject and the actor model.

### Gaps
- No review counts or ratings were retrieved for Fab listings (Fab pages render poorly). Licence and engine versions for Arc Inventory are unknown. No GAS-specific crafting framework was found. Fab prices are unverified.

## Godot: Asset Library and GitHub add-ons

### Takeaway
Godot's options are open source (MIT) and moderately popular, at about 700–1,100 stars each. **expressobits Inventory System** is the most complete inventory-plus-crafting add-on: C++ GDExtension, CraftStation nodes, recipes with consumed and required items. **GLoot** is a clean inventory core with constraints and JSON prototypes but no crafting. **Pandora** is a general RPG data framework that labels itself alpha and not production-ready.

### Cited Findings
- GLoot (peter-kish): 990 stars, 46 forks, MIT; Godot 4.4+ (v3.0 on master, not backward-compatible with 2.x). `InventoryItem` is a stack whose properties come from a JSON "prototree". Constraints include `GridConstraint`, `WeightConstraint` and `ItemCountConstraint`, plus `ItemSlot`. `serialize()` and `deserialize()` go to and from dictionaries. UI controls (`CtrlInventory`, `CtrlInventoryGrid` and others) are separate and meant mainly for prototyping. No crafting — [GitHub](https://github.com/peter-kish/gloot)
- The GLoot Asset Library entry is v3.0.1 for Godot 4.4, MIT. A third-party tracker earlier showed 912 stars and 27 open issues — [Godot Asset Library](https://godotengine.org/asset-library/asset/edit/13343); [gittrend](https://gittrend.io/repo/peter-kish/gloot)
- expressobits Inventory System: 739 stars, 67 forks, MIT, plugin version badge 2.13.0. Features: item stacks, crafting and autocrafts, hotbar, loot, multiplayer, Item Editor, categories, slot, grid and equipment inventories. The core is a C++ plugin (godot-cpp, CMake or SCons) — [GitHub](https://github.com/expressobits/inventory-system)
- Its recipes are resources that define requirements and outcomes, used at **CraftStation** nodes. Ingredients are consumed; "required items" such as a hammer are needed but not consumed; products are the outputs. The docs claim C++ performance without recompiling Godot. Branches: `main` (C++ core), `addon` (Asset Library build) and `module` — [expressobits docs](https://expressobits.com/inventory-system/getting_started/introduction.html)
- The Asset Library lists version 0.8.62 for Godot 4.4, MIT. That conflicts with the 2.13.0 GitHub badge, so the Asset Library entry is probably stale — [Godot Asset Library](https://godotengine.org/asset-library/asset/edit/8972)
- Pandora (bitbrain): about 1.1k stars, 41 forks, MIT, Godot 4 (branch `godot-4.x`). It manages items, inventories, spells, mobs, quests, NPCs and loot tables through an editor UI and a `Pandora` singleton, with unit tests. The README says "STILL IN ALPHA AND NOT PRODUCTION-READY". Recipes are not mentioned — [GitHub](https://github.com/bitbrain/pandora)

### Inferences
- Godot add-ons are leaner and closer to engine-agnostic in spirit. GLoot keeps logic and UI apart, and expressobits keeps a C++ core. All of them still depend on Godot's Resource and Node types.
- expressobits' consumed versus required split is a commonly needed recipe feature that Opsive does not model explicitly.

### Gaps
- No release dates were visible (GitHub Releases did not render). No `godot-craft` or dedicated crafting-only add-on was found. I could not confirm whether expressobits supports timed crafting or crafting queues.

## Common feature set, differentiators and failure modes

### Takeaway
Every serious system has: item definitions separate from instances, categories or tags, stacking and max count, typed properties or attributes, constraints (slots, grid, weight), recipes as data (ingredients to outputs), serialization, and UI separate from logic. The differentiators are replication, category-based ingredient substitution, catalysts and tools (required but not consumed), currency costs, stations that own an output inventory, queued or timed crafting, and editor tooling. The recurring complaints are steep setup, over-complication or bloat in "complete" templates, ecosystem lock-in, and abandonment or deprecation.

### Cited Findings
- Definition versus instance with overridable properties: Opsive (inheritance, attributes), Arc (static vs dynamic fragments), Lyra (data-only definition plus fragments), Hiro (config defaults vs instance properties) — [Opsive](https://opsive.com/support/documentation/ultimate-inventory-system/); [Arc](https://docs.punyhuman.com/arcinventory/concepts/itemstack/); [Lyra](https://dev.epicgames.com/documentation/en-us/unreal-engine/lyra-inventory-and-equipment-in-unreal-engine); [Hiro](https://heroiclabs.com/docs/hiro/concepts/inventory/llm.md)
- Category or tag substitution in recipes: Opsive accepts Item Category ingredients — [Opsive crafting](https://opsive.com/support/documentation/ultimate-inventory-system/crafting/)
- Required but not consumed tools: expressobits — [docs](https://expressobits.com/inventory-system/getting_started/introduction.html)
- Stations with their own output inventory: Games by Hyper — [docs](https://gamesbyhyper.com/docs/project-foundations/template-logic-and-flexibility/)
- Queue-based crafting: Easy Survival RPG V5 — [Fab](https://www.fab.com/listings/bc8e1ee8-5b82-40df-ad2c-226524ca6253). Timed crafting is not built into Opsive; you need a custom processor — [Opsive crafting](https://opsive.com/support/documentation/ultimate-inventory-system/crafting/)
- Failure modes: setup difficulty and fragility (UIS reviews); "overcomplicated" (Survival Template PRO reviews); ecosystem lock-in (GC2 Inventory is "tightly integrated"); abandonment (Inventory Pro handed to the community; several assets deprecated; Marketplace listings not migrated to Fab); alpha status (Pandora) — sources as cited in the sections above.

### Inferences
- "Mature" here means years of maintenance, a database plus editor, a pluggable crafting processor, save integration and documentation. Very few products meet all of these. Opsive UIS is the clearest example. expressobits is the nearest open-source equivalent.

### Gaps
- No systematic evidence was found on performance with many items, or on save-format compatibility across versions, in reviews. These complaints are commonly reported anecdotally, but I found no citable source.

## Engine-agnostic cores: which separate logic from presentation?

### Takeaway
No widely used, mature, open-source, engine-agnostic crafting library turned up. The engine-agnostic offerings are small, early C# libraries (Kangaskhan, The Chest) with no crafting, or server-backed services (InventoryFramework over gRPC, Heroic Labs Hiro). Within engines, Opsive, GLoot and expressobits separate logic from UI, but none can be used outside its engine. The niche is open: a pure-core, data-driven recipe and inventory library, especially one designed for factory or automation crafting.

### Cited Findings
- Kangaskhan: C# inventory library with "no dependency on any game engine", MIT, .NET Standard 2.0, "initial stage", minimal features, no crafting — [GitHub](https://github.com/Rinkaa/Kangaskhan)
- The Chest: C# container and inventory library (TheChest.Core) that aims to be usable from most C# engines. The repos are quiet and no crafting was seen — [GitHub](https://github.com/The-Chest)
- InventoryFramework.SDK: a netstandard2.1 gRPC client to a server-authoritative inventory and crafting service with preview-craft, execute, browse-recipes and craftable-now APIs. It targets Unity, Godot and Unreal. Licensing is tiered, and source code is a paid feature — [NuGet](https://www.nuget.org/packages/inventoryframework.sdk); [Unity adapter](https://www.nuget.org/packages/InventoryFramework.UnityAdapter/1.0.1)
- Heroic Labs Hiro: server-side JSON inventory config (category, item_sets, max_count, stackable, consumable, string and numeric properties, keep_zero, disabled). Consume rewards support weighted random picks. Category and set limits apply. There is no dedicated crafting system; crafting is expressed as consuming materials. The page references a Unity sample — [Hiro docs](https://heroiclabs.com/docs/hiro/concepts/inventory/llm.md)
- Unity Gaming Services Economy has an `InventoryItemDefinition` model (a backend service) — [Unity docs](https://docs.unity.com/ugs/en-us/packages/com.unity.services.economy/3.1/api/Unity.Services.Economy.Model.InventoryItemDefinition)
- Logic and UI separation within engines: in Opsive the UI "only presents and binds" while rules live in recipes and processors — [Opsive crafting](https://opsive.com/support/documentation/ultimate-inventory-system/crafting/). GLoot's Ctrl* UI nodes are optional over logic classes — [GitHub](https://github.com/peter-kish/gloot). expressobits keeps a C++ core with a separate addon layer — [GitHub](https://github.com/expressobits/inventory-system)

### Inferences
- The best patterns to copy into an engine-agnostic core:
  - Opsive's split of recipe data from the processor, plus category ingredients
  - expressobits' consumed versus required items
  - Arc and Lyra's static versus dynamic fragments
  - GLoot's constraint objects and dictionary serialization
  - Hiro's config-defaults-plus-instance-overrides and keep_zero, which matches the AGENTS.md rule that a learned recipe persists at zero inventory
  - Hyper's stations that own an output inventory
- None of the surveyed systems models continuous, throughput-based factory crafting (machines with rates feeding one another). They all model discrete, player-initiated crafts.

### Gaps
- Roblox: only inventory frameworks were found (InventoryMaker, open source; OpenInventory, archived Jun 30, 2024), with no notable crafting library — [InventoryMaker](https://devforum.roblox.com/t/inventorymaker-a-framework-to-help-you-create-your-own-inventory-system/3619173); [OpenInventory](https://github.com/XnLogicaL/OpenInventory)
- GameMaker and Defold: no notable crafting or inventory library was found — [GameMaker asset collection](https://github.com/Pikku-a/gamemaker-assets). Defold was not searched specifically.
- Rust or Bevy crates were out of scope and were not searched.
