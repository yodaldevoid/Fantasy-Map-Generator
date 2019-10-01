use std::collections::HashMap;

use triangulation::{Delaunay, EdgeIndex, Point, PointIndex};

pub struct Voronoi {
    center_points: usize,
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
            // Get the point that this edge points from.
            let point = dcel.vertices[edge];
            // If this point is not on the boundry and we have not created a
            // voronoi cell for it, create one.
            if point < num_center_points.into() && voronoi.cells.get(&point).is_none() {
                // Outgoing edges from this point.
                let edges: Vec<_> = dcel.outgoing_edges(point).collect();
                // The triangles that use this point.
                let vertices: Vec<_> = edges.iter().map(|&e| dcel.triangle_first_edge(e)).collect();
                // End point of the outgoing edges. Points kept only if they
                // are not on the boundry.
                let adjacent_cells: Vec<_> = edges
                    .iter()
                    .map(|&e| dcel.vertices[dcel.next_edge(e)])
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
                    connected_vertices: dcel
                        .triangle_edges(triangle)
                        .iter()
                        .filter_map(|&e| delaunay.dcel.twin(e))
                        .collect(),
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

    pub fn get_cell_vertex_coords<'a>(
        &'a self,
    ) -> impl Iterator<Item = impl Iterator<Item = Point> + 'a> + 'a {
        (0..self.center_points)
            .map(PointIndex::from)
            .filter_map(move |p| self.get_cell_vertices(p))
            .map(|i| i.map(|v| v.coords))
    }

    pub fn is_border_point(&self, p: PointIndex) -> bool {
        p.as_usize() >= self.center_points
    }
}
