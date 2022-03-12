use bevy::prelude::*;
use itertools::Itertools;

#[derive(Component, Debug, Default)]
pub struct Collider(pub Vec<Vec2>);

impl Collider {
    pub fn rectangle(size: Vec2) -> Self {
        let half = size * 0.5;
        let points = vec![
            -half,
            -half + size.x * Vec2::X,
            half,
            half - size.y * Vec2::Y,
        ];

        Self(points)
    }

    pub fn _triangle(p1: Vec2, p2: Vec2, p3: Vec2) -> Self {
        Self(vec![p1, p2, p3])
    }
}

pub fn are_colliding(
    (col_a, trans_a): (&Collider, &Transform),
    (col_b, trans_b): (&Collider, &Transform),
) -> Option<(Vec2, f32)> {
    let points_a = points_to_world(&col_a.0, trans_a);
    let points_b = points_to_world(&col_b.0, trans_b);

    let edges_a = edges_between(&points_a);
    let edges_b = edges_between(&points_b);

    let mut overlaps = get_overlaps(edges_a, &points_a, &points_b)
        .chain(get_overlaps(edges_b, &points_a, &points_b));

    let first = overlaps.next().expect("oneagon?");

    overlaps.fold(
        Some(first),
        |min_overlap: Option<(Vec2, f32)>, (axis, overlap)| {
            if overlap <= 0.0 {
                None
            } else {
                min_overlap.map(|(back_axis, min_ov)| {
                    if min_ov > overlap {
                        (axis, overlap)
                    } else {
                        (back_axis, min_ov)
                    }
                })
            }
        },
    )
}

fn points_to_world(ps: &[Vec2], trans: &Transform) -> Vec<Vec2> {
    ps.iter()
        .map(|p| *trans * p.extend(0.0))
        // get them back to vec2
        .map(|p: Vec3| p.truncate())
        .collect()
}

fn edges_between(ps: &[Vec2]) -> impl Iterator<Item = Vec2> + '_ {
    let initial = *ps.last().expect("Nonagon infinity ?");
    ps.iter().scan(initial, |prev: &mut Vec2, p| {
        let ret = *p - *prev;
        *prev = *p;
        Some(ret)
    })
}

fn get_overlaps<'a, I: Iterator<Item = Vec2> + 'a>(
    edges: I,
    points_a: &'a [Vec2],
    points_b: &'a [Vec2],
) -> impl Iterator<Item = (Vec2, f32)> + 'a {
    edges.map(move |e| {
        let e = e.perp().normalize();
        let (a_min, a_max) = projection_bounds(e, points_a);
        let (b_min, b_max) = projection_bounds(e, points_b);

        //a_min <= b_max && a_max >= b_min

        let overlap = if a_min < b_max {
            a_max - b_min
        } else {
            b_max - a_min
        };

        (e, overlap)
    })
}

fn projection_bounds(norm: Vec2, points: &[Vec2]) -> (f32, f32) {
    points
        .iter()
        .map(|p| norm.dot(*p))
        .minmax()
        .into_option()
        .unwrap_or((0.0, 0.0))
}

#[cfg(test)]
mod tests {
    use super::Collider;
    use bevy::prelude::*;

    #[test]
    fn not_colliding() {
        let a = Collider::rectangle(10.0 * Vec2::ONE);
        let b = Collider::rectangle(10.0 * Vec2::ONE);
        let ta = Transform::default();
        let tb = Transform::from_translation(Vec3::new(20.0, 20.0, 0.0));

        assert!(
            super::are_colliding((&a, &ta), (&b, &tb)).is_none(),
            "Colliders are not colliding"
        )
    }

    #[test]
    fn colliding() {
        let a = Collider::rectangle(10.0 * Vec2::ONE);
        let b = Collider::rectangle(10.0 * Vec2::ONE);
        let ta = Transform::default();
        let tb = Transform::from_translation(Vec3::new(5.0, 5.0, 0.0));

        assert!(
            super::are_colliding((&a, &ta), (&b, &tb)).is_some(),
            "Colliders are colliding"
        )
    }

    #[test]
    fn correct_collision() {
        let a = Collider::rectangle(10.0 * Vec2::ONE);
        let b = Collider::rectangle(10.0 * Vec2::ONE);
        let ta = Transform::default();
        let mut tb = Transform::from_translation(Vec3::new(5.0, 5.0, 0.0));

        let (dir, mag) = super::are_colliding((&a, &ta), (&b, &tb)).unwrap();

        tb.translation += mag * dir.extend(0.0);

        assert!(
            dbg!(super::are_colliding((&a, &ta), (&b, &tb))).is_none(),
            "Corrected the collision"
        )
    }

    /*#[test]
    fn get_edges() {
        let a = Collider::rectangle(10.0 * Vec2::ONE);
        let edges = HashSet::from([
            Vec2::new(0.0, -10.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(0.0, 10.0),
            Vec2::new(-10.0, 0.0),
        ]);

        let result: HashSet<_> = super::edges_between(&a.0).collect();

        assert_eq!(edges, result, "Doesn't calculate the right edges")
    }*/
}
