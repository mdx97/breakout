use bevy::app::Events;
use bevy::prelude::*;
use bevy::sprite::collide_aabb::{collide, Collision};
use bevy::window::WindowResized;
use rand::Rng;

/// The constant speed the ball moves at.
const BALL_SPEED: f32 = 250.0;

// Boost Bar dimensions.
const BOOST_BAR_H: f32 = 15.0;
const BOOST_BAR_W: f32 = 200.0;

/// Constant factor at which boost drains when the boost is held down.
const BOOST_DRAIN: f32 = 50.0;

/// The amount of pixels the boost HUD is inset from the edge of the screen.
const BOOST_HUD_INSET: f32 = 30.0;

/// The amount of pixels that the boost HUD border is on the edges.
const BOOST_PADDING: f32 = 5.0;

/// The amount of boost rewarded each time.
const BOOST_RECHARGE: f32 = 5.0;

/// How often the boost recharges, in seconds.
const BOOST_RECHARGE_INTERVAL: f32 = 1.0;

/// Allowed deviation when comparing floats.
const EPSILON: f32 = 0.005;

/// Constant factor to calculate paddle speed.
const PADDLE_SPEED: f32 = 500.0;

/// Thickness of the walls that surround the playing area, in pixels.
const WALL_THICKNESS: f32 = 20.0;

// Default window dimensions.
const WINDOW_HEIGHT: f32 = 720.0;
const WINDOW_WIDTH: f32 = 1280.0;

// Brick display constants.
const BRICK_MARGIN: f32 = 15.0;
const BRICK_ROWS: u32 = 3;
const BRICK_SIZE: f32 = 50.0;
const BRICK_COUNT: u32 = (WINDOW_WIDTH / (BRICK_SIZE + BRICK_MARGIN)) as u32;
const BRICK_ROW_SIZE: f32 = (BRICK_SIZE * BRICK_COUNT as f32) + (BRICK_MARGIN * (BRICK_COUNT - 1) as f32);
const BRICK_X_START: f32 = ((WINDOW_WIDTH - BRICK_ROW_SIZE) / 2.0) - (WINDOW_WIDTH / 2.0) + (BRICK_SIZE / 2.0);
const BRICK_Y_START: f32 = 250.0;

// Brick health constants.
const BRICK_HEALTH_MAX: usize = 3;
const BRICK_HEALTH_COLORS: [(u8, u8, u8); 3] = [(255, 255, 255), (255, 122, 0), (255, 0, 0)];

// Brick spawning cosntants.
const BRICK_PROBABILITY: f32 = 0.6;

struct Ball {
    velocity: Vec2,
}

struct Boost;
struct BoostBackground;
struct BoostTimer(Timer);
struct Brick(u32);
struct Collider;
struct Paddle;

struct State {
    boost: f32,
    window_height: f32,
    window_width: f32,
}

struct Wall(u32);

