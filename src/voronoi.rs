use std::collections::HashMap;

use triangulation::{Delaunay, EdgeIndex, Point, PointIndex};
use web_sys::console;

pub struct Voronoi {
    pub center_points: usize,
    pub cells: HashMap<PointIndex, VoronoiCell>,
    pub vertices: HashMap<EdgeIndex, VoronoiVertex>,
}

pub struct VoronoiCell {
    pub vertices: Vec<EdgeIndex>,
    pub adjacent_cells: Vec<PointIndex>,
    pub border_cell: bool,
}

pub struct VoronoiVertex {
    pub coords: Point,
    pub connected_vertices: Vec<EdgeIndex>,
    pub connected_cells: [PointIndex; 3],
}

impl Voronoi {
    pub fn from_delaunay(delaunay: &Delaunay, points: &[Point], num_center_points: usize) -> Self {
        let dcel = &delaunay.dcel;

        let mut voronoi = Voronoi {
            center_points: num_center_points,
            cells: HashMap::with_capacity(num_center_points),
            vertices: HashMap::with_capacity(points.len()),
        };

        for edge in (0..dcel.vertices.len()).map(|e| EdgeIndex::from(e)) {
            // Get the point that this edge points to.
            let point = dcel.vertices[dcel.next_edge(edge)];
            // If this point is not on the boundry and we have not created a
            // voronoi cell for it, create one.
            if point < num_center_points.into() && voronoi.cells.get(&point).is_none() {
                // Incoming edges to this point.
                let edges: Vec<_> = dcel.incoming_edges(point).collect();
                // The triangles that use this point.
                let vertices: Vec<_> = edges.iter().map(|&e| dcel.triangle_first_edge(e)).collect();
                // Start point of the incoming edges. Points kept only if they
                // are not on the boundry.
                let adjacent_cells: Vec<_> = edges
                    .iter()
                    .map(|&e| dcel.vertices[e])
                    .filter(|&c| c < num_center_points.into())
                    .collect();
                // If the number of incoming edges is greater than the number of
                // adjacent cells, then this cell is on the border.
                let border_cell = edges.len() > adjacent_cells.len();
                voronoi.cells.insert(
                    point,
                    VoronoiCell {
                        vertices,
                        adjacent_cells,
                        border_cell,
                    }
                );
            }

            let triangle = dcel.triangle_first_edge(edge);
            // Create a vertex for this triangle if one doesn't yet exist.
            if voronoi.vertices.get(&triangle).is_none() {
                let point = dcel.triangle(triangle, points).circumcenter();
                let vertex = VoronoiVertex {
                    coords: Point {
                        x: point.x.floor(),
                        y: point.y.floor(),
                    },
                    connected_vertices: dcel.triangle_adjacent_triangles(triangle).collect(),
                    connected_cells: dcel.triangle_points(triangle),
                };
                voronoi.vertices.insert(triangle, vertex);
            }
        }

        voronoi
    }

    pub fn get_cell_vertices<'a>(
        &'a self,
        cell_index: PointIndex
    ) -> Option<impl Iterator<Item = &VoronoiVertex> + 'a> {
        self.cells.get(&cell_index).map(|c|
            c.vertices.iter().map(move |v| self.vertices.get(v).expect("No vertices for cell"))
        )
    }
}
