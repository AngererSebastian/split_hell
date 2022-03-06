#![feature(let_chains)]

mod collider;
use bevy::{prelude::*, render::camera::Camera};
use collider::Collider;

#[derive(Default)]
struct Velocity(Vec2);

struct Obstacle;
struct Bullet;

enum Player {
    Start,
    Game,
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_system(handle_movement.system())
        .add_system(move_transform.system())
        .add_system(handle_start_shot.system())
        .add_system(bullet_collide.system())
        .run();
}

fn setup(mut cmds: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    // Player
    let player_size = 20.0 * Vec2::ONE;
    cmds.spawn_bundle(SpriteBundle {
        material: materials.add(Color::rgb(1.0, 0.0, 1.0).into()),
        sprite: Sprite::new(player_size),
        ..Default::default()
    })
    .insert(Velocity::default())
    .insert(Collider::rectangle(Vec2::ZERO, player_size))
    .insert(Player::Start);

    //obstacle
    let obstacle_size = Vec2::new(20.0, 100.0);
    cmds.spawn_bundle(SpriteBundle {
        material: materials.add(Color::rgb(0.0, 1.0, 0.0).into()),
        sprite: Sprite::new(obstacle_size),
        transform: Transform::from_xyz(40.0, 0.0, 1.0),
        ..Default::default()
    })
    .insert(Collider::rectangle(Vec2::ZERO, obstacle_size))
    .insert(Obstacle);

    // Camera
    cmds.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn bullet_collide(
    mut materials: ResMut<Assets<ColorMaterial>>,
    bullets: Query<(&Velocity, &Transform, &Collider), With<Bullet>>,
    obstacles: Query<(&Collider, &Transform, &Handle<ColorMaterial>), With<Obstacle>>,
) {
    obstacles.for_each(|(obs_col, obs_trans, handle)| {
        // is any bullet colliding
        let colliding = bullets.iter().any(|(_, bullet_trans, bul_collider)| {
            collider::are_colliding((bul_collider, bullet_trans), (obs_col, obs_trans))
        });

        let material = materials.get_mut(handle).unwrap();

        // change the color if it's colliding
        material.color = if colliding {
            Color::CRIMSON
        } else {
            Color::rgb(1.0, 0.0, 1.0)
        }
    })
}

fn move_transform(time: Res<Time>, query: Query<(&Velocity, &mut Transform)>) {
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
    if let Ok(mut tran) = query.single_mut() {
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
}

fn handle_start_shot(
    mut cmds: Commands,
    mut colors: ResMut<Assets<ColorMaterial>>,
    mouse_input: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    camera: Query<&Transform, With<Camera>>,
    mut query: Query<(&Transform, &mut Player)>,
) {
    if let Ok((player_trans, mut player)) = query.single_mut()
    && let Player::Start = *player
    && mouse_input.just_pressed(MouseButton::Left) {
        // the real game starts now
        *player = Player::Game;

        let window = windows.get_primary().expect("no primary window");
        let curs_pos = window.cursor_position().expect("no cursor");
        let curs_pos = screen_to_world(curs_pos, window, camera);

        let dir = curs_pos - player_trans.translation.into();
        let proj_vel = PROJECTILE_SPEED * dir.normalize();

        let bullet_size = 10.0 * Vec2::ONE;
        cmds.spawn_bundle(SpriteBundle {
            sprite: Sprite::new(10.0 * Vec2::ONE),
            material: colors.add(Color::BLACK.into()),
            transform: *player_trans,
            ..Default::default()
        })
        .insert(Collider::rectangle(Vec2::ZERO, bullet_size))
        .insert(Bullet)
        .insert(Velocity(proj_vel));
    }
}

fn screen_to_world(screen: Vec2, window: &Window, camera: Query<&Transform, With<Camera>>) -> Vec2 {
    let w_size = Vec2::new(window.width() as f32, window.height() as f32);

    let pos = screen - (w_size / 2.0);

    let transform_matrix = camera.single().unwrap().compute_matrix();

    let world = transform_matrix * pos.extend(0.0).extend(1.0);

    world.truncate().truncate()
}
