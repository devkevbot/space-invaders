use bevy::{
    prelude::*, sprite::collide_aabb::collide, sprite::MaterialMesh2dBundle, window::PresentMode,
};

// Defines the amount of time that should elapse between each physics step.
const TIME_STEP: f32 = 1.0 / 60.0;

const GAP_BETWEEN_PLAYER_AND_FLOOR: f32 = 60.0;
const GAP_BETWEEN_ENEMIES_AND_CEILING: f32 = 30.0;
const GAP_BETWEEN_ENEMIES_AND_SIDES: f32 = 100.0;
const GAP_BETWEEN_PLAYER_AND_ENEMIES: f32 = 450.0;
const HORIONZTAL_GAP_BETWEEN_ENEMIES: f32 = 50.0;
const VERTICAL_GAP_BETWEEN_ENEMIES: f32 = 25.0;

const INITIAL_PLAYER_LIVES: usize = 3;
const PLAYER_SIZE: Vec3 = Vec3::new(100.0, 25.0, 0.0);
const PLAYER_SPEED: f32 = 400.0;
// How close a player can get to a wall
const PLAYER_PADDING: f32 = 10.0;

const ENEMY_SIZE: Vec3 = Vec3::new(50.0, 25.0, 0.0);
const ENEMY_SPEED: f32 = 50.0;
// How close an enemy can get to a wall
const ENEMY_PADDING: f32 = 10.0;
const INITIAL_ENEMY_DIRECTION: Vec2 = Vec2::new(-1.0, 0.0);
const N_ENEMY_ROWS: usize = 4;
const N_ENEMY_COLS: usize = 6;

const PROJECTILE_SIZE: Vec3 = Vec3::new(25.0, 25.0, 0.0);
const PROJECTILE_SPEED: f32 = 400.0;
const INITIAL_PLAYER_PROJECTILE_DIRECTION: Vec2 = Vec2::new(0.0, 1.0);
const INITIAL_ENEMY_PROJECTILE_DIRECTION: Vec2 = Vec2::new(0.0, -1.0);

const WALL_THICKNESS: f32 = 10.0;
// x coordinates
const LEFT_WALL: f32 = -450.0;
const RIGHT_WALL: f32 = 450.0;
// y coordinates
const BOTTOM_WALL: f32 = -400.0;
const TOP_WALL: f32 = 400.0;

const BACKGROUND_COLOR: Color = Color::BLACK;
const WALL_COLOR: Color = Color::GREEN;
const PLAYER_COLOR: Color = Color::GREEN;
const ENEMY_COLOR: Color = Color::RED;
const PLAYER_PROJECTILE_COLOR: Color = Color::GREEN;
const ENEMY_PROJECTILE_COLOR: Color = Color::RED;
const TEXT_COLOR: Color = Color::GREEN;
const SCORE_COLOR: Color = Color::GREEN;

const SCOREBOARD_FONT_SIZE: f32 = 40.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(30.0);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Totally not Space Invaders".into(),
                resolution: (1920.0, 1080.0).into(),
                present_mode: PresentMode::AutoVsync,
                // Tells wasm to resize the window according to the available canvas
                fit_canvas_to_parent: true,
                position: WindowPosition::At(IVec2 { x: 0, y: 0 }),
                // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_startup_system(setup)
        .add_event::<CollisionEvent>()
        .insert_resource(Scoreboard { score: 0 })
        .add_system(
            shoot_player_projectile
                .before(check_for_collisions)
                .before(apply_velocity),
        )
        .insert_resource(EnemyShootTimer(Timer::from_seconds(
            2.0,
            TimerMode::Repeating,
        )))
        .add_system(shoot_enemy_projectile)
        .add_systems(
            (
                check_for_collisions,
                apply_velocity.before(check_for_collisions),
                move_enemies
                    .before(check_for_collisions)
                    .after(apply_velocity),
                move_player
                    .before(check_for_collisions)
                    .after(apply_velocity),
            )
                .in_schedule(CoreSchedule::FixedUpdate),
        )
        .insert_resource(FixedTime::new_from_secs(TIME_STEP))
        .add_system(update_scoreboard)
        .add_system(bevy::window::close_on_esc)
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Lives(usize);

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct Collider;

