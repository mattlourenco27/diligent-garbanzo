use crate::vector::Vector2D;

/// Computes the signed area of a polygon given by a list of points.
/// The area is positive if the points are wound in the counter-clockwise direction
/// and zero if the polygon is collinear or has fewer than 3 points.
fn signed_polygon_area(points: &[Vector2D<f32>]) -> f32 {
    let n = points.len();
    if n < 3 {
        return 0.0;
    }

    let mut area = 0.0;
    for i in 0..n {
        let j = (i + 1) % n;
        area += points[i].cross(&points[j]);
    }
    area / 2.0
}

/// For a chain of points (a, b, c), on a polygon wound in the counter-clockwise direction,
/// returns true if point p is inside the triangle formed by a, b, and c.
fn is_point_in_triangle(
    p: &Vector2D<f32>,
    a: &Vector2D<f32>,
    b: &Vector2D<f32>,
    c: &Vector2D<f32>,
) -> bool {
    let ab = b - a;
    let ap = p - a;
    if ab.cross(&ap) < 0.0 {
        return false;
    }

    let bc = c - b;
    let bp = p - b;
    if bc.cross(&bp) < 0.0 {
        return false;
    }

    let ca = a - c;
    let cp = p - c;
    if ca.cross(&cp) < 0.0 {
        return false;
    }

    true
}

#[derive(PartialEq, Debug)]
enum AngleType {
    Convex,
    Reflex,
    Collinear,
}

/// For a chain of points (a, b, c), on a polygon wound in the counter-clockwise direction,
/// returns the type of angle formed at point b.
fn get_angle_type(a: &Vector2D<f32>, b: &Vector2D<f32>, c: &Vector2D<f32>) -> AngleType {
    let product = (c - b).cross(&(a - b));
    if product > 0.0 {
        AngleType::Convex
    } else if product < 0.0 {
        AngleType::Reflex
    } else {
        AngleType::Collinear
    }
}

fn triangulate_by_ear_clipping(polygon: &[Vector2D<f32>]) -> Option<Vec<[usize; 3]>> {
    let n = polygon.len();
    if n < 3 {
        return None;
    }

    let area = signed_polygon_area(polygon);
    if area == 0.0 {
        return None;
    }

    let is_wound_counter_clockwise = area > 0.0;

    let mut vertices: Vec<usize> = (0..n).collect();
    {
        let mut i = 0;
        while i < vertices.len() {
            let prev = &polygon[vertices[(i + vertices.len() - 1) % vertices.len()]];
            let curr = &polygon[vertices[i]];
            let next = &polygon[vertices[(i + 1) % vertices.len()]];

            let angle_type = get_angle_type(prev, curr, next);
            if angle_type == AngleType::Collinear {
                vertices.remove(i);
            } else {
                i += 1;
            }
        }
    }

    let mut triangles = Vec::new();
    triangles.reserve_exact(vertices.len() - 2); // There will be num_vertices - 2 triangles

    while vertices.len() > 3 {
        let mut ear_found = false;
        for i in 0..vertices.len() {
            let prev_idx = (i + vertices.len() - 1) % vertices.len();
            let curr_idx = i;
            let next_idx = (i + 1) % vertices.len();
            let prev = &polygon[vertices[prev_idx]];
            let curr = &polygon[vertices[curr_idx]];
            let next = &polygon[vertices[next_idx]];

            let angle_type = get_angle_type(prev, curr, next);
            if (angle_type == AngleType::Convex) != is_wound_counter_clockwise {
                continue;
            }

            let mut is_ear = true;
            for &v in &vertices {
                if v == prev_idx || v == curr_idx || v == next_idx {
                    continue;
                }

                let point_is_in_triangle = if is_wound_counter_clockwise {
                    is_point_in_triangle(&polygon[v], prev, curr, next)
                } else {
                    is_point_in_triangle(&polygon[v], next, curr, prev)
                };

                if point_is_in_triangle {
                    is_ear = false;
                    break;
                }
            }

            if is_ear {
                if is_wound_counter_clockwise {
                    triangles.push([vertices[prev_idx], vertices[curr_idx], vertices[next_idx]]);
                } else {
                    triangles.push([vertices[next_idx], vertices[curr_idx], vertices[prev_idx]]);
                }
                vertices.remove(i);
                ear_found = true;
                break;
            }
        }

        if !ear_found {
            return None;
        }
    }

    if is_wound_counter_clockwise {
        triangles.push([vertices[0], vertices[1], vertices[2]]);
    } else {
        triangles.push([vertices[2], vertices[1], vertices[0]]);
    }

    Some(triangles)
}

