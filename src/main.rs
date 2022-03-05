#![feature(let_chains)]

use bevy::{prelude::*, render::camera::Camera};

#[derive(Default)]
struct Velocity(Vec2);

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
        .run();
}

fn setup(mut cmds: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    // Player
    cmds.spawn_bundle(SpriteBundle {
        material: materials.add(Color::rgb(1.0, 0.0, 1.0).into()),
        sprite: Sprite::new(20.0 * Vec2::ONE),
        ..Default::default()
    })
    .insert(Velocity::default())
    .insert(Player::Start);

    // Camera
    cmds.spawn_bundle(OrthographicCameraBundle::new_2d());
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

        cmds.spawn_bundle(SpriteBundle {
            sprite: Sprite::new(10.0 * Vec2::ONE),
            material: colors.add(Color::rgb(1.0, 0.0, 0.0).into()),
            transform: *player_trans,
            ..Default::default()
        })
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
