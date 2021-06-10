use bevy::prelude::*;

// Boost Bar Attributes
const BOOST_BAR_H: f32 = 15.0;
const BOOST_BAR_W: f32 = 200.0;
const BOOST_BAR_X: f32 = -500.0;

/// Constant factor at which boost drains when the boost is held down.
const BOOST_DRAIN: f32 = 50.0;

/// The amount of boost rewarded each time.
const BOOST_RECHARGE: f32 = 5.0;

/// How often the boost recharges, in seconds.
const BOOST_RECHARGE_INTERVAL: f32 = 1.0;

/// Playing area, unsigned x-coordinate bounds.
const BOUNDS: f32 = 500.0;

/// Allowed deviation when comparing floats.
const EPSILON: f32 = 0.005;

/// Constant factor to calculate paddle speed.
const PADDLE_SPEED: f32 = 500.0;

struct Ball;
struct Boost;
struct BoostBackground;
struct BoostTimer(Timer);
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
        .insert_resource(BoostTimer(Timer::from_seconds(BOOST_RECHARGE_INTERVAL, true)))
        .add_plugins(DefaultPlugins)
        .add_startup_system(startup.system())
        .add_system(paddle_movement.system())
        .add_system(boost_display.system())
        .add_system(boost_recharge.system())
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
            sprite: Sprite::new(Vec2::new(210.0, 25.0)),
            ..Default::default()
        })
        .insert(BoostBackground);

    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(Color::LIME_GREEN.into()),
            transform: Transform::from_xyz(BOOST_BAR_X, 300.0, 0.0),
            sprite: Sprite::new(Vec2::new(BOOST_BAR_W, BOOST_BAR_H)),
            ..Default::default()
        })
        .insert(Boost);
}

/// Handle player control of the paddle.
fn paddle_movement(
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
    mut state: ResMut<State>,
    mut boost_timer: ResMut<BoostTimer>,
    mut query: Query<(&Paddle, &mut Transform)>,
) {
    if let Ok((_, mut transform)) = query.single_mut() {
        let mut direction: f32 = 0.0;
        if input.pressed(KeyCode::A) {
            direction -= 1.0;
        }
        if input.pressed(KeyCode::D) {
            direction += 1.0;
        }

        let mut speed = PADDLE_SPEED;
        if direction.abs() > EPSILON && (input.pressed(KeyCode::LShift) || input.pressed(KeyCode::RShift)) {
            if state.boost >= 0.0 {
                speed *= 2.0;
                state.boost -= time.delta_seconds() * BOOST_DRAIN;
            }
            boost_timer.0.reset();
        }

        transform.translation.x += time.delta_seconds() * direction * speed;
        transform.translation.x = transform.translation.x.clamp(-BOUNDS, BOUNDS);
    }
}

/// Handle the boost HUD element.
fn boost_display(
    state: ResMut<State>,
    mut query: Query<(&Boost, &mut Transform, &mut Sprite)>,
) {
    if let Ok((_, mut transform, mut sprite)) = query.single_mut() {
        sprite.size = Vec2::new(BOOST_BAR_W * (&state.boost / 100.0).clamp(0.0, 1.0), BOOST_BAR_H);
        transform.translation.x = BOOST_BAR_X - (100.0 - &state.boost);
    }
}

/// Handles recharging the boost at regular intervals.
fn boost_recharge(
    time: Res<Time>,
    mut state: ResMut<State>,
    mut timer: ResMut<BoostTimer>,
) {
    if timer.0.tick(time.delta()).finished() {
        state.boost += BOOST_RECHARGE;
        state.boost = state.boost.clamp(0.0, 100.0)
    }
}