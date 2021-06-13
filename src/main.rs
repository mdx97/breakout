use bevy::app::Events;
use bevy::prelude::*;
use bevy::sprite::collide_aabb::{collide, Collision};
use bevy::window::WindowResized;

/// The constant speed the ball moves at.
const BALL_SPEED: f32 = 250.0;

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

/// Thickness of the walls that surround the playing area, in pixels.
const WALL_THICKNESS: f32 = 20.0;

// Default window dimensions.
const WINDOW_HEIGHT: f32 = 720.0;
const WINDOW_WIDTH: f32 = 1280.0;

struct Ball {
    velocity: Vec2,
}

struct Boost;
struct BoostBackground;
struct BoostTimer(Timer);
struct Collider;
struct Paddle;

struct State {
    boost: f32,
}

struct Wall(u32);

impl Default for State {
    fn default() -> Self {
        Self {
            boost: 100.0,
        }
    }
}

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: String::from("Breakout"),
            height: WINDOW_HEIGHT,
            width: WINDOW_WIDTH,
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .insert_resource(State::default())
        .insert_resource(BoostTimer(Timer::from_seconds(BOOST_RECHARGE_INTERVAL, true)))
        .add_plugins(DefaultPlugins)
        .add_startup_system(startup.system())
        .add_system(paddle_movement.system())
        .add_system(ball_movement.system())
        .add_system(ball_collision.system())
        .add_system(boost_display.system())
        .add_system(boost_recharge.system())
        .add_system(window_resize.system())
        .run();
}

fn startup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    let black = materials.add(Color::BLACK.into());
    let white = materials.add(Color::WHITE.into());

    // Create walls
    commands
        .spawn_bundle(SpriteBundle {
            material: black.clone(),
            transform: Transform::from_xyz(-(WINDOW_WIDTH / 2.0) - (WALL_THICKNESS / 2.0), 0.0, 0.0),
            sprite: Sprite::new(Vec2::new(WALL_THICKNESS, WINDOW_HEIGHT)),
            ..Default::default()
        })
        .insert(Collider)
        .insert(Wall(0));

    commands
        .spawn_bundle(SpriteBundle {
            material: black.clone(),
            transform: Transform::from_xyz((WINDOW_WIDTH / 2.0) + (WALL_THICKNESS / 2.0), 0.0, 0.0),
            sprite: Sprite::new(Vec2::new(WALL_THICKNESS, WINDOW_HEIGHT)),
            ..Default::default()
        })
        .insert(Collider)
        .insert(Wall(1));

    commands
        .spawn_bundle(SpriteBundle {
            material: black.clone(),
            transform: Transform::from_xyz(0.0, (WINDOW_HEIGHT / 2.0) + (WALL_THICKNESS / 2.0), 0.0),
            sprite: Sprite::new(Vec2::new(WINDOW_WIDTH, WALL_THICKNESS)),
            ..Default::default()
        })
        .insert(Collider)
        .insert(Wall(2));


    // Create paddle
    commands
        .spawn_bundle(SpriteBundle {
            material: white.clone(),
            transform: Transform::from_xyz(0.0, -300.0, 0.0),
            sprite: Sprite::new(Vec2::new(200.0, 50.0)),
            ..Default::default()
        })
        .insert(Collider)
        .insert(Paddle);


    // Create ball
    commands
        .spawn_bundle(SpriteBundle {
            material: white.clone(),
            transform: Transform::from_xyz(-150.0, 0.0, 0.0),
            sprite: Sprite::new(Vec2::new(30.0, 30.0)),
            ..Default::default()
        })
        .insert(Ball { velocity: Vec2::new(BALL_SPEED, -BALL_SPEED)});

    // Create Boost HUD
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

/// System for moving the paddle in response to player input.
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

/// System for updating the ball's position.
fn ball_movement(time: Res<Time>, mut query: Query<(&Ball, &Sprite, &mut Transform)>) {
    if let Ok((ball, _, mut transform)) = query.single_mut() {
        transform.translation.x += time.delta_seconds() * ball.velocity.x;
        transform.translation.y += time.delta_seconds() * ball.velocity.y;
    }
}

/// System for handling collisions between the ball and other geometry in the scene.
fn ball_collision(
    mut query: QuerySet<(
        Query<&mut Ball>,
        Query<(&Ball, &Sprite, &Transform)>,
        Query<(&Sprite, &Collider, &Transform)>,
    )>
) {
    let mut velocity = query.q0_mut().single_mut().unwrap().velocity.clone();

    if let Ok((_, sprite, transform)) = query.q1().single() {
        for (other_sprite, _, other_transform) in query.q2().iter() {
            if let Some(collision) = collide(other_transform.translation.clone(), other_sprite.size.clone(), transform.translation.clone(), sprite.size.clone()) {
                match collision {
                    Collision::Left | Collision::Right => { velocity.x = -velocity.x },
                    Collision::Top | Collision::Bottom => { velocity.y = -velocity.y },
                };
            }
        }
    }

    query.q0_mut().single_mut().unwrap().velocity = velocity;
}

/// System for updating the boost HUD.
fn boost_display(
    state: ResMut<State>,
    mut query: Query<(&Boost, &mut Transform, &mut Sprite)>,
) {
    if let Ok((_, mut transform, mut sprite)) = query.single_mut() {
        sprite.size = Vec2::new(BOOST_BAR_W * (&state.boost / 100.0).clamp(0.0, 1.0), BOOST_BAR_H);
        transform.translation.x = BOOST_BAR_X - (100.0 - &state.boost);
    }
}

/// System for recharging the player's boost.
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

/// System for handling the position / size of elements when the screen resizes.
fn window_resize(event: Res<Events<WindowResized>>, mut query: Query<(&Wall, &mut Sprite, &mut Transform)>) {
    // TODO: Need to handle the location of the HUD.
    // TODO: Need to handle paddle limits.
    for e in event.get_reader().iter(&event) {
        for (wall, mut sprite, mut transform) in query.iter_mut() {
            match wall.0 {
                0 => {
                    *sprite = Sprite::new(Vec2::new(WALL_THICKNESS, e.height));
                    transform.translation.x = -(e.width / 2.0) - (WALL_THICKNESS / 2.0);
                },
                1 => {
                    *sprite = Sprite::new(Vec2::new(WALL_THICKNESS, e.height));
                    transform.translation.x = (e.width / 2.0) + (WALL_THICKNESS / 2.0);
                },
                2 => {
                    *sprite = Sprite::new(Vec2::new(e.width, WALL_THICKNESS));
                    transform.translation.y = (e.height / 2.0) + (WALL_THICKNESS / 2.0);
                },
                _ => { },
            }
        }
    }
}