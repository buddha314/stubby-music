## 1. Launcher
- [x] 1.1 Dev builds start `stubby_rules_server` and open its page unless it answers on 127.0.0.1:7878 or `STUBBY_NO_DESIGNER` is set; `tools/jackdaw_timed_test.py` sets it

## 2. Rules (rule PR)
- [x] 2.1 `machines` registry in `stubby_recipes`: rows `{id, targets, clock_reach_m?}`, shape validation, schema (`rules-fmt`), editing model, `ids.lock`
- [x] 2.2 `machines.json` with the campfire's gate targets and reach; remove `configure.targets` and `clock_reach_m` from profiles and `create_campfire.py`

## 3. Game
- [x] 3.1 Glue: copy rows into `bevy_stubby_rack::Profiles` on load and reload; the campfire reads its reach from the row
- [x] 3.2 `RackTargets { met, missed }`, reflected; `PlacedAsset` and `MachineVoice` reflected
- [x] 3.3 Test checks every row's targets against shipped profiles and modules (`stubby_game` glue, where both are known)

## 4. Designer
- [x] 4.1 Machines panel in the live view: instance, sounding, targets met or the missed list
- [x] 4.2 Smoke test: retune the campfire's gates in the designer while the game runs and watch the panel (also `add-crafting-designer` 4.2). Done 2026-11-18 by a bounded run: with gate 4 off the panel's query showed `seq.gate_4 at 0`; editing `machines.json` mid-run reloaded the rules, the targets were met and the camp lit within two bars, with no rebuild