#[cfg(test)]
mod tests {
    use crate::vector::Vector2D;

    use super::*;

    #[test]
    fn polygon_area() {
        let square = [
            Vector2D::from([0.0, 0.0]),
            Vector2D::from([1.0, 0.0]),
            Vector2D::from([1.0, 1.0]),
            Vector2D::from([0.0, 1.0]),
        ];
        assert_eq!(signed_polygon_area(&square), 1.0);

        let triangle = [
            Vector2D::from([0.0, 0.0]),
            Vector2D::from([4.0, 0.0]),
            Vector2D::from([2.0, 3.0]),
        ];
        assert_eq!(signed_polygon_area(&triangle), 6.0);

        let line = [Vector2D::from([0.0, 0.0]), Vector2D::from([1.0, 1.0])];
        assert_eq!(signed_polygon_area(&line), 0.0);

        let point = [Vector2D::from([2.0, 3.0])];
        assert_eq!(signed_polygon_area(&point), 0.0);
    }

    #[test]
    fn polygon_area_is_signed() {
        let square = [
            Vector2D::from([0.0, 0.0]),
            Vector2D::from([1.0, 0.0]),
            Vector2D::from([1.0, 1.0]),
            Vector2D::from([0.0, 1.0]),
        ];
        assert_eq!(signed_polygon_area(&square), 1.0);

        let square_reversed = [
            Vector2D::from([0.0, 0.0]),
            Vector2D::from([0.0, 1.0]),
            Vector2D::from([1.0, 1.0]),
            Vector2D::from([1.0, 0.0]),
        ];
        assert_eq!(signed_polygon_area(&square_reversed), -1.0);
    }

    #[test]
    fn convex_angle() {
        let a = Vector2D::from([0.0, 0.0]);
        let b = Vector2D::from([1.0, 0.0]);
        let c = Vector2D::from([1.0, 1.0]);
        assert_eq!(get_angle_type(&a, &b, &c), AngleType::Convex);
    }

    #[test]
    fn reflex_angle() {
        let a = Vector2D::from([0.0, 0.0]);
        let b = Vector2D::from([1.0, 0.0]);
        let c = Vector2D::from([1.0, -1.0]);
        assert_eq!(get_angle_type(&a, &b, &c), AngleType::Reflex);
    }

    #[test]
    fn collinear_angle() {
        let a = Vector2D::from([0.0, 0.0]);
        let b = Vector2D::from([1.0, 0.0]);
        let c = Vector2D::from([2.0, 0.0]);
        assert_eq!(get_angle_type(&a, &b, &c), AngleType::Collinear);
    }

    #[test]
    fn switchback_angle() {
        let a = Vector2D::from([0.0, 0.0]);
        let b = Vector2D::from([2.0, 0.0]);
        let c = Vector2D::from([1.0, 0.0]);
        assert_eq!(get_angle_type(&a, &b, &c), AngleType::Collinear);
    }

    #[test]
    fn point_in_triangle() {
        let a = Vector2D::from([0.0, 0.0]);
        let b = Vector2D::from([4.0, 0.0]);
        let c = Vector2D::from([2.0, 3.0]);

        let inside = Vector2D::from([2.0, 1.0]);
        assert!(is_point_in_triangle(&inside, &a, &b, &c));
    }

    #[test]
    fn point_outside_of_triangle() {
        let a = Vector2D::from([0.0, 0.0]);
        let b = Vector2D::from([4.0, 0.0]);
        let c = Vector2D::from([2.0, 3.0]);

        let outside = Vector2D::from([4.0, 3.0]);
        assert!(!is_point_in_triangle(&outside, &a, &b, &c));
    }

