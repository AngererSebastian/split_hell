use bevy::prelude::*;

/// mirrors an vector along an side
pub fn mirror_vector(vec: Vec2, side: Vec2) -> Vec2 {
    // https://stackoverflow.com/questions/56274674/how-to-mirror-a-vector-along-a-surface
    vec - 2.0 * vec.dot(side) * side
}

/// creates an vector with the given magnitude and
/// the given angle on the x-axis
pub fn vector_at_angle(magnitude: f32, angle: f32) -> Vec2 {
    magnitude * Vec2::new(angle.cos(), angle.sin())
}

/// converts Screen coordinates to world coordinates
pub fn screen_to_world(
    screen: Vec2,
    window: &Window,
    camera: Query<&Transform, With<Camera>>,
) -> Vec2 {
    let w_size = Vec2::new(window.width() as f32, window.height() as f32);

    let pos = screen - (w_size / 2.0);

    let transform_matrix = camera.single().compute_matrix();

    let world = transform_matrix * pos.extend(0.0).extend(1.0);

    world.truncate().truncate()
}
