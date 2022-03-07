#![feature(let_chains)]

mod collider;
use std::f32::consts::PI;
use std::time::Duration;

use bevy::math::const_vec2;
use bevy::{prelude::*, render::camera::Camera};
use collider::Collider;

#[derive(Component, Debug, Default)]
struct Velocity(Vec2);

#[derive(Component, Debug)]
struct Obstacle;
#[derive(Component, Debug)]
struct Bullet(Timer);

#[derive(Component, Debug)]
enum Player {
    Start,
    Game,
}

const BULLET_SIZE: Vec2 = const_vec2!([10.0, 10.0]);
// TODO: add a better system to replace this delay
const BULLET_ACTIVATION_TIME: Duration = Duration::from_millis(70);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(handle_movement)
        .add_system(move_transform)
        .add_system(handle_start_shot)
        .add_system(bullet_collide)
        .add_system(bullet_hits_player)
        .add_system(advance_bullet_time)
        .run();
}

fn spawn_bullet(cmds: &mut Commands, vel: Velocity, transform: Transform) {
    cmds.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: Color::BLACK,
            custom_size: Some(BULLET_SIZE),
            ..Default::default()
        },
        transform,
        ..Default::default()
    })
    .insert(Collider::rectangle(BULLET_SIZE))
    .insert(Bullet(Timer::new(BULLET_ACTIVATION_TIME, false)))
    .insert(vel);
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

    //obstacle
    let obstacle_size = Vec2::new(20.0, 100.0);
    cmds.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.0, 1.0, 0.0),
            custom_size: Some(obstacle_size),
            ..Default::default()
        },
        transform: Transform::from_translation(Vec3::new(40.0, 0.0, 1.0)),
        ..Default::default()
    })
    .insert(Collider::rectangle(obstacle_size))
    .insert(Obstacle);

    // Camera
    cmds.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn bullet_collide(
    mut cmds: Commands,
    mut bullets: Query<(&mut Velocity, &Transform, &mut Bullet, &Collider), Without<Obstacle>>,
    obstacles: Query<(&Collider, &Transform), (With<Obstacle>, Without<Bullet>)>,
) {
    bullets.for_each_mut(|(mut vel, bul_trans, mut bullet, bul_col)| {
        // wait a certain time before detecting collisions
        if !bullet.0.finished() {
            return;
        }

        let collision = obstacles
            .iter()
            .filter_map(|obstacle| collider::are_colliding((bul_col, &bul_trans), obstacle))
            .next();

        // TODO: maybe add a spawn delay
        if let Some((norm, _)) = collision {
            let vel_magnitude = vel.0.length();
            let side = norm.perp();
            let angle = vel.0.angle_between(side);

            vel.0 = vector_at_angle(vel_magnitude, PI + angle / 2.0);
            bullet.0.reset(); // reset timer
            let vel = Velocity(mirror_vector(vel.0, side));

            spawn_bullet(&mut cmds, vel, *bul_trans);
        }
    })
}

fn mirror_vector(vec: Vec2, norm: Vec2) -> Vec2 {
    // https://stackoverflow.com/questions/56274674/how-to-mirror-a-vector-along-a-surface
    vec - 2.0 * vec.dot(norm) * norm
}

fn vector_at_angle(magnitude: f32, angle: f32) -> Vec2 {
    magnitude * Vec2::new(angle.cos(), angle.sin())
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

    // TODO: create a delay before the bullet can kill
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
        x -= 1.0;
    }
    if keyboard.pressed(KeyCode::D) {
        x += 1.0;
    }
    if keyboard.pressed(KeyCode::S) {
        y -= 1.0;
    }
    if keyboard.pressed(KeyCode::W) {
        y += 1.0;
    }

    tran.translation += WALK_SPEED * time.delta_seconds() * Vec3::new(x, y, 0.0);
}

fn handle_start_shot(
    mut cmds: Commands,
    mouse_input: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    camera: Query<&Transform, With<Camera>>,
    mut query: Query<(&Transform, &mut Player)>,
) {
    if let (player_trans, mut player) = query.single_mut()
    //&& let Player::Start = *player
    && mouse_input.just_pressed(MouseButton::Left) {
        // the real game starts now
        *player = Player::Game;

        let window = windows.get_primary().expect("no primary window");
        let curs_pos = window.cursor_position().expect("no cursor");
        let curs_pos = screen_to_world(curs_pos, window, camera);

        let dir = curs_pos - player_trans.translation.truncate();
        let proj_vel = PROJECTILE_SPEED * dir.normalize();

        spawn_bullet(&mut cmds, Velocity(proj_vel), *player_trans);
    }
}

fn screen_to_world(screen: Vec2, window: &Window, camera: Query<&Transform, With<Camera>>) -> Vec2 {
    let w_size = Vec2::new(window.width() as f32, window.height() as f32);

    let pos = screen - (w_size / 2.0);

    let transform_matrix = camera.single().compute_matrix();

    let world = transform_matrix * pos.extend(0.0).extend(1.0);

    world.truncate().truncate()
}
