use std::{collections::BTreeSet, sync::OnceLock};

use crate::vector::Vector2D;

// Imagining that a line splits up the 2D plane into two halves, this enum describes
// which half a point is in relation to the line, or if the point is exactly on the line.
#[derive(PartialEq, Debug)]
enum PointLineRelation {
    SideA,
    SideB,
    Intersection,
}

// A line can be represented by the equation: ax + by + c = 0
// where (x0, y0) and (x1, y1) are points on the line, a = (y1 - y0), b = (x0 - x1), and c = - a*x0 - b*y0
fn get_point_line_relation(p: &Vector2D<f32>, a: f32, b: f32, c: f32) -> PointLineRelation {
    let p_position = a * p[0] + b * p[1] + c;
    if p_position == 0.0 {
        return PointLineRelation::Intersection;
    }

    if p_position > 0.0 {
        return PointLineRelation::SideA;
    }

    PointLineRelation::SideB
}

// Uses the sweep line algorithm to determine if a polygon is simple (i.e. does not intersect itself).
fn is_simple_polygon(polygon: &[Vector2D<f32>]) -> bool {
    #[derive(Clone)]
    enum EventType {
        Start,
        End,
        Vertical,
    }

    #[derive(Clone)]
    struct Event {
        edge: usize,
        event_type: EventType,
        position: f32,
    }

    let mut events = Vec::new();
    for i in 0..polygon.len() {
        let a = i;
        let b = (i + 1) % polygon.len();

        let node_a = &polygon[a];
        let node_b = &polygon[b];

        if node_a[0] != node_b[0] {
            events.push(Event {
                edge: i,
                event_type: EventType::Start,
                position: node_a[0].min(node_b[0]),
            });
            events.push(Event {
                edge: i,
                event_type: EventType::End,
                position: node_a[0].max(node_b[0]),
            });
        } else {
            events.push(Event {
                edge: i,
                event_type: EventType::Vertical,
                position: node_a[0],
            });
        }
    }

    events.sort_by(|event_a, event_b| {
        event_a
            .position
            .partial_cmp(&event_b.position)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut active_edges: BTreeSet<usize> = BTreeSet::new();
    let mut prev_event: Option<Event> = None;
    let mut has_vertical_edges = false;

    let remove_passed_vertical_edges =
        |edges: &mut BTreeSet<usize>,
         event: &Event,
         prev_event: &Option<Event>,
         has_vertical_edges: &mut bool| {
            if !*has_vertical_edges {
                return;
            }

            match &prev_event {
                Some(prev) if prev.position < event.position => {
                    edges.retain(|edge| {
                        let node_a = *edge;
                        let node_b = (*edge + 1) % polygon.len();
                        polygon[node_a][0] != polygon[node_b][0]
                    });
                    *has_vertical_edges = false;
                }
                _ => {}
            }
        };

    let edges_are_adjacent =
        |e1: usize, e2: usize| e1 == (e2 + 1) % polygon.len() || e2 == (e1 + 1) % polygon.len();

    for event in events {
        remove_passed_vertical_edges(
            &mut active_edges,
            &event,
            &prev_event,
            &mut has_vertical_edges,
        );
        prev_event = Some(event.clone());

        match event.event_type {
            EventType::End => {
                active_edges.remove(&event.edge);
                continue;
            }
            EventType::Vertical => {
                has_vertical_edges = true;
            }
            _ => {}
        }

        // A line can be represented by the equation: ax + by + c = 0
        // where (x0, y0) and (x1, y1) are points on the line, a = (y1 - y0), b = (x0 - x1), and c = -a*x0 - b*y0
        let curr_node0 = &polygon[event.edge];
        let curr_node1 = &polygon[(event.edge + 1) % polygon.len()];
        let curr_edge_a = curr_node1[1] - curr_node0[1];
        let curr_edge_b = curr_node0[0] - curr_node1[0];
        let curr_edge_c = -curr_edge_a * curr_node0[0] - curr_edge_b * curr_node0[1];

        for test_edge in active_edges.iter() {
            if edges_are_adjacent(*test_edge, event.edge) {
                continue;
            }

            let test_node0 = &polygon[*test_edge];
            let test_node1 = &polygon[(*test_edge + 1) % polygon.len()];

            let test_node0_on_line =
                get_point_line_relation(test_node0, curr_edge_a, curr_edge_b, curr_edge_c);
            let test_node1_on_line =
                get_point_line_relation(test_node1, curr_edge_a, curr_edge_b, curr_edge_c);
            match (test_node0_on_line, test_node1_on_line) {
                (PointLineRelation::SideA, PointLineRelation::SideA)
                | (PointLineRelation::SideB, PointLineRelation::SideB) => {
                    continue;
                }
                _ => {}
            }

            let test_edge_a = test_node1[1] - test_node0[1];
            let test_edge_b = test_node0[0] - test_node1[0];
            let test_edge_c = -test_edge_a * test_node0[0] - test_edge_b * test_node0[1];

            let curr0_on_line =
                get_point_line_relation(curr_node0, test_edge_a, test_edge_b, test_edge_c);
            let curr1_on_line =
                get_point_line_relation(curr_node1, test_edge_a, test_edge_b, test_edge_c);

            match (curr0_on_line, curr1_on_line) {
                (PointLineRelation::SideA, PointLineRelation::SideA)
                | (PointLineRelation::SideB, PointLineRelation::SideB) => {
                    continue;
                }
                _ => {}
            }

            return false;
        }

        active_edges.insert(event.edge);
    }

    true
}

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
            for v in 0..vertices.len() {
                if v == prev_idx || v == curr_idx || v == next_idx {
                    continue;
                }

                let point_is_in_triangle = if is_wound_counter_clockwise {
                    is_point_in_triangle(&polygon[vertices[v]], prev, curr, next)
                } else {
                    is_point_in_triangle(&polygon[vertices[v]], next, curr, prev)
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

pub fn triangulate(polygon: &[Vector2D<f32>]) -> Option<Vec<[usize; 3]>> {
    static COMPLEX_POLYGON_WARNING: OnceLock<()> = OnceLock::new();
    if !is_simple_polygon(polygon) {
        COMPLEX_POLYGON_WARNING.get_or_init(|| {
            eprintln!("Warning: Attempted to triangulate a non-simple polygon. The triangulation will be skipped.");
        });
        return None;
    }

    triangulate_by_ear_clipping(polygon)
}

#[cfg(test)]
mod tests {
    use crate::vector::Vector2D;

    use super::*;

    #[test]
    fn points_on_opposite_sides_of_line() {
        let a = Vector2D::from([0.0, 0.0]);
        let b = Vector2D::from([4.0, 4.0]);

        let p1 = Vector2D::from([1.0, 2.0]);
        let p2 = Vector2D::from([2.0, 1.0]);

        let line_a = b[1] - a[1];
        let line_b = a[0] - b[0];
        let line_c = -line_a * a[0] - line_b * a[1];

        assert_ne!(
            get_point_line_relation(&p1, line_a, line_b, line_c),
            PointLineRelation::Intersection
        );
        assert_ne!(
            get_point_line_relation(&p2, line_a, line_b, line_c),
            PointLineRelation::Intersection
        );
        assert_ne!(
            get_point_line_relation(&p1, line_a, line_b, line_c),
            get_point_line_relation(&p2, line_a, line_b, line_c)
        );
    }

    #[test]
    fn points_on_same_sides_of_line() {
        let a = Vector2D::from([0.0, 0.0]);
        let b = Vector2D::from([4.0, 4.0]);

        let p1 = Vector2D::from([1.0, 2.0]);
        let p2 = Vector2D::from([1.0, 3.0]);

        let line_a = b[1] - a[1];
        let line_b = a[0] - b[0];
        let line_c = -line_a * a[0] - line_b * a[1];

        assert_ne!(
            get_point_line_relation(&p1, line_a, line_b, line_c),
            PointLineRelation::Intersection
        );
        assert_ne!(
            get_point_line_relation(&p2, line_a, line_b, line_c),
            PointLineRelation::Intersection
        );
        assert_eq!(
            get_point_line_relation(&p1, line_a, line_b, line_c),
            get_point_line_relation(&p2, line_a, line_b, line_c)
        );
    }

    #[test]
    fn points_intersecting_line_once() {
        let a = Vector2D::from([0.0, 0.0]);
        let b = Vector2D::from([4.0, 4.0]);

        let p1 = Vector2D::from([2.0, 2.0]);
        let p2 = Vector2D::from([2.0, 1.0]);

        let line_a = b[1] - a[1];
        let line_b = a[0] - b[0];
        let line_c = -line_a * a[0] - line_b * a[1];

        assert_eq!(
            get_point_line_relation(&p1, line_a, line_b, line_c),
            PointLineRelation::Intersection
        );
        assert_ne!(
            get_point_line_relation(&p1, line_a, line_b, line_c),
            get_point_line_relation(&p2, line_a, line_b, line_c)
        );
    }

    #[test]
    fn points_intersecting_line_twice() {
        let a = Vector2D::from([0.0, 0.0]);
        let b = Vector2D::from([4.0, 4.0]);

        let p1 = Vector2D::from([2.0, 2.0]);
        let p2 = Vector2D::from([1.0, 1.0]);

        let line_a = b[1] - a[1];
        let line_b = a[0] - b[0];
        let line_c = -line_a * a[0] - line_b * a[1];

        assert_eq!(
            get_point_line_relation(&p1, line_a, line_b, line_c),
            PointLineRelation::Intersection
        );
        assert_eq!(
            get_point_line_relation(&p2, line_a, line_b, line_c),
            PointLineRelation::Intersection
        );
    }

    #[test]
    fn simple_polygon() {
        let square = [
            Vector2D::from([0.0, 0.0]),
            Vector2D::from([1.0, 0.0]),
            Vector2D::from([1.0, 1.0]),
            Vector2D::from([0.0, 1.0]),
        ];
        assert!(is_simple_polygon(&square));

        let triangle = [
            Vector2D::from([0.0, 0.0]),
            Vector2D::from([4.0, 0.0]),
            Vector2D::from([2.0, 3.0]),
        ];
        assert!(is_simple_polygon(&triangle));
    }

    #[test]
    fn hourglass_polygon() {
        let hourglass = [
            Vector2D::from([0.0, 0.0]),
            Vector2D::from([2.0, 2.0]),
            Vector2D::from([0.0, 2.0]),
            Vector2D::from([2.0, 0.0]),
        ];
        assert!(!is_simple_polygon(&hourglass));
    }

    #[test]
    fn lightning_bolt_polygon() {
        let lightning_bolt = [
            Vector2D::from([0.0, 0.0]),
            Vector2D::from([1.0, 1.0]),
            Vector2D::from([1.0, 2.0]),
            Vector2D::from([2.0, 2.0]),
        ];
        assert!(!is_simple_polygon(&lightning_bolt));
    }

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
