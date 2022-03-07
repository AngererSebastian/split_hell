#![feature(let_chains)]

mod collider;
mod util;

use std::time::Duration;

use bevy::math::const_vec2;
use bevy::{prelude::*, render::camera::Camera};
use collider::Collider;

#[derive(Component, Debug, Default, Clone, Copy)]
struct Velocity(Vec2);

#[derive(Component, Debug)]
struct Obstacle;
#[derive(Component, Debug)]
struct Bullet(Timer);

struct BulletSpawn {
    velocity: Velocity,
    transform: Transform,
}

#[derive(Default)]
struct BulletCount(u8);

#[derive(Component, Debug)]
enum Player {
    Start,
    Game,
}

const BULLET_SIZE: Vec2 = const_vec2!([10.0, 10.0]);
const MAX_BULLETS: u8 = 100;
// TODO: add a better system to replace this delay
const BULLET_ACTIVATION_TIME: Duration = Duration::from_millis(70);
const OBSTACLE_SIZE_A: f32 = 500.0;
const OBSTACLE_SIZE_B: f32 = 20.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<BulletCount>()
        .add_event::<BulletSpawn>()
        .add_startup_system(setup)
        .add_system(handle_movement)
        .add_system(handle_player_looking)
        .add_system(move_transform)
        .add_system(handle_start_shot)
        .add_system(bullet_collide)
        .add_system(bullet_hits_player)
        .add_system(advance_bullet_time)
        .add_system(spawn_bullets)
        .run();
}

fn spawn_bullets(
    mut cmds: Commands,
    mut bullet_count: ResMut<BulletCount>,
    mut bullets: EventReader<BulletSpawn>,
) {
    for b in bullets.iter() {
        // spawn no more
        if bullet_count.0 > MAX_BULLETS {
            break;
        }

        cmds.spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::BLACK,
                custom_size: Some(BULLET_SIZE),
                ..Default::default()
            },
            transform: b.transform,
            ..Default::default()
        })
        .insert(Collider::rectangle(BULLET_SIZE))
        .insert(Bullet(Timer::new(BULLET_ACTIVATION_TIME, false)))
        .insert(b.velocity);

        bullet_count.0 += 1;
    }
}

fn advance_bullet_time(time: Res<Time>, mut bullets: Query<&mut Bullet>) {
    let elapsed = time.delta();

    bullets.for_each_mut(|mut b| {
        b.0.tick(elapsed);
    })
}

fn setup(mut cmds: Commands) {
    // Player
    let player_size = 20.0 * Vec2::ONE;
    cmds.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(1.0, 0.0, 1.0),
            custom_size: Some(player_size),
            ..Default::default()
        },
        ..Default::default()
    })
    .insert(Velocity::default())
    .insert(Collider::rectangle(player_size))
    .insert(Player::Start);

    // TODO: use hexagons as boundary
    //obstacle left
    let obstacle_size_vert = Vec2::new(OBSTACLE_SIZE_B, OBSTACLE_SIZE_A);
    let obstacle_size_hor = Vec2::new(OBSTACLE_SIZE_A, OBSTACLE_SIZE_B);
    let half_a = OBSTACLE_SIZE_A / 2.0;

    cmds.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.0, 1.0, 0.0),
            custom_size: Some(obstacle_size_vert),
            ..Default::default()
        },
        transform: Transform::from_translation(Vec3::new(half_a, 0.0, 1.0)),
        ..Default::default()
    })
    .insert(Collider::rectangle(obstacle_size_vert))
    .insert(Obstacle);

    // obstacle right
    cmds.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.0, 1.0, 0.0),
            custom_size: Some(obstacle_size_vert),
            ..Default::default()
        },
        transform: Transform::from_translation(Vec3::new(-half_a, 0.0, 1.0)),
        ..Default::default()
    })
    .insert(Collider::rectangle(obstacle_size_vert))
    .insert(Obstacle);

    // obstacle top
    cmds.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.0, 1.0, 0.0),
            custom_size: Some(obstacle_size_hor),
            ..Default::default()
        },
        transform: Transform::from_translation(Vec3::new(0.0, -half_a, 1.0)),
        ..Default::default()
    })
    .insert(Collider::rectangle(obstacle_size_hor))
    .insert(Obstacle);

    // obstacle bottom
    cmds.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.0, 1.0, 0.0),
            custom_size: Some(obstacle_size_hor),
            ..Default::default()
        },
        transform: Transform::from_translation(Vec3::new(0.0, half_a, 1.0)),
        ..Default::default()
    })
    .insert(Collider::rectangle(obstacle_size_hor))
    .insert(Obstacle);

    // Camera
    cmds.spawn_bundle(OrthographicCameraBundle::new_2d());
}