#[derive(Component)]
struct Projectile;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Default)]
struct CollisionEvent;

#[derive(Resource)]
struct EnemyShootTimer(Timer);

#[derive(Resource)]
struct Scoreboard {
    score: usize,
}

enum WallLocation {
    Left,
    Right,
    Bottom,
    Top,
}

impl WallLocation {
    fn position(&self) -> Vec2 {
        match self {
            WallLocation::Left => Vec2::new(LEFT_WALL, 0.),
            WallLocation::Right => Vec2::new(RIGHT_WALL, 0.),
            WallLocation::Bottom => Vec2::new(0., BOTTOM_WALL),
            WallLocation::Top => Vec2::new(0., TOP_WALL),
        }
    }

    fn size(&self) -> Vec2 {
        let arena_height = TOP_WALL - BOTTOM_WALL;
        let arena_width = RIGHT_WALL - LEFT_WALL;
        // Make sure we haven't messed up our constants
        assert!(arena_height > 0.0);
        assert!(arena_width > 0.0);

        match self {
            WallLocation::Left | WallLocation::Right => {
                Vec2::new(WALL_THICKNESS, arena_height + WALL_THICKNESS)
            }
            WallLocation::Bottom | WallLocation::Top => {
                Vec2::new(arena_width + WALL_THICKNESS, WALL_THICKNESS)
            }
        }
    }
}

