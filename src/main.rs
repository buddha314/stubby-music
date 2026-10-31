use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Stubby Music".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::srgb(0.12, 0.69, 0.94),
            custom_size: Some(Vec2::new(120.0, 120.0)),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ..default()
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_can_be_created() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, || {});
    }
}
