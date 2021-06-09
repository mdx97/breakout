use bevy::prelude::*;

/// Constant factor at which boost drains when the boost is held down.
const BOOST_DRAIN: f32 = 25.0;

/// Playing area, unsigned x-coordinate bounds.
const BOUNDS: f32 = 500.0;

/// Constant factor to calculate paddle speed.
const PADDLE_SPEED: f32 = 500.0;

struct Ball;
struct Boost;
struct BoostBackground;
struct Paddle;

struct State {
    boost: f32,
}

impl Default for State {
    fn default() -> Self {
        Self {
            boost: 100.0,
        }
    }
}

fn main() {
    App::build()
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .insert_resource(State::default())
        .add_plugins(DefaultPlugins)
        .add_startup_system(startup.system())
        .add_system(paddle_movement.system())
        .add_system(boost_display.system())
        .run();
}

fn startup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    let white = materials.add(Color::WHITE.into());

    commands
        .spawn_bundle(SpriteBundle {
            material: white.clone(),
            transform: Transform::from_xyz(0.0, -300.0, 0.0),
            sprite: Sprite::new(Vec2::new(200.0, 50.0)),
            ..Default::default()
        })
        .insert(Paddle);

    commands
        .spawn_bundle(SpriteBundle {
            material: white.clone(),
            transform: Transform::from_xyz(0.0, -250.0, 0.0),
            sprite: Sprite::new(Vec2::new(30.0, 30.0)),
            ..Default::default()
        })
        .insert(Ball);

    commands
        .spawn_bundle(SpriteBundle {
            material: white.clone(),
            transform: Transform::from_xyz(-500.0, 300.0, 0.0),
            sprite: Sprite::new(Vec2::new(200.0, 25.0)),
            ..Default::default()
        })
        .insert(BoostBackground);

    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(Color::LIME_GREEN.into()),
            transform: Transform::from_xyz(-500.0, 300.0, 0.0),
            sprite: Sprite::new(Vec2::new(190.0, 15.0)),
            ..Default::default()
        })
        .insert(Boost);
}

fn paddle_movement(
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
    mut state: ResMut<State>,
    mut query: Query<(&Paddle, &mut Transform)>,
) {
    if let Ok((_, mut transform)) = query.single_mut() {
        let mut direction = 0.0;
        if input.pressed(KeyCode::A) {
            direction -= 1.0;
        }
        if input.pressed(KeyCode::D) {
            direction += 1.0;
        }

        let mut speed = PADDLE_SPEED;
        if (input.pressed(KeyCode::LShift) || input.pressed(KeyCode::RShift)) && state.boost >= 0.0 {
            speed *= 2.0;
            state.boost -= time.delta_seconds() * BOOST_DRAIN;
        }

        transform.translation.x += time.delta_seconds() * direction * speed;
        transform.translation.x = transform.translation.x.min(BOUNDS).max(-BOUNDS);
    }
}

fn boost_display(
    state: ResMut<State>,
    mut query: Query<(&Boost, &mut Transform)>,
) {
    if let Ok((_, mut transform)) = query.single_mut() {
        transform.scale.x = (&state.boost / 100.0).min(0.0).max(1.0);
    }
}
