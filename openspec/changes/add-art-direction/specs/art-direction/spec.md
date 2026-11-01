## ADDED Requirements

### Requirement: Living style guide
The repo SHALL keep an art style guide at `docs/art/style-guide.md` stating the theme
(the music of nature brings light to the world), the bleak-to-bloomed arc, the
palette, scale, legibility rules, material rules, naming, pipeline, budgets and a
review checklist. Any change to the look SHALL update the guide and add a dated line
to its changelog.

#### Scenario: Look changes
- **WHEN** a lighting key, palette colour or asset rule changes
- **THEN** the style guide reflects it and its changelog has a dated entry

### Requirement: Art-authored lighting keys
Lighting for the day/night cycle and the bleak-to-bloomed blend SHALL be authored as
data in `assets/lighting/lighting_keys.json`: one key per mood (bleak, bloomed) and
time (dawn, day, dusk, night), each giving sun colour, illuminance and elevation,
ambient colour and illuminance, fog colour and density, sky colours, exposure and
emissive gain, with colours in sRGB. Each mood SHALL give a ground tint and accent
colours. The file SHALL carry a `version` that changes with every edit.

#### Scenario: Keys are complete
- **WHEN** the keys file is loaded
- **THEN** every mood has all four time keys and every key has every field

### Requirement: Signal colour is constant
Module signal light (lamps, LEDs, glyphs, displays) SHALL use the amber signal
palette in every lighting key. Accent colours belong to flora and the bloomed world
and SHALL NOT be used for module signal light.

#### Scenario: Module under lunarpunk night
- **WHEN** a module is rendered under the bloomed night key
- **THEN** its lights are amber and remain legible against the violet sky and accent flora

### Requirement: Style sheet review
`tools/blender/render_style_sheet.py` SHALL render every exported module under every
lighting key into one sheet, `docs/art/style_sheet.png`. A new asset or key change
SHALL be reviewed on that sheet before it merges.

#### Scenario: New key added
- **WHEN** a key is edited
- **THEN** the style sheet is regenerated and each module's waveform still reads in every tile

### Requirement: Assets are generated
Every shipped model SHALL be produced by a script in `tools/blender/` that rebuilds it
headless from a factory-startup Blender, writing the `.blend`, the `.glb` and, for
modules, the profile and previews. Shipped models SHALL NOT be hand-edited.

#### Scenario: Regenerate after a style change
- **WHEN** a generator is re-run
- **THEN** it reproduces its asset files without manual steps

### Requirement: Growth reads as a response to music
Flora and fauna assets SHALL show, through game-drivable nodes or materials, that they
are being played to: a visible reaction to nearby music, and growth stages from bleak
to bloomed. Their bloomed state SHALL use the accent palette.

#### Scenario: A stream feeding a grove
- **WHEN** a stream carries music to plants
- **THEN** it reads as a musical staff with notes flowing along it toward them

#### Scenario: Scrub oak being played to
- **WHEN** music plays near a scrub oak
- **THEN** the asset exposes parts the game can animate in time with it, and its growth stages run from bare to bloomed