impl Default for State {
    fn default() -> Self {
        Self {
            boost: 100.0,
            window_height: WINDOW_HEIGHT,
            window_width: WINDOW_WIDTH,
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
        .insert_resource(Vec::<Handle<ColorMaterial>>::new())
        .add_plugins(DefaultPlugins)
        .add_startup_system(startup.system())
        .add_system(paddle_movement.system())
        .add_system(ball_movement.system())
        .add_system(brick_collision.system())
        .add_system(general_collision.system())
        .add_system(boost_display.system())
        .add_system(boost_recharge.system())
        .add_system(window_resize.system())
        .run();
}

fn startup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut colors: ResMut<Vec<Handle<ColorMaterial>>>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    let black = materials.add(Color::BLACK.into());
    let white = materials.add(Color::WHITE.into());

    // TODO: Create functions for calculating positions both here and in window_resize()?

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
    let boost_transform = Transform::from_xyz(
        -(WINDOW_WIDTH / 2.0) + (BOOST_BAR_W / 2.0) + BOOST_HUD_INSET,
        (WINDOW_HEIGHT / 2.0) - (BOOST_BAR_H / 2.0) - BOOST_HUD_INSET,
        0.0
    );

    commands
        .spawn_bundle(SpriteBundle {
            material: white.clone(),
            transform: boost_transform.clone(),
            sprite: Sprite::new(Vec2::new(BOOST_BAR_W + (BOOST_PADDING * 2.0), BOOST_BAR_H + (BOOST_PADDING * 2.0))),
            ..Default::default()
        })
        .insert(BoostBackground);

    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(Color::LIME_GREEN.into()),
            transform: boost_transform,
            sprite: Sprite::new(Vec2::new(BOOST_BAR_W, BOOST_BAR_H)),
            ..Default::default()
        })
        .insert(Boost);

    // Create bricks
    let color = BRICK_HEALTH_COLORS[0];
    let color = materials.add(Color::rgb_u8(color.0, color.1, color.2).into());
    let mut rng = rand::thread_rng();

    for col in 0..BRICK_COUNT {
        for row in 0..BRICK_ROWS {
            if rng.gen_range(0.0..1.0) < BRICK_PROBABILITY {
                commands
                    .spawn_bundle(SpriteBundle {
                        material: color.clone(),
                        transform: Transform::from_xyz(
                            BRICK_X_START + (col as f32 * (BRICK_SIZE + BRICK_MARGIN)),
                            BRICK_Y_START - (row as f32 * (BRICK_SIZE + BRICK_MARGIN)),
                            0.0,
                        ),
                        sprite: Sprite::new(Vec2::new(BRICK_SIZE, BRICK_SIZE)),
                        ..Default::default()
                    })
                    .insert(Brick(BRICK_HEALTH_MAX as u32));
            }
        }
    }

    // Add colors to resource
    for idx in 0..BRICK_HEALTH_COLORS.len() {
        let color = BRICK_HEALTH_COLORS[idx];
        colors.push(materials.add(Color::rgb_u8(color.0, color.1, color.2).into()));
    }
}

/// System for moving the paddle in response to player input.
fn paddle_movement(
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
    mut state: ResMut<State>,
    mut boost_timer: ResMut<BoostTimer>,
    mut query: Query<(&Paddle, &Sprite, &mut Transform)>,
) {
    if let Ok((_, sprite, mut transform)) = query.single_mut() {
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

        let x_bound = (&state.window_width / 2.0) - (sprite.size[0] / 2.0);
        transform.translation.x += time.delta_seconds() * direction * speed;
        transform.translation.x = transform.translation.x.clamp(-x_bound.clone(), x_bound);
    }
}

/// System for updating the ball's position.
fn ball_movement(time: Res<Time>, mut query: Query<(&Ball, &Sprite, &mut Transform)>) {
    if let Ok((ball, _, mut transform)) = query.single_mut() {
        transform.translation.x += time.delta_seconds() * ball.velocity.x;
        transform.translation.y += time.delta_seconds() * ball.velocity.y;
    }
}

/// Detects if there is a collision between the two objects, handles updating the velocity, and returns whether or not a collision occured.
fn detect_collision(sprite: &Sprite, transform: &mut Transform, other_sprite: &Sprite, other_transform: &Transform, velocity: &mut Vec2) -> bool {
    if let Some(collision) = collide(transform.translation.clone(), sprite.size.clone(), other_transform.translation.clone(), other_sprite.size.clone()) {
        match collision {
            Collision::Left | Collision::Right => { velocity.x = -velocity.x },
            Collision::Top | Collision::Bottom => { velocity.y = -velocity.y },
        };

        // TODO: Collisions are working a bit better, but still wonky sometimes.
        // Maybe research a better way to do this?
        match collision {
            Collision::Left => {
                let left = other_transform.translation.x - (other_sprite.size.x / 2.0);
                transform.translation.x = left - (sprite.size.x / 2.0) - 0.1;
            },
            Collision::Right => {
                let right = other_transform.translation.x + (other_sprite.size.x / 2.0);
                transform.translation.x = right + (sprite.size.x / 2.0) + 0.1;
            },
            Collision::Top => {
                let top = other_transform.translation.y + (other_sprite.size.x / 2.0);
                transform.translation.y = top + (sprite.size.y / 2.0) + 0.1;
            },
            Collision::Bottom => {
                let bottom = other_transform.translation.y - (other_sprite.size.x / 2.0);
                transform.translation.y = bottom - (sprite.size.y / 2.0) - 0.1;
            },
        };
        return true;
    }
    false
}