// This bundle is a collection of the components that define a "wall" in our game
#[derive(Bundle)]
struct WallBundle {
    // You can nest bundles inside of other bundles like this
    // Allowing you to compose their functionality
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

impl WallBundle {
    // This "builder method" allows us to reuse logic across our wall entities,
    // making our code easier to read and less prone to bugs when we change the logic
    fn new(location: WallLocation) -> WallBundle {
        WallBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    // We need to convert our Vec2 into a Vec3, by giving it a z-coordinate
                    // This is used to determine the order of our sprites
                    translation: location.position().extend(0.0),
                    // The z-scale of 2D objects must always be 1.0,
                    // or their ordering will be affected in surprising ways.
                    // See https://github.com/bevyengine/bevy/issues/4149
                    scale: location.size().extend(1.0),
                    ..default()
                },
                sprite: Sprite {
                    color: WALL_COLOR,
                    ..default()
                },
                ..default()
            },
            collider: Collider,
        }
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let player_y = BOTTOM_WALL + GAP_BETWEEN_PLAYER_AND_FLOOR;

    // Spawn the player
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, player_y, 0.0),
                scale: PLAYER_SIZE,
                ..default()
            },
            sprite: Sprite {
                color: PLAYER_COLOR,
                ..default()
            },
            ..default()
        },
        Player,
        Collider,
        Lives(INITIAL_PLAYER_LIVES),
    ));

    // Walls
    commands.spawn(WallBundle::new(WallLocation::Left));
    commands.spawn(WallBundle::new(WallLocation::Right));
    commands.spawn(WallBundle::new(WallLocation::Bottom));
    commands.spawn(WallBundle::new(WallLocation::Top));

    // Scoreboard
    commands.spawn(
        TextBundle::from_sections([
            TextSection::new(
                "Score: ",
                TextStyle {
                    font_size: SCOREBOARD_FONT_SIZE,
                    color: TEXT_COLOR,
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                },
            ),
            TextSection::from_style(TextStyle {
                font_size: SCOREBOARD_FONT_SIZE,
                color: SCORE_COLOR,
                font: asset_server.load("fonts/FiraMono-Medium.ttf"),
            }),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                top: SCOREBOARD_TEXT_PADDING,
                left: SCOREBOARD_TEXT_PADDING,
                ..default()
            },
            ..default()
        }),
    );

    // Enemies
    let total_width_of_enemies = (RIGHT_WALL - LEFT_WALL) - 2. * GAP_BETWEEN_ENEMIES_AND_SIDES;
    let bottom_edge_of_enemies = player_y + GAP_BETWEEN_PLAYER_AND_ENEMIES;
    let total_height_of_enemies =
        TOP_WALL - bottom_edge_of_enemies - GAP_BETWEEN_ENEMIES_AND_CEILING;

    assert!(total_width_of_enemies > 0.0);
    assert!(total_height_of_enemies > 0.0);

    let n_vertical_gaps = N_ENEMY_COLS - 1;

    let center_of_enemies = (LEFT_WALL + RIGHT_WALL) / 2.0;
    let left_edge_of_enemies = center_of_enemies
        // Space taken up by the enemies
        - (N_ENEMY_COLS as f32 / 2.0 * ENEMY_SIZE.x)
        // Space taken up by the gaps between enemies
        - n_vertical_gaps as f32 / 2.0 * HORIONZTAL_GAP_BETWEEN_ENEMIES;

    let offset_x = left_edge_of_enemies + ENEMY_SIZE.x / 2.0;
    let offset_y = bottom_edge_of_enemies + ENEMY_SIZE.y / 2.0;

    for row in 0..N_ENEMY_ROWS {
        for column in 0..N_ENEMY_COLS {
            let enemy_position = Vec2::new(
                offset_x + column as f32 * (ENEMY_SIZE.x + HORIONZTAL_GAP_BETWEEN_ENEMIES),
                offset_y + row as f32 * (ENEMY_SIZE.y + VERTICAL_GAP_BETWEEN_ENEMIES),
            );

            // enemy
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: ENEMY_COLOR,
                        ..default()
                    },
                    transform: Transform {
                        translation: enemy_position.extend(0.0),
                        scale: Vec3::new(ENEMY_SIZE.x, ENEMY_SIZE.y, 1.0),
                        ..default()
                    },
                    ..default()
                },
                Enemy,
                Collider,
                Velocity(INITIAL_ENEMY_DIRECTION.normalize() * ENEMY_SPEED),
            ));
        }
    }
}

fn shoot_player_projectile(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<&Transform, With<Player>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        if let Ok(player_position) = query.get_single() {
            // Spawn projectile
            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: meshes.add(shape::Circle::default().into()).into(),
                    material: materials.add(ColorMaterial::from(PLAYER_PROJECTILE_COLOR)),
                    transform: Transform::from_translation(
                        Vec2::new(
                            player_position.translation.x,
                            player_position.translation.y + PLAYER_SIZE.y,
                        )
                        .extend(0.),
                    )
                    .with_scale(PROJECTILE_SIZE),
                    ..default()
                },
                Projectile,
                Velocity(INITIAL_PLAYER_PROJECTILE_DIRECTION.normalize() * PROJECTILE_SPEED),
            ));
        }
    }
}

fn move_player(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    if let Ok(mut player_transform) = query.get_single_mut() {
        let mut direction = 0.0;

        if keyboard_input.pressed(KeyCode::A) {
            direction -= 1.0;
        }

        if keyboard_input.pressed(KeyCode::D) {
            direction += 1.0;
        }

        let new_player_position =
            player_transform.translation.x + direction * PLAYER_SPEED * TIME_STEP;

        // Update the player position,
        // making sure it doesn't cause the player to leave the arena
        let left_bound = LEFT_WALL + WALL_THICKNESS / 2.0 + PLAYER_SIZE.x / 2.0 + PLAYER_PADDING;
        let right_bound = RIGHT_WALL - WALL_THICKNESS / 2.0 - PLAYER_SIZE.x / 2.0 - PLAYER_PADDING;

        player_transform.translation.x = new_player_position.clamp(left_bound, right_bound);
    }
}

