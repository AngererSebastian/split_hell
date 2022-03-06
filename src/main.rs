#![feature(let_chains)]

mod collider;
use bevy::{prelude::*, render::camera::Camera};
use collider::Collider;

#[derive(Component, Debug, Default)]
struct Velocity(Vec2);

#[derive(Component, Debug)]
struct Obstacle;
#[derive(Component, Debug)]
struct Bullet;

#[derive(Component, Debug)]
enum Player {
    Start,
    Game,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_system(handle_movement.system())
        .add_system(move_transform.system())
        .add_system(handle_start_shot.system())
        .add_system(bullet_collide.system())
        .add_system(bullet_hits_player.system())
        .run();
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
    mut bullets: Query<
        (&mut Velocity, &mut Transform, &Collider, &Sprite),
        (With<Bullet>, Without<Obstacle>),
    >,
    obstacles: Query<(&Collider, &Transform), (With<Obstacle>, Without<Bullet>)>,
) {
    bullets.for_each_mut(|(vel, bul_trans, bul_col, material)| {
        let colliding: Vec<_> = obstacles
            .iter()
            .filter_map(|obstacle| collider::are_colliding((bul_col, &bul_trans), obstacle))
            .collect();

        if !colliding.is_empty() {
            println!("Colliding! {:?}", colliding);
        }
    })
}

fn bullet_hits_player(
    player: Query<(&Collider, &Transform), With<Player>>,
    bullets: Query<(&Collider, &Transform), With<Bullet>>,
) {
    // there is only one player (for now)
    let player = player.single();

    let hit = bullets
        .iter()
        .any(|bullet| collider::are_colliding(bullet, player).is_some());

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

        let bullet_size = 10.0 * Vec2::ONE;
        cmds.spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::BLACK,
                custom_size: Some(bullet_size),
                ..Default::default()
            },
            transform: *player_trans,
            ..Default::default()
        })
        .insert(Collider::rectangle(bullet_size))
        .insert(Bullet)
        .insert(Velocity(proj_vel));
    }
}

fn screen_to_world(screen: Vec2, window: &Window, camera: Query<&Transform, With<Camera>>) -> Vec2 {
    let w_size = Vec2::new(window.width() as f32, window.height() as f32);

    let pos = screen - (w_size / 2.0);

    let transform_matrix = camera.single().compute_matrix();

    let world = transform_matrix * pos.extend(0.0).extend(1.0);

    world.truncate().truncate()
}
