# Long-session design previews

**Open.** Review scenarios for the [mudflat-to-city arc](design-guide.md#the-arc).
Use milestones as review stops. Their trigger rules and play-time budgets are
open. See [the simulation research](../research/long-session-simulation-previews.md)
for the methods behind these scenarios.

Compare the four [Jackdaw phase scenes](../../assets/world/phase_reviews/README.md)
from the same opening view. Their content and the remaining design decisions
are tracked in [OpenSpec](../../openspec/changes/add-settlement-phase-reviews/design.md).

## What a long session could feel like

| Review stop | What Omar sees and hears | What the player has made possible |
| --- | --- | --- |
| Camp | The vardo sits above a mud channel. A campfire plays on the backbeat. From the far bank, haze still hides most of the camp. | A running heartbeat carries a first stream. The player can follow it by ear back to a machine they patched. |
| Grove | Mud enters the first check dam. Water downstream lightens in steps, and the first oak has leafed out. Closer to the grove the sky reads differently, even at the same solar hour. | The player found the ingredients, laid the route, and tuned the filters. From the channel and the sound, they can tell which dam is working. |
| Village | Returning to camp at dusk, Omar can follow a stream toward a lived-in place. Its music changes along the way, and there are still hazy stretches beyond tended ground. | Earlier work remains legible. The player can walk upstream and recognize the burbler and first dam underneath a larger arrangement. |
| City | From an overlook he can still pick out the route of the first stream, now part of a city that lights up at night. Walking the route brings individual patches into earshot. | The settlement feels like an accumulation of the player's decisions. Its water, sound and light tell a coherent story from the overlook and from the street. |

The grove is the first scenario the current systems can substantiate.
Village and City use authored architectural blockouts. Preserve a
recognizable landmark, the early route, and a matching camera at each stop so
the transformation stays legible. Pair a fixed view at the same solar hour
with a short walk at normal speed to judge how the world sounds and feels.

## First experiment

Set up a seed-42 grove with a sounding burbler, a connected stream, three dam
sites and oaks upstream and downstream. Run two paths from the same setup.
In the baseline, leave the dams unpatched. In the second, tune the three
dams in sequence, then disconnect the middle dam. Hold the weather and solar
hour fixed in the comparison captures. Watch the water below each dam, the
oaks' growth stage and local air quality, and whether the rack and stream
provide enough information for the player to explain what changed.

- **Script player actions.** Add a short sequence of semantic actions such as place, patch, tune and disconnect using the existing gameplay paths where available. Record any fixture setup, such as granting materials, separately from the actions. Actions belong at specific simulation times or milestones.
- **Run world evolution headlessly.** Step through the same growth and clarity rules at bounded intervals, including across a day/night boundary. Preserve event ordering and run a short matched interval at 1× as a reference. Supply a controlled clock for world tests and verify real rack timing separately through offline audio and the live listen.
- **Record the cause and the outcome.** Keep the seed, scene and code revision, action sequence and elapsed world time. At each checkpoint record clarity above and below each dam, growth stage, local air quality and the active patch. Record player or scripted actions alongside simulation time so an hour of unattended evolution and an hour of accumulated player decisions remain distinguishable.
- **Review the result in Jackdaw.** Begin from a prepared scene and inspect it at normal speed under the existing 20-second watchdog. Use the headless run for long horizons. Once checkpoints can be restored, compare uninterrupted and resumed runs over the same short stretch and check for duplicated rewards or lost connections.

The first slice can live in the existing review-driver and headless-test
workflows. A general save or replay system can wait until designers have a
checkpoint they want to revisit from several starting conditions.
