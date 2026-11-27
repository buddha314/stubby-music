//! The player's settings (add-multiplayer-and-menus 1.3), ported from the
//! us-outpost-31 template: display and volumes, saved per user by Bevy's own
//! `bevy_settings` as TOML in the OS's preferences folder
//! (`~/.config/<APP_ID>/settings.toml` on Linux, `%LocalAppData%\<APP_ID>\` on Windows).
//!
//! Each group is an ordinary resource: change it, then call [`save`]. The change applies
//! at once ([`SettingsPlugin`] pushes it to the window, the lights and the audio engine's
//! volume stages). Only the binaries load and write the file
//! ([`SettingsPersistencePlugin`]), so tests never touch a player's settings.

use core::time::Duration;

use bevy::{
    prelude::*,
    settings::{ReflectSettingsGroup, SaveSettingsDeferred, SaveSettingsSync, SettingsGroup},
    window::{MonitorSelection, PresentMode, PrimaryWindow, WindowMode},
};
use stubby_audio::{Bus, Command};

use crate::audio::Synth;

/// The settings folder's name: unique to this game, in reverse-domain form.
pub const APP_ID: &str = "com.github.buddha314.stubby-music";

/// Write changed settings soon (batched, so repeated clicks write once).
pub fn save(commands: &mut Commands) {
    commands.queue(SaveSettingsDeferred(Duration::from_millis(500)));
}

/// Write changed settings now: before the game exits.
pub fn save_now(commands: &mut Commands) {
    commands.queue(SaveSettingsSync::IfChanged);
}

/// Load the saved settings, and save changes. The binaries add it before the game's
/// plugins, so play starts from the saved values.
pub struct SettingsPersistencePlugin;

impl Plugin for SettingsPersistencePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy::settings::SettingsPlugin::new(APP_ID));
    }
}

/// Apply the settings (from the file, or the defaults) to the running game.
pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GraphicsSettings>()
            .init_resource::<AudioSettings>()
            .add_systems(
                Update,
                (mark_shadow_casters, apply_graphics, apply_volumes).chain(),
            );
    }
}

#[derive(Resource, SettingsGroup, Reflect, Clone, Debug, PartialEq)]
#[reflect(Resource, SettingsGroup, Default)]
#[settings_group(group = "graphics")]
pub struct GraphicsSettings {
    pub display: DisplayMode,
    pub vsync: bool,
    /// The sun's shadows (lights the art made shadowless stay so).
    pub shadows: bool,
    /// Size of every screen and the HUD, 1.0 = as designed.
    pub ui_scale: f32,
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            display: DisplayMode::Windowed,
            vsync: true,
            shadows: true,
            ui_scale: 1.0,
        }
    }
}

impl GraphicsSettings {
    pub const UI_SCALE: (f32, f32) = (0.75, 2.0);
}

#[derive(Reflect, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayMode {
    #[default]
    Windowed,
    Borderless,
    Fullscreen,
}

impl DisplayMode {
    pub fn next(self) -> Self {
        match self {
            Self::Windowed => Self::Borderless,
            Self::Borderless => Self::Fullscreen,
            Self::Fullscreen => Self::Windowed,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Windowed => "Windowed",
            Self::Borderless => "Borderless fullscreen",
            Self::Fullscreen => "Fullscreen",
        }
    }

    fn window_mode(self) -> WindowMode {
        match self {
            Self::Windowed => WindowMode::Windowed,
            Self::Borderless => WindowMode::BorderlessFullscreen(MonitorSelection::Current),
            Self::Fullscreen => WindowMode::Fullscreen(
                MonitorSelection::Current,
                bevy::window::VideoModeSelection::Current,
            ),
        }
    }
}

/// The player's volumes, 0.0 to 1.0, one per engine bus (`stubby_audio::Bus`).
#[derive(Resource, SettingsGroup, Reflect, Clone, Debug, PartialEq)]
#[reflect(Resource, SettingsGroup, Default)]
#[settings_group(group = "audio")]
pub struct AudioSettings {
    pub master: f32,
    /// Machines, nature, every sound in the world.
    pub world: f32,
    /// Interface sounds (a plug snapping in, a collection confirmed).
    pub interface: f32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master: 1.0,
            world: 1.0,
            interface: 1.0,
        }
    }
}

/// A light the art spawned casting shadows: the only kind the shadows setting turns off.
#[derive(Component)]
struct CastsShadows;

fn mark_shadow_casters(
    lights: Query<(Entity, &DirectionalLight), Added<DirectionalLight>>,
    mut commands: Commands,
) {
    for (entity, light) in &lights {
        if light.shadow_maps_enabled {
            commands.entity(entity).insert(CastsShadows);
        }
    }
}

/// Push graphics settings to the window, the UI and the shadow-casting lights. The window
/// follows only a change of setting; lights are checked every frame (they arrive with
/// the world), writing only what differs.
fn apply_graphics(
    settings: Res<GraphicsSettings>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut ui_scale: ResMut<UiScale>,
    mut lights: Query<&mut DirectionalLight, With<CastsShadows>>,
) {
    if settings.is_changed()
        && let Ok(mut window) = windows.single_mut()
    {
        let mode = settings.display.window_mode();
        if window.mode != mode {
            window.mode = mode;
        }
        let present = if settings.vsync {
            PresentMode::AutoVsync
        } else {
            PresentMode::AutoNoVsync
        };
        if window.present_mode != present {
            window.present_mode = present;
        }
    }
    let (low, high) = GraphicsSettings::UI_SCALE;
    let scale = settings.ui_scale.clamp(low, high);
    if ui_scale.0 != scale {
        ui_scale.0 = scale;
    }
    for mut light in &mut lights {
        if light.shadow_maps_enabled != settings.shadows {
            light.shadow_maps_enabled = settings.shadows;
        }
    }
}

/// Send the volumes to the engine when they change, and once the engine is up.
fn apply_volumes(settings: Res<AudioSettings>, synth: Option<Res<Synth>>) {
    let Some(synth) = synth else {
        return;
    };
    if !settings.is_changed() && !synth.is_added() {
        return;
    }
    for (bus, volume) in [
        (Bus::Master, settings.master),
        (Bus::World, settings.world),
        (Bus::Interface, settings.interface),
    ] {
        synth.send(Command::SetBusVolume {
            bus,
            volume: volume.clamp(0.0, 1.0),
        });
    }
}
