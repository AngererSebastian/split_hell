use bevy::prelude::*;

/// mirrors an vector along an side
pub fn reflect_vector(vec: Vec2, norm: Vec2) -> Vec2 {
    // https://stackoverflow.com/questions/56274674/how-to-mirror-a-vector-along-a-surface
    vec - 2.0 * vec.dot(norm) * norm
}

pub fn mirror_vector(vec: Vec2, norm: Vec2) -> Vec2 {
    -reflect_vector(vec, norm)
}

/// creates an vector with the given magnitude and
/// the given angle on the x-axis
pub fn _vector_at_angle(magnitude: f32, angle: f32) -> Vec2 {
    magnitude * Vec2::new(angle.cos(), angle.sin())
}

/// converts Screen coordinates to world coordinates
pub fn screen_to_world(screen: Vec2, window: &Window, camera: &Transform) -> Vec2 {
    let w_size = Vec2::new(window.width() as f32, window.height() as f32);

    let pos = screen - (w_size / 2.0);

    let transform_matrix = camera.compute_matrix();

    let world = transform_matrix * pos.extend(0.0).extend(1.0);

    world.truncate().truncate()
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use std::f32::consts::PI;

    fn vec_equal(v1: Vec2, v2: Vec2) -> bool {
        let diff = (v1 - v2).abs();

        diff.x < 0.001 && diff.y < 0.001
    }

    #[test]
    fn degrees_90() {
        let expected = Vec2::Y;

        let result = super::_vector_at_angle(1.0, PI / 2.0);

        assert!(
            vec_equal(expected, result),
            "really rotated by ninety degrees"
        )
    }

    #[test]
    fn reflect() {
        let input = Vec2::new(1.0, -1.0);
        let norm = Vec2::new(0.0, 1.0);
        let expected = Vec2::new(1.0, 1.0);

        let result = super::reflect_vector(input, norm);

        assert_eq!(expected, result, "reflect on y-axis")
    }

    #[test]
    fn mirror_1() {
        let input = Vec2::new(-1.0, 1.0);
        let norm = Vec2::new(0.0, 1.0);
        let expected = Vec2::new(1.0, 1.0);

        let result = super::mirror_vector(input, norm);

        assert_eq!(expected, result, "reflect on y-axis")
    }

    #[test]
    fn mirror_2() {
        let input = Vec2::new(-1.0, -1.0);
        let norm = Vec2::new(0.0, -1.0);
        let expected = Vec2::new(1.0, -1.0);

        let result = super::mirror_vector(input, norm);

        assert_eq!(expected, result, "reflect on y-axis")
    }

    #[test]
    fn mirror_3() {
        let input = Vec2::new(1.0, 1.0);
        let norm = Vec2::new(1.0, 0.0);
        let expected = Vec2::new(1.0, -1.0);

        let result = super::mirror_vector(input, norm);

        assert_eq!(expected, result, "reflect on y-axis")
    }

    #[test]
    fn mirror_4() {
        let input = Vec2::new(1.0, -1.0);
        let norm = Vec2::new(1.0, 0.0);
        let expected = Vec2::new(1.0, 1.0);

        let result = super::mirror_vector(input, norm);

        assert_eq!(expected, result, "reflect on y-axis")
    }

    #[test]
    fn mirror_5() {
        let input = Vec2::new(-1.0, 1.0);
        let norm = Vec2::new(-1.0, 0.0);
        let expected = Vec2::new(-1.0, -1.0);

        let result = super::mirror_vector(input, norm);

        assert_eq!(expected, result, "reflect on y-axis")
    }
}
