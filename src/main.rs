#![feature(let_chains)]

mod collider;
mod util;

use std::f32::consts::PI;
use std::time::Duration;

use bevy::core::Stopwatch;
use bevy::math::const_vec2;
use bevy::prelude::*;
use collider::Collider;

#[derive(Component, Debug, Default, Clone, Copy)]
struct Velocity(Vec2);

#[derive(Component, Debug)]
struct MainCamera;
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
    Game(Stopwatch),
    GameOver(Duration),
}

const BULLET_SIZE: Vec2 = const_vec2!([10.0, 10.0]);
const MAX_BULLETS: u8 = 254;
// TODO: add a better system to replace this delay
const BULLET_ACTIVATION_TIME: Duration = Duration::from_millis(120);
const OBSTACLE_WIDTH: f32 = 60.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<BulletCount>()
        .add_event::<BulletSpawn>()
        .add_startup_system(setup_border)
        .add_startup_system(setup)
        .add_startup_system(setup_ui)
        .add_system(display_timer)
        .add_system(handle_movement)
        .add_system(move_transform)
        .add_system(handle_start_shot)
        .add_system(bullet_collide)
        .add_system(border_collide)
        .add_system(bullet_hits_player)
        .add_system(advance_bullet_time)
        .add_system(spawn_bullets)
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
        transform: Transform::from_rotation(Quat::from_rotation_z(PI / 2.0)),
        ..Default::default()
    })
    .insert(Velocity::default())
    .insert(Collider::rectangle(player_size))
    .insert(Player::Start);

    // Cameras
    cmds.spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(MainCamera);
}

fn setup_ui(mut cmds: Commands, assets: Res<AssetServer>) {
    cmds.spawn_bundle(UiCameraBundle::default());

    println!("hello");
    cmds.spawn_bundle(TextBundle {
        style: Style {
            align_self: AlignSelf::FlexEnd,
            position_type: PositionType::Absolute,
            position: Rect {
                bottom: Val::Px(5.0),
                right: Val::Px(15.0),
                ..Default::default()
            },
            ..Default::default()
        },
        text: Text::with_section(
            "00:00.000",
            TextStyle {
                font: assets.load("FiraMono-Regular.otf"),
                font_size: 100.0,
                color: Color::WHITE,
            },
            Default::default(),
        ),
        ..Default::default()
    });
}

fn setup_border(mut cmds: Commands) {
    let radius = 250.0f32; // center to a vertex
    let side = (3.0 * radius * radius).sqrt(); // side of the hexagon
    dbg!(side);
    let segment = Vec2::new(OBSTACLE_WIDTH, side);
    let to_side = (radius * radius + side * side / 4.0).sqrt();

    let side_rot = Quat::from_rotation_z(60.0f32.to_radians());
    let mut distance = (to_side + OBSTACLE_WIDTH / 2.0) * Vec3::X;

    for i in 0..6 {
        cmds.spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::GREEN,
                custom_size: Some(segment),
                ..Default::default()
            },
            transform: Transform {
                translation: distance,
                rotation: Quat::from_rotation_z(i as f32 * 60.0f32.to_radians()),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Obstacle)
        .insert(Collider::rectangle(segment));

        distance = side_rot * distance;
    }
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

fn display_timer(mut text: Query<&mut Text>, player: Query<&Player>) {
    let mut text = text.single_mut();

    match *player.single() {
        Player::Game(ref stopwatch) => {
            text.sections[0].value = util::display_duration(stopwatch.elapsed());
        }
        Player::GameOver(elapsed) => {
            text.sections[0].value = util::display_duration(elapsed);
        }
        _ => (),
    }
}

fn advance_bullet_time(
    time: Res<Time>,
    mut player: Query<&mut Player>,
    mut bullets: Query<&mut Bullet>,
) {
    let elapsed = time.delta();

    let player = player.single_mut().into_inner();
    if let Player::Game(ref mut s) = player {
        s.tick(elapsed);
    }

    bullets.for_each_mut(|mut b| {
        b.0.tick(elapsed);
    })
}

type IsObstacleQuery = (With<Obstacle>, Without<Bullet>, Without<Player>);

fn bullet_collide(
    mut bullets: Query<(&mut Velocity, &mut Transform, &mut Bullet, &Collider), Without<Obstacle>>,
    obstacles: Query<(&Collider, &Transform), IsObstacleQuery>,
    mut spawn_bullet: EventWriter<BulletSpawn>,
) {
    bullets.for_each_mut(|(mut vel, mut bul_trans, mut bullet, bul_col)| {
        // wait a certain time before detecting collisions
        // if !bullet.0.finished() {
        //     return;
        // }

        let collision = obstacles
            .iter()
            .find_map(|obstacle| collider::are_colliding(obstacle, (bul_col, &bul_trans)));

        if let Some((norm, magnitude)) = collision {
            bul_trans.translation += magnitude * norm.extend(0.0);
            //let vel_magnitude = vel.0.length();
            //let side = norm.perp();
            //let angle = vel.0.angle_between(side);

            //let dir = bevy::math::Mat2::from_angle(angle / 2.0) * side;
            //vel.0 = vel_magnitude * dir.normalize();
            vel.0 = util::reflect_vector(vel.0, norm);
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
    mut player: Query<((&Collider, &Transform), &mut Player)>,
    bullets: Query<(&Collider, &Transform, &Bullet)>,
) {
    // there is only one player (for now)
    let (p_collision, player) = player.single_mut();
    let player = player.into_inner();

    let hit = bullets.iter().any(|(col, trans, bullet)| {
        bullet.0.finished() && collider::are_colliding((col, trans), p_collision).is_some()
    });

    if let Player::Game(ref s) = player && hit {
        *player = Player::GameOver(s.elapsed());
        println!("hit!! call an ambulance");
    }
}

fn border_collide(
    mut player: Query<(&Collider, &mut Transform), With<Player>>,
    obstacles: Query<(&Collider, &Transform), IsObstacleQuery>,
) {
    let (p_col, mut p_trans) = player.single_mut();

    obstacles.for_each(|obs| {
        if let Some((dir, mag)) = collider::are_colliding(obs, (p_col, &p_trans)) {
            p_trans.translation += mag * dir.extend(0.0);
        }
    })
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

    let movement_dir = Vec3::new(x, y, 0.);
    tran.translation += WALK_SPEED * time.delta_seconds() * movement_dir;
}

fn handle_start_shot(
    mouse_input: Res<Input<MouseButton>>,
    mut player: Query<(&Transform, &mut Player)>,
    windows: Res<Windows>,
    camera: Query<&Transform, (With<MainCamera>, Without<Player>)>,
    mut spawn_bullet: EventWriter<BulletSpawn>,
) {
    if let (player_trans, mut player) = player.single_mut()
    //&& let Player::Start = *player
    && mouse_input.just_pressed(MouseButton::Left) {
        // the real game starts now
        *player = Player::Game(Stopwatch::new());

        let camera = camera.single();
        let window = windows.get_primary().expect("no primary window");
        let curs_pos = window.cursor_position().expect("clicking with no cursor?");
        let curs_pos = util::screen_to_world(curs_pos, window, camera);
        let dir = curs_pos - player_trans.translation.truncate();
        let proj_vel = PROJECTILE_SPEED * dir.normalize();

        spawn_bullet.send(BulletSpawn {
            velocity: Velocity(proj_vel),
            transform: *player_trans,
        })
    }
}
