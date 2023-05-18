use bevy::{prelude::*, sprite::collide_aabb::collide, sprite::MaterialMesh2dBundle};

// Defines the amount of time that should elapse between each physics step.
const TIME_STEP: f32 = 1.0 / 60.0;

const GAP_BETWEEN_PLAYER_AND_FLOOR: f32 = 60.0;
const GAP_BETWEEN_ENEMIES_AND_CEILING: f32 = 30.0;
const GAP_BETWEEN_ENEMIES_AND_SIDES: f32 = 100.0;
const GAP_BETWEEN_PLAYER_AND_ENEMIES: f32 = 450.0;
const HORIONZTAL_GAP_BETWEEN_ENEMIES: f32 = 50.0;
const VERTICAL_GAP_BETWEEN_ENEMIES: f32 = 25.0;

const PLAYER_SIZE: Vec3 = Vec3::new(100.0, 25.0, 0.0);
const PLAYER_SPEED: f32 = 400.0;
// How close a player can get to a wall
const PLAYER_PADDING: f32 = 10.0;

const ENEMY_SIZE: Vec3 = Vec3::new(50.0, 25.0, 0.0);
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

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_startup_system(setup)
        .add_event::<CollisionEvent>()
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
                move_player
                    .before(check_for_collisions)
                    .after(apply_velocity),
            )
                .in_schedule(CoreSchedule::FixedUpdate),
        )
        .insert_resource(FixedTime::new_from_secs(TIME_STEP))
        .add_system(bevy::window::close_on_esc)
        .run();
}

#[derive(Component)]
struct Player;

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

fn setup(mut commands: Commands) {
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
    ));

    // Walls
    commands.spawn(WallBundle::new(WallLocation::Left));
    commands.spawn(WallBundle::new(WallLocation::Right));
    commands.spawn(WallBundle::new(WallLocation::Bottom));
    commands.spawn(WallBundle::new(WallLocation::Top));

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
        let player_position = query.single();

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

fn move_player(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let mut player_transform = query.single_mut();
    let mut direction = 0.0;

    if keyboard_input.pressed(KeyCode::A) {
        direction -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::D) {
        direction += 1.0;
    }

    let new_player_position = player_transform.translation.x + direction * PLAYER_SPEED * TIME_STEP;

    // Update the player position,
    // making sure it doesn't cause the player to leave the arena
    let left_bound = LEFT_WALL + WALL_THICKNESS / 2.0 + PLAYER_SIZE.x / 2.0 + PLAYER_PADDING;
    let right_bound = RIGHT_WALL - WALL_THICKNESS / 2.0 - PLAYER_SIZE.x / 2.0 - PLAYER_PADDING;

    player_transform.translation.x = new_player_position.clamp(left_bound, right_bound);
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

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * TIME_STEP;
        transform.translation.y += velocity.y * TIME_STEP;
    }
}

fn check_for_collisions(
    mut commands: Commands,
    projectile_query: Query<(Entity, &Transform), With<Projectile>>,
    collider_query: Query<(Entity, &Transform, Option<&Enemy>), With<Collider>>,
    mut collision_events: EventWriter<CollisionEvent>,
) {
    // check collision with walls
    for (collider_entity, transform, maybe_enemy) in &collider_query {
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

                if maybe_enemy.is_some() {
                    commands.entity(collider_entity).despawn()
                }
            }
        }
    }
}