    #[test]
    fn point_on_edge_of_triangle() {
        let a = Vector2D::from([0.0, 0.0]);
        let b = Vector2D::from([4.0, 0.0]);
        let c = Vector2D::from([2.0, 3.0]);

        let on_edge = Vector2D::from([2.0, 0.0]);
        assert!(is_point_in_triangle(&on_edge, &a, &b, &c));
    }

    #[test]
    fn point_on_vertex_of_triangle() {
        let a = Vector2D::from([0.0, 0.0]);
        let b = Vector2D::from([4.0, 0.0]);
        let c = Vector2D::from([2.0, 3.0]);

        let on_vertex = Vector2D::from([0.0, 0.0]);
        assert!(is_point_in_triangle(&on_vertex, &a, &b, &c));
    }

    #[test]
    fn ear_clip_line() {
        let line = [Vector2D::from([0.0, 0.0]), Vector2D::from([1.0, 1.0])];
        assert_eq!(triangulate_by_ear_clipping(&line), None);
    }

    #[test]
    fn ear_clip_collinear_points() {
        let points = [
            Vector2D::from([0.0, 0.0]),
            Vector2D::from([1.0, 1.0]),
            Vector2D::from([2.0, 2.0]),
            Vector2D::from([3.0, 3.0]),
        ];
        assert_eq!(triangulate_by_ear_clipping(&points), None);
    }

    #[test]
    fn ear_clip_triangle() {
        let triangle = [
            Vector2D::from([0.0, 0.0]),
            Vector2D::from([4.0, 0.0]),
            Vector2D::from([2.0, 3.0]),
        ];
        assert_eq!(
            triangulate_by_ear_clipping(&triangle),
            Some(vec![[0, 1, 2]])
        );
    }

    #[test]
    fn ear_clip_reverse_triangle() {
        let triangle = [
            Vector2D::from([0.0, 0.0]),
            Vector2D::from([2.0, 3.0]),
            Vector2D::from([4.0, 0.0]),
        ];
        assert_eq!(
            triangulate_by_ear_clipping(&triangle),
            Some(vec![[2, 1, 0]])
        );
    }

    #[test]
    fn ear_clip_rectangle() {
        let rectangle = [
            Vector2D::from([0.0, 0.0]),
            Vector2D::from([4.0, 0.0]),
            Vector2D::from([4.0, 3.0]),
            Vector2D::from([0.0, 3.0]),
        ];
        let result = triangulate_by_ear_clipping(&rectangle);
        assert!(result.is_some());
        let triangles = result.unwrap();
        assert_eq!(triangles.len(), 2);
        assert!(triangles.contains(&[0, 1, 2]) || triangles.contains(&[1, 2, 3]));
        assert!(triangles.contains(&[2, 3, 0]) || triangles.contains(&[3, 0, 1]));
    }

    #[test]
    fn ear_clip_reverse_rectangle() {
        let rectangle = [
            Vector2D::from([0.0, 0.0]),
            Vector2D::from([0.0, 3.0]),
            Vector2D::from([4.0, 3.0]),
            Vector2D::from([4.0, 0.0]),
        ];
        let result = triangulate_by_ear_clipping(&rectangle);
        assert!(result.is_some());
        let triangles = result.unwrap();
        assert_eq!(triangles.len(), 2);
        assert!(triangles.contains(&[2, 1, 0]) || triangles.contains(&[3, 2, 1]));
        assert!(triangles.contains(&[0, 3, 2]) || triangles.contains(&[1, 0, 3]));
    }

    #[test]
    fn ear_clip_arrow() {
        let arrow = [
            Vector2D::from([0.0, 1.0]),
            Vector2D::from([1.0, 0.0]),
            Vector2D::from([0.0, 2.0]),
            Vector2D::from([-1.0, 0.0]),
        ];
        let result = triangulate_by_ear_clipping(&arrow);
        assert!(result.is_some());
        let triangles = result.unwrap();
        assert_eq!(triangles.len(), 2);
        assert!(triangles.contains(&[0, 1, 2]) && triangles.contains(&[0, 2, 3]));
    }
}