/// System for handling collisions between the ball and bricks.
fn brick_collision(
    mut commands: Commands,
    mut query: QuerySet<(
        Query<&mut Ball>,
        Query<(&Ball, &Sprite, &mut Transform)>,
        Query<(Entity, &mut Brick, &Sprite, &Transform, &mut Handle<ColorMaterial>)>,
    )>,
    colors: Res<Vec<Handle<ColorMaterial>>>,
) {
    // TODO: There has GOT to be a better way to do this...
    let mut velocity = query.q0_mut().single_mut().unwrap().velocity.clone();
    let ball_sprite = query.q1_mut().single_mut().unwrap().1.clone();
    let mut ball_transform = query.q1_mut().single_mut().unwrap().2.clone();
    
    for (entity, mut brick, brick_sprite, brick_transform, mut brick_material) in query.q2_mut().iter_mut() {
        if detect_collision(&ball_sprite, &mut ball_transform, brick_sprite, &brick_transform, &mut velocity) {
            brick.0 -= 1;
            if brick.0 == 0 {
                commands.entity(entity).despawn();
            } else {
                *brick_material = colors[BRICK_HEALTH_MAX - brick.0 as usize].clone();
            }
        }
    }

    query.q0_mut().single_mut().unwrap().velocity = velocity;

}

/// System for handling collisions between the ball and other geometry in the scene.
fn general_collision(
    mut query: QuerySet<(
        Query<&mut Ball>,
        Query<(&Ball, &Sprite, &mut Transform)>,
        Query<(&Sprite, &Collider, &Transform)>,
    )>
) {
    let mut velocity = query.q0_mut().single_mut().unwrap().velocity.clone();
    let ball_sprite = query.q1_mut().single_mut().unwrap().1.clone();
    let mut ball_transform = query.q1_mut().single_mut().unwrap().2.clone();

    for (other_sprite, _, other_transform) in query.q2().iter() {
        detect_collision(&ball_sprite, &mut ball_transform, other_sprite, other_transform, &mut velocity);
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
        transform.translation.x = (-(&state.window_width / 2.0) + (BOOST_BAR_W / 2.0) + BOOST_HUD_INSET) - (100.0 - &state.boost);
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
fn window_resize(
    event: Res<Events<WindowResized>>,
    mut query: QuerySet<(
        Query<(&Wall, &mut Sprite, &mut Transform)>,
        Query<(&Boost, &mut Transform)>,
        Query<(&BoostBackground, &mut Transform)>,
    )>,
    mut state: ResMut<State>,
) {
    // TODO: Need to handle the location of the HUD.
    // TODO: Need to handle paddle limits.
    for e in event.get_reader().iter(&event) {
        state.window_height = e.height;
        state.window_width = e.width;

        for (wall, mut sprite, mut transform) in query.q0_mut().iter_mut() {
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

        let boost_transform = Transform::from_xyz(
            -(e.width / 2.0) + (BOOST_BAR_W / 2.0) + BOOST_HUD_INSET,
            (e.height / 2.0) - (BOOST_BAR_H / 2.0) - BOOST_HUD_INSET,
            0.0
        );

        if let Ok((_, mut transform)) = query.q1_mut().single_mut() {
            *transform = boost_transform.clone();
        }

        if let Ok((_, mut transform)) = query.q2_mut().single_mut() {
            *transform = boost_transform;
        }
    }
}
