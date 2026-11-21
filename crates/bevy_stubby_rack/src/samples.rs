//! Select and audition collected clips from the rack screen.

use crate::rack::{RackEngine, open_rack};
use bevy::picking::{Pickable, events::PointerClick, pointer::PointerButton};
use bevy::{
    input::{InputSystems, keyboard::KeyboardInput},
    prelude::*,
};
use bevy_stubby_audio::Synth;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use stubby_audio::{Command, SampleId};
use stubby_core::ScreenMode;

/// Stable catalog IDs are saved by rack place. Engine IDs are runtime handles.
#[derive(Component, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RackSamples(pub BTreeMap<String, String>);

pub struct Clip {
    pub engine_id: SampleId,
    pub name: String,
    pub source: String,
    pub seconds: f32,
}

#[derive(Resource, Default)]
pub struct SampleLibrary(pub BTreeMap<String, Clip>);

#[derive(Resource, Default)]
pub(crate) struct Picker {
    target: Option<(Entity, String)>,
    filter: String,
    row: usize,
    preview_until: f64,
}

impl Picker {
    pub(crate) fn is_open(&self) -> bool {
        self.target.is_some()
    }
}

#[derive(Component)]
struct SampleControls;

#[derive(Component)]
pub(crate) struct SampleActions;

#[derive(Component, Clone)]
enum Action {
    Open(String),
    Assign(String),
    Preview(String),
    Move(i32),
    Clear,
    Close,
}

pub(crate) struct SamplePickerPlugin;

impl Plugin for SamplePickerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SampleLibrary>()
            .init_resource::<Picker>()
            .add_observer(click_action)
            .add_systems(PreUpdate, picker_shortcuts.after(InputSystems))
            .add_systems(
                Update,
                (keyboard_actions, draw_controls)
                    .chain()
                    .after(stubby_core::MachineInteractionSet),
            );
    }
}

fn picker_shortcuts(mut keys: ResMut<ButtonInput<KeyCode>>, mut picker: ResMut<Picker>) {
    if picker.is_open() {
        // The typed E goes to search, while the rack's E shortcut is paused.
        keys.clear_just_pressed(stubby_core::INTERACT);
        keys.clear_just_pressed(stubby_core::RECIPE_BOOK);
        if keys.clear_just_pressed(KeyCode::Escape) {
            picker.target = None;
        }
    }
}

#[derive(bevy::ecs::system::SystemParam)]
struct Controls<'w, 's> {
    picker: ResMut<'w, Picker>,
    screens: Res<'w, ScreenMode>,
    library: Res<'w, SampleLibrary>,
    synth: Option<Res<'w, Synth>>,
    time: Res<'w, Time<Real>>,
    racks: Query<'w, 's, (&'static RackEngine, &'static mut RackSamples)>,
}