fn shoot_enemy_projectile(
    time: Res<Time>,
    mut timer: ResMut<EnemyShootTimer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<(Entity, &Transform), With<Enemy>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        for (entity_a, transform_a) in query.iter() {
            let mut can_shoot = true;

            for (entity_b, transform_b) in query.iter() {
                if entity_a == entity_b {
                    continue;
                }

                // We don't need to check if the entities are not below one another
                if transform_b.translation.x != transform_a.translation.x {
                    continue;
                }

                // An Enemy can't shoot if another Enemy is below it
                if transform_b.translation.y < transform_a.translation.y {
                    can_shoot = false;
                    break;
                }
            }

            if !can_shoot {
                continue;
            }

            // Spawn projectile
            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: meshes.add(shape::Circle::default().into()).into(),
                    material: materials.add(ColorMaterial::from(ENEMY_PROJECTILE_COLOR)),
                    transform: Transform::from_translation(
                        Vec2::new(
                            transform_a.translation.x,
                            transform_a.translation.y - ENEMY_SIZE.y,
                        )
                        .extend(0.),
                    )
                    .with_scale(PROJECTILE_SIZE),
                    ..default()
                },
                Projectile,
                Velocity(INITIAL_ENEMY_PROJECTILE_DIRECTION.normalize() * PROJECTILE_SPEED),
            ));
        }
    }
}

fn move_enemies(
    transform_query: Query<&Transform, With<Enemy>>,
    mut velocity_query: Query<&mut Velocity, With<Enemy>>,
) {
    // If any Enemy hits the left or right bounds, we need to every Enemy in the opposite direction

    let mut reverse_direction = false;

    let left_bound = LEFT_WALL + WALL_THICKNESS / 2.0 + ENEMY_SIZE.x / 2.0 + ENEMY_PADDING;
    let right_bound = RIGHT_WALL - WALL_THICKNESS / 2.0 - ENEMY_SIZE.x / 2.0 - ENEMY_PADDING;

    for transform in transform_query.iter() {
        if transform.translation.x <= left_bound || transform.translation.x >= right_bound {
            reverse_direction = true
        }
    }

    if !reverse_direction {
        return;
    }

    for mut velocity in velocity_query.iter_mut() {
        velocity.x *= -1.0
    }
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * TIME_STEP;
        transform.translation.y += velocity.y * TIME_STEP;
    }
}

fn update_scoreboard(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text>) {
    let mut text = query.single_mut();
    text.sections[1].value = scoreboard.score.to_string();
}

fn check_for_collisions(
    mut commands: Commands,
    projectile_query: Query<(Entity, &Transform), With<Projectile>>,
    mut lives_query: Query<(Entity, &mut Lives), With<Player>>,
    collider_query: Query<(Entity, &Transform, Option<&Enemy>, Option<&Player>), With<Collider>>,
    mut collision_events: EventWriter<CollisionEvent>,
    mut scoreboard: ResMut<Scoreboard>,
) {
    for (collider_entity, transform, maybe_enemy, maybe_player) in &collider_query {
        for (projectile_entity, projectile_transform) in &projectile_query {
            let collision = collide(
                projectile_transform.translation,
                projectile_transform.scale.truncate(),
                transform.translation,
                transform.scale.truncate(),
            );

            if collision.is_some() {
                // Sends a collision event so that other systems can react to the collision
                collision_events.send_default();

                commands.entity(projectile_entity).despawn();

                if maybe_player.is_some() {
                    let (player_entity, mut lives) = lives_query.single_mut();
                    // Game over
                    if lives.0 == 1 {
                        commands.entity(player_entity).despawn()
                    } else {
                        lives.0 -= 1
                    }
                }

                if maybe_enemy.is_some() {
                    scoreboard.score += 1;
                    commands.entity(collider_entity).despawn()
                }
            }
        }
    }
}
