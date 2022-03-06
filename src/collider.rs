use bevy::prelude::*;
use itertools::Itertools;

#[derive(Component, Debug, Default)]
pub struct Collider(pub Vec<Vec2>);

impl Collider {
    pub fn rectangle(origin: Vec2, size: Vec2) -> Self {
        let points = vec![
            origin,
            origin + Vec2::new(size.x, 0.0),
            origin + size,
            origin + Vec2::new(0.0, size.y),
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
) -> bool {
    let points_a = points_to_world(&col_a.0, trans_a);
    let points_b = points_to_world(&col_b.0, trans_b);

    let edges_a: Vec<_> = edges_between(&points_a).collect();
    let edges_b: Vec<_> = edges_between(&points_b).collect();

    are_projs_overlapping(edges_a.into_iter(), &points_a, &points_b)
        && are_projs_overlapping(edges_b.into_iter(), &points_a, &points_b)
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

fn are_projs_overlapping<I: Iterator<Item = Vec2>>(
    mut edges: I,
    points_a: &[Vec2],
    points_b: &[Vec2],
) -> bool {
    edges.all(|e| {
        let (a_min, a_max) = projection_bounds(e, &points_a);
        let (b_min, b_max) = projection_bounds(e, &points_b);

        a_min <= b_max && a_max >= b_min
    })
}

fn projection_bounds(edge: Vec2, ps: &[Vec2]) -> (f32, f32) {
    ps.iter()
        .map(|p| {
            // perpendicular of edge and then the dot product with p
            edge.perp_dot(*p)
        })
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
        let a = Collider::rectangle(Vec2::ZERO, 10.0 * Vec2::ONE);
        let b = Collider::rectangle(20.0 * Vec2::ONE, 10.0 * Vec2::ONE);
        let t = Transform::default();

        assert!(
            !super::are_colliding((&a, &t), (&b, &t)),
            "Colliders are not colliding"
        )
    }

    #[test]
    fn colliding() {
        let a = Collider::rectangle(Vec2::ZERO, 10.0 * Vec2::ONE);
        let b = Collider::rectangle(5.0 * Vec2::ONE, 10.0 * Vec2::ONE);
        let t = Transform::default();

        assert!(
            super::are_colliding((&a, &t), (&b, &t)),
            "Colliders are colliding"
        )
    }

    #[test]
    fn get_edges() {
        let a = Collider::rectangle(Vec2::ZERO, 10.0 * Vec2::ONE);
        let edges = vec![
            Vec2::new(0.0, -10.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(0.0, 10.0),
            Vec2::new(-10.0, 0.0),
        ];

        let result: Vec<_> = super::edges_between(&a.0).collect();

        assert_eq!(edges, result, "Doesn't calculate the right edges")
    }
}