impl Controls<'_, '_> {
    fn apply(&mut self, action: Action) {
        let Some(machine) = open_rack(&self.screens) else {
            return;
        };
        let Ok((engine, mut samples)) = self.racks.get_mut(machine) else {
            return;
        };
        match action {
            Action::Open(place) if engine.sample_places().any(|key| key == place) => {
                self.picker.target = Some((machine, place));
                self.picker.filter.clear();
                self.picker.row = 0;
            }
            Action::Open(_) => {}
            Action::Preview(id) => {
                let now = self.time.elapsed_secs_f64();
                if let Some(synth) = &self.synth
                    && let Some(clip) = self.library.0.get(&id)
                    && now >= self.picker.preview_until
                {
                    synth.send(Command::PlaySample {
                        sample: clip.engine_id,
                        gain: 0.8,
                    });
                    self.picker.preview_until = now + f64::from(clip.seconds);
                }
            }
            Action::Move(offset) => {
                let end = matching(&self.library, &self.picker.filter)
                    .len()
                    .saturating_sub(1);
                self.picker.row = self
                    .picker
                    .row
                    .saturating_add_signed(offset as isize)
                    .min(end);
            }
            Action::Close => self.picker.target = None,
            Action::Assign(_) | Action::Clear => {
                let Some((owner, place)) = self.picker.target.take() else {
                    return;
                };
                if owner != machine {
                    return;
                }
                match action {
                    Action::Assign(id) if self.library.0.contains_key(&id) => {
                        samples.0.insert(place, id);
                    }
                    Action::Clear => {
                        samples.0.remove(&place);
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Reapply selections after a patch rebuild or when a saved clip becomes available.
pub(crate) fn bind_selections(
    synth: Option<Res<Synth>>,
    library: Res<SampleLibrary>,
    racks: Query<(Ref<RackEngine>, Ref<RackSamples>)>,
) {
    let Some(synth) = synth.filter(|synth| synth.is_running()) else {
        return;
    };
    for (engine, samples) in &racks {
        if !engine.is_changed() && !samples.is_changed() && !library.is_changed() {
            continue;
        }
        for place in engine.sample_places() {
            let sample = samples.0.get(place).and_then(|id| library.0.get(id));
            engine.select_clip(&synth, place, sample.map(|clip| clip.engine_id));
        }
    }
}

fn matching<'a>(library: &'a SampleLibrary, filter: &str) -> Vec<(&'a str, &'a Clip)> {
    let filter = filter.to_lowercase();
    let mut clips: Vec<_> = library
        .0
        .iter()
        .filter(|(_, clip)| {
            clip.name.to_lowercase().contains(&filter)
                || clip.source.to_lowercase().contains(&filter)
        })
        .map(|(id, clip)| (id.as_str(), clip))
        .collect();
    clips.sort_by(|(a_id, a), (b_id, b)| a.name.cmp(&b.name).then(a_id.cmp(b_id)));
    clips
}

fn keyboard_actions(mut events: MessageReader<KeyboardInput>, mut controls: Controls) {
    for event in events.read() {
        if !controls.picker.is_open() || !event.state.is_pressed() {
            continue;
        }
        match event.key_code {
            KeyCode::ArrowUp => controls.apply(Action::Move(-1)),
            KeyCode::ArrowDown => controls.apply(Action::Move(1)),
            KeyCode::Backspace => {
                controls.picker.filter.pop();
                controls.picker.row = 0;
            }
            KeyCode::Enter => {
                if let Some((id, _)) =
                    matching(&controls.library, &controls.picker.filter).get(controls.picker.row)
                {
                    controls.apply(Action::Assign((*id).into()));
                }
            }
            _ => {
                if let Some(text) = &event.text {
                    let remaining = 64_usize.saturating_sub(controls.picker.filter.len());
                    controls
                        .picker
                        .filter
                        .extend(text.chars().filter(|c| !c.is_control()).take(remaining));
                    controls.picker.row = 0;
                }
            }
        }
    }
}

fn click_action(click: On<PointerClick>, actions: Query<&Action>, mut controls: Controls) {
    if click.button == PointerButton::Primary
        && let Ok(action) = actions.get(click.entity)
    {
        controls.apply(action.clone());
    }
}

fn label(parent: &mut ChildSpawnerCommands, text: impl Into<String>) {
    parent.spawn((
        Text::new(text),
        Pickable::IGNORE,
        TextFont {
            font_size: FontSize::Px(15.),
            ..default()
        },
    ));
}

fn button(
    parent: &mut ChildSpawnerCommands,
    text: impl Into<String>,
    action: Action,
    selected: bool,
) {
    let color = if selected {
        Color::srgb(0.40, 0.25, 0.12)
    } else {
        Color::srgb(0.15, 0.14, 0.13)
    };
    parent
        .spawn((
            action,
            BackgroundColor(color),
            Node {
                padding: UiRect::all(px(8)),
                ..default()
            },
        ))
        .with_children(|button| label(button, text));
}

fn place_label(place: &str) -> String {
    place
        .split('_')
        .map(str::to_uppercase)
        .collect::<Vec<_>>()
        .join(" ")
}

fn draw_controls(
    mut commands: Commands,
    screens: Res<ScreenMode>,
    library: Res<SampleLibrary>,
    mut picker: ResMut<Picker>,
    racks: Query<(Ref<RackEngine>, Ref<RackSamples>)>,
    roots: Query<Entity, With<SampleControls>>,
    containers: Query<Entity, With<SampleActions>>,
    mut shown: Local<Option<Entity>>,
) {
    let Ok(container) = containers.single() else {
        return;
    };
    let machine = open_rack(&screens);
    if picker
        .target
        .as_ref()
        .is_some_and(|(owner, _)| Some(*owner) != machine)
    {
        picker.target = None;
    }
    let rack = machine.and_then(|entity| racks.get(entity).ok());
    if *shown == machine
        && !picker.is_changed()
        && !library.is_changed()
        && !rack
            .as_ref()
            .is_some_and(|(engine, samples)| engine.is_changed() || samples.is_changed())
    {
        return;
    }
    *shown = machine;
    for root in &roots {
        commands.entity(root).despawn();
    }
    let Some((engine, samples)) = rack else {
        return;
    };
    let places: Vec<_> = engine.sample_places().collect();
    if places.is_empty() {
        return;
    }
    let open = picker.target.as_ref().map(|(_, place)| place.as_str());
    commands
        .spawn((
            SampleControls,
            ChildOf(container),
            BackgroundColor(Color::srgb(0.09, 0.08, 0.07)),
            Node {
                width: percent(100),
                padding: UiRect::all(px(8.)),
                row_gap: px(6.),
                flex_direction: FlexDirection::Column,
                ..default()
            },
        ))
        .with_children(|panel| {
            if let Some(place) = open {
                label(panel, format!("Choose sound for {}", place_label(place)));
                label(panel, format!("Search clips  {}", picker.filter));
                let matches = matching(&library, &picker.filter);
                if matches.is_empty() {
                    label(panel, "No collected clips match.");
                }
                if matches.len() > 6 {
                    panel
                        .spawn(Node {
                            column_gap: px(6.),
                            ..default()
                        })
                        .with_children(|row| {
                            button(row, "Previous", Action::Move(-6), false);
                            button(row, "Next", Action::Move(6), false);
                        });
                }
                let start = picker.row / 6 * 6;
                for (index, (id, clip)) in matches.into_iter().enumerate().skip(start).take(6) {
                    panel
                        .spawn(Node {
                            column_gap: px(5.),
                            ..default()
                        })
                        .with_children(|row| {
                            button(
                                row,
                                format!("{}  {:.2} s", clip.name, clip.seconds),
                                Action::Assign(id.into()),
                                index == picker.row,
                            );
                            button(row, "Preview", Action::Preview(id.into()), false);
                        });
                    label(panel, &clip.source);
                }
                button(panel, "Clear selection", Action::Clear, false);
                button(panel, "Close", Action::Close, false);
            } else {
                for place in places {
                    let current = samples.0.get(place).map_or("Choose sound", |id| {
                        library
                            .0
                            .get(id)
                            .map_or("Unavailable clip", |clip| clip.name.as_str())
                    });
                    button(
                        panel,
                        format!("{}  /  {current}", place_label(place)),
                        Action::Open(place.into()),
                        false,
                    );
                }
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sampler_picker_assigns_two_places_and_clears_only_the_open_place() {
        use bevy::ecs::system::RunSystemOnce;

        let mut world = World::new();
        let machine = world
            .spawn((RackEngine::default(), RackSamples::default()))
            .id();
        world.insert_resource(ScreenMode {
            open: Some(stubby_core::Screen::MachineConfiguration(machine)),
            ..default()
        });
        world.insert_resource(Time::<Real>::default());
        world.insert_resource(Picker::default());
        world.insert_resource(SampleLibrary(BTreeMap::from([
            (
                "rock".into(),
                Clip {
                    engine_id: SampleId(7),
                    name: "Rock hit".into(),
                    source: "Stone".into(),
                    seconds: 0.1,
                },
            ),
            (
                "metal".into(),
                Clip {
                    engine_id: SampleId(8),
                    name: "Metal click".into(),
                    source: "Latch".into(),
                    seconds: 0.1,
                },
            ),
        ])));

        world
            .run_system_once(move |mut controls: Controls| {
                controls.picker.target = Some((machine, "a".into()));
                controls.apply(Action::Assign("rock".into()));
                controls.picker.target = Some((machine, "b".into()));
                controls.apply(Action::Assign("metal".into()));
            })
            .unwrap();
        assert_eq!(world.get::<RackSamples>(machine).unwrap().0["a"], "rock");
        assert_eq!(world.get::<RackSamples>(machine).unwrap().0["b"], "metal");

        world
            .run_system_once(move |mut controls: Controls| {
                controls.picker.target = Some((machine, "a".into()));
                controls.apply(Action::Clear);
            })
            .unwrap();
        let selections = &world.get::<RackSamples>(machine).unwrap().0;
        assert!(!selections.contains_key("a"));
        assert_eq!(selections["b"], "metal");
    }

    #[test]
    fn sampler_assignments_round_trip_by_place_and_catalog_id() {
        let saved = RackSamples(BTreeMap::from([
            ("sampler_a".into(), "rock_wood".into()),
            ("sampler_b".into(), "metal_click".into()),
        ]));
        let text = serde_json::to_string(&saved).unwrap();
        let restored: RackSamples = serde_json::from_str(&text).unwrap();
        assert_eq!(restored.0, saved.0);
        assert!(
            serde_json::from_str::<RackSamples>("{}")
                .unwrap()
                .0
                .is_empty()
        );
    }
}