type IsObstacleQuery = (With<Obstacle>, Without<Bullet>);

fn bullet_collide(
    mut bullets: Query<(&mut Velocity, &Transform, &mut Bullet, &Collider), Without<Obstacle>>,
    obstacles: Query<(&Collider, &Transform), IsObstacleQuery>,
    mut spawn_bullet: EventWriter<BulletSpawn>,
) {
    bullets.for_each_mut(|(mut vel, bul_trans, mut bullet, bul_col)| {
        // wait a certain time before detecting collisions
        if !bullet.0.finished() {
            return;
        }

        let collision = obstacles
            .iter()
            .find_map(|obstacle| collider::are_colliding((bul_col, bul_trans), obstacle));

        if let Some((norm, _)) = collision {
            let vel_magnitude = vel.0.length();
            let side = norm.perp();
            let angle = vel.0.angle_between(side);

            let dir = bevy::math::Mat2::from_angle(angle / 2.0) * side;
            vel.0 = vel_magnitude * dir.normalize();
            bullet.0.reset(); // reset timer
            let velocity = Velocity(util::mirror_vector(vel.0, norm));

            spawn_bullet.send(BulletSpawn {
                velocity,
                transform: *bul_trans,
            })
        }
    })
}

fn bullet_hits_player(
    player: Query<(&Collider, &Transform), With<Player>>,
    bullets: Query<(&Collider, &Transform, &Bullet)>,
) {
    // there is only one player (for now)
    let player = player.single();

    let hit = bullets.iter().any(|(col, trans, bullet)| {
        bullet.0.finished() && collider::are_colliding((col, trans), player).is_some()
    });

    if hit {
        println!("hit!! call an ambulance");
    }
}

fn move_transform(time: Res<Time>, mut query: Query<(&Velocity, &mut Transform)>) {
    query.for_each_mut(|(v, mut t)| {
        t.translation.x += v.0.x * time.delta_seconds();
        t.translation.y += v.0.y * time.delta_seconds();
    })
}

const WALK_SPEED: f32 = 600.0;
const PROJECTILE_SPEED: f32 = 200.0;

fn handle_movement(
    time: Res<Time>,
    keyboard: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let mut tran = query.single_mut();
    let mut x = 0.0;
    let mut y = 0.0;

    if keyboard.pressed(KeyCode::A) {
        y -= 1.0;
    }
    if keyboard.pressed(KeyCode::D) {
        y += 1.0;
    }
    if keyboard.pressed(KeyCode::S) {
        x -= 1.0;
    }
    if keyboard.pressed(KeyCode::W) {
        x += 1.0;
    }

    let movement_dir = tran.rotation * Vec3::new(x, y, 0.0);
    tran.translation += WALK_SPEED * time.delta_seconds() * movement_dir;
}

fn handle_player_looking(
    windows: Res<Windows>,
    camera: Query<&Transform, (With<Camera>, Without<Player>)>,
    mut player: Query<&mut Transform, With<Player>>,
) {
    let mut player = player.single_mut();
    let window = windows.get_primary().expect("no primary window");

    if let Some(curs_pos) = window.cursor_position() {
        let camera = camera.single();
        let curs_pos = util::screen_to_world(curs_pos, window, camera);
        let dir = curs_pos - player.translation.truncate();
        let angle = -dir.angle_between(Vec2::X);

        player.rotation = Quat::from_axis_angle(Vec3::Z, angle);
    }
}

fn handle_start_shot(
    mouse_input: Res<Input<MouseButton>>,
    mut player: Query<(&Transform, &mut Player)>,
    mut spawn_bullet: EventWriter<BulletSpawn>,
) {
    if let (player_trans, mut player) = player.single_mut()
    //&& let Player::Start = *player
    && mouse_input.just_pressed(MouseButton::Left) {
        // the real game starts now
        *player = Player::Game;

        let dir =  player_trans.rotation * Vec3::X;
        let proj_vel = PROJECTILE_SPEED * dir.truncate().normalize();

        spawn_bullet.send(BulletSpawn {
            velocity: Velocity(proj_vel),
            transform: *player_trans,
        })
    }
}
