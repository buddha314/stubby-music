//! The white outline on the pickup E is aimed at (sync-targeting-and-compass, after
//! Satisfactory and the Godot game's `add-collectible-outline`), drawn by
//! `bevy_mod_outline`'s jump-flood outlines: screen-space, so thin reeds and small shells
//! get the same width in pixels. Only the current target is outlined, and nothing while a
//! screen or build mode has the controls. Style: `assets/ui/collectible_outline.json`.
use bevy::prelude::*;
use bevy_mod_outline::{OutlinePlugin, OutlineVolume, PropagateOutline};
use stubby_core::{BuildMode, InteractTarget, Interactable, ScreenMode};

#[derive(serde::Deserialize)]
struct StyleFile {
    color: [f32; 3],
    width_px: f32,
}

/// The outline's colour and width in screen pixels.
#[derive(Resource, Clone, Copy, Debug)]
pub struct OutlineStyle {
    pub color: Color,
    pub width_px: f32,
}
impl OutlineStyle {
    pub fn shipped() -> Self {
        let file: StyleFile =
            serde_json::from_str(include_str!("../../../assets/ui/collectible_outline.json"))
                .expect("collectible outline style");
        let [r, g, b] = file.color;
        Self {
            color: Color::srgb(r, g, b),
            width_px: file.width_px,
        }
    }
}

/// On the pickup currently outlined.
#[derive(Component)]
struct Outlined;

pub(crate) fn add(app: &mut App) {
    // Headless apps (tests) have no renderer to draw outlines with.
    if app.is_plugin_added::<bevy::render::RenderPlugin>()
        && !app.is_plugin_added::<OutlinePlugin>()
    {
        app.add_plugins(OutlinePlugin::EXTRUDE_VERTEX);
    }
    app.insert_resource(OutlineStyle::shipped()).add_systems(
        Update,
        outline_target.after(stubby_core::InteractionTargeting),
    );
}

/// The pickup to outline: the target, if it is still a pickup and nothing else has the
/// controls.
fn shown(
    target: InteractTarget,
    available: impl Fn(Entity) -> bool,
    screen_open: bool,
    building: bool,
) -> Option<Entity> {
    target
        .pickup()
        .filter(|pickup| available(*pickup) && !screen_open && !building)
}

fn outline_target(
    mut commands: Commands,
    target: Res<InteractTarget>,
    screens: Option<Res<ScreenMode>>,
    build: Option<Res<BuildMode>>,
    style: Res<OutlineStyle>,
    outlined: Query<Entity, With<Outlined>>,
    pickups: Query<(), With<Interactable>>,
) {
    let show = shown(
        *target,
        |pickup| pickups.contains(pickup),
        screens.is_some_and(|screens| screens.open.is_some()),
        build.is_some_and(|build| build.0),
    );
    for entity in &outlined {
        if Some(entity) != show {
            commands
                .entity(entity)
                .try_remove::<(OutlineVolume, PropagateOutline, Outlined)>();
        }
    }
    if let Some(entity) = show
        && !outlined.contains(entity)
    {
        commands.entity(entity).insert((
            OutlineVolume {
                visible: true,
                width: style.width_px,
                colour: style.color,
            },
            // Every mesh below the pickup inherits it: the outline follows its LODs and parts.
            PropagateOutline,
            Outlined,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_an_available_target_is_outlined_and_never_behind_a_screen_or_in_build_mode() {
        let pickup = Entity::from_raw_u32(7).unwrap();
        let target = InteractTarget::Pickup(pickup);
        assert_eq!(shown(target, |_| true, false, false), Some(pickup));
        assert_eq!(shown(target, |_| false, false, false), None, "collected");
        assert_eq!(
            shown(target, |_| true, true, false),
            None,
            "a screen is open"
        );
        assert_eq!(shown(target, |_| true, false, true), None, "build mode");
        let machine = InteractTarget::Machine(pickup);
        assert_eq!(shown(machine, |_| true, false, false), None);
        assert_eq!(OutlineStyle::shipped().width_px, 2.);
    }
}
