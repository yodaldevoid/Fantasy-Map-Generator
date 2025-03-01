#![allow(unused_variables)]

mod heightmap;
mod util;
mod voronoi;
mod svg_test;

use std::collections::VecDeque;
use std::iter::successors;
#[cfg(target_arch = "wasm32")]
use std::panic;
use std::num::NonZeroU32;

#[cfg(target_arch = "wasm32")]
use js_sys::{Array, JsString};
use rand::{random, SeedableRng};
use rand::distributions::Distribution;
use rand::distributions::uniform::Uniform;
use rand::rngs::StdRng;
use svg::node::Value;
use svg::node::element::path::Data;
use triangulation::{Delaunay, EdgeIndex, Point, PointIndex};
use wasm_bindgen::prelude::*;

use heightmap::{HeightmapGenerator, OCEAN_HEIGHT, Template, WORLD_MAX};
use util::FloatExt;
use voronoi::Voronoi;
#[cfg(not(target_arch = "wasm32"))]
use svg_test::*;

#[wasm_bindgen(module = "/modules/ui-util.js")]
#[cfg(target_arch = "wasm32")]
extern "C" {
    #[wasm_bindgen(js_name = removeLoading)]
    fn remove_loading();
    #[wasm_bindgen(js_name = undrawAll)]
    fn undraw_all();
    #[wasm_bindgen]
    fn unfog();
    #[wasm_bindgen(js_name = drawCells)]
    fn __draw_cells(path: String);
    #[wasm_bindgen(js_name = clearCells)]
    fn clear_cells();
    // All arrays are arrays of strings
    #[wasm_bindgen(js_name = drawCoastline)]
    fn __draw_coastline(
        land_mask_paths: Array,
        water_mask_paths: Array,
        coastline_paths: Array,
        coastline_ids: Array,
        lake_groups: Array,
        lake_paths: Array,
        lake_ids: Array,
    );
    // All arrays are arrays of strings
    #[wasm_bindgen(js_name = drawHeightmap)]
    fn __draw_heightmap(
        height_paths: Array,
        height_colors: Array,
        height_values: &[u8],
    );
    #[wasm_bindgen(js_name = clearHeightmap)]
    fn clear_heightmap();
}

#[macro_export]
macro_rules! log {
    ( $( $t:tt )* ) => {
        if cfg!(target_arch = "wasm32") {
            web_sys::console::log_1(&format!( $( $t )* ).into());
        } else {
            println!( $( $t )* )
        }
    }
}

#[macro_export]
macro_rules! err {
    ( $( $t:tt )* ) => {
        if cfg!(target_arch = "wasm32") {
            web_sys::console::error_1(&format!( $( $t )* ).into());
        } else {
            eprintln!( $( $t )* );
        }
    }
}

#[macro_export]
macro_rules! time_start {
    ( $e:expr ) => {
        if cfg!(target_arch = "wasm32") {
            web_sys::console::time_with_label($e);
        } else {
            // TODO: track start time
            eprintln!("time_start: {}", $e);
        }
    }
}

#[macro_export]
macro_rules! time_end {
    ( $e:expr ) => {
        if cfg!(target_arch = "wasm32") {
            web_sys::console::time_end_with_label($e);
        } else {
            eprintln!("time_end: {}", $e);
        }
    }
}

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    #[cfg(target_arch = "wasm32")]
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    // TODO: Setup SVG

    // Create grid
    // Create pack?
    // generate seed
    // Map history
    // selected element
    // modules
    // notes
    let customization = MapCustomization::None;
    let winds = [225, 45, 225, 315, 135, 315]; // default wind directions
    // TODO: biomes
    // TODO: name bases

    // TODO: Load stored options from local storage
    // TODO: get graph/svg size from input field
    let graph_size = Size::new(1000, 1000);

    // TODO: setup landmass/ocean bases

    remove_loading();

    load_initial_map(graph_size);

    Ok(())
}

pub enum MapCustomization {
    None = 0,
    HightmapDraw = 1,
    StatesDraw = 2,
    AddStateBurg = 3,
    CulturesDraw = 4,
}

#[derive(Copy, Clone, Debug)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub fn new(width: u32, height: u32) -> Self {
        Size {
            width,
            height,
        }
    }
}

fn load_initial_map(graph_size: Size) -> Map {
    // TODO: if valid link in href, load the map
    // TODO: if there is a seed in the href, use seed
    // TODO: if map was saved and "load saved map" option checked, load from
    //       storage
    // TODO: else, generate a new map
    let density = NonZeroU32::new(1).unwrap();
    generate_map_on_load(graph_size, density)
}

fn generate_map_on_load(graph_size: Size, density: NonZeroU32) -> Map {
    // TODO: apply the default style, maybe do before
    // TODO: generate map
    let map = Map::generate(graph_size, density);
    // TODO: focus on the current target, may have been set by href
    // TODO: apply the current (set in local storage) layer preset
    map
}

const DENSITY_STEP: u32 = 10_000;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Coast {
    None,
    Beach,
    Shallows,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum FeatureType {
    Island(IslandGroup),
    Ocean,
    Lake(LakeGroup),
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum LakeGroup {
    Freshwater,
    Salt,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum IslandGroup {
    Continent,
    Island,
    Isle,
}

impl std::fmt::Display for FeatureType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FeatureType::Ocean => write!(f, "ocean"),
            FeatureType::Lake(LakeGroup::Freshwater) => write!(f, "freshwater"),
            FeatureType::Lake(LakeGroup::Salt) => write!(f, "salt"),
            FeatureType::Island(IslandGroup::Continent) => write!(f, "continent"),
            FeatureType::Island(IslandGroup::Island) => write!(f, "island"),
            FeatureType::Island(IslandGroup::Isle) => write!(f, "isle"),
        }
    }
}

#[derive(Debug)]
pub struct Feature {
    index: usize,
    land: bool,
    border: bool,
    ty: FeatureType,
}

pub struct Grid {
    pub size: Size,
    pub density: NonZeroU32,
    pub point_spacing: f32,
    pub cells_x: u32,
    pub cells_y: u32,
    pub boundary: Vec<Point>,
    pub points: Vec<Point>,
    pub voronoi: Voronoi,
    pub heights: Vec<u8>,
    // TODO: FeatureIndex?
    pub feature_map: Vec<Option<usize>>,
    pub features: Vec<Feature>,
    pub coasts: Vec<Coast>,
}

impl Grid {
    pub fn new(size: Size, density: NonZeroU32, rng: &mut StdRng) -> Self {
        time_start!("place_points");
        let cells_desired = DENSITY_STEP * density.get();
        // Spacing between points before jittering
        let spacing =
            ((size.width * size.height / cells_desired) as f32)
            .sqrt()
            .round_decimals(2);

        let cells_x = ((size.width as f32 + 0.5 * spacing) / spacing).floor() as u32;
        let cells_y =  ((size.height as f32 + 0.5 * spacing) / spacing).floor() as u32;
        // grid boundary points
        let boundary = Grid::generate_boundary_points(size, spacing);
        // jittered square grid
        let points = Grid::generate_jittered_grid(size, spacing, rng);
        time_end!("place_points");

        time_start!("calculate_delaunay");
        let mut allpoints = Vec::with_capacity(points.len() + boundary.len());
        allpoints.extend_from_slice(&points);
        allpoints.extend_from_slice(&boundary);
        let mut delaunay = Delaunay::new(allpoints.as_slice()).unwrap();
        delaunay.dcel.init_revmap();
        time_end!("calculate_delaunay");

        time_start!("calculate_voronoi");
        let voronoi = Voronoi::from_delaunay(&delaunay, &allpoints, points.len());
        time_end!("calculate_voronoi");

        let heights = vec![0; voronoi.cells.len()];
        let feature_map = vec![None; voronoi.cells.len()];
        let features = vec![];
        let coasts = vec![Coast::None; voronoi.cells.len()];

        Grid {
            size,
            density,
            point_spacing: spacing,
            cells_x,
            cells_y,
            boundary,
            points,
            voronoi,
            heights,
            feature_map,
            features,
            coasts,
        }
    }

    fn generate_boundary_points(size: Size, spacing: f32) -> Vec<Point> {
        let offset = (spacing * -1.0).round();
        let b_spacing = spacing * 2.0;
        let w = size.width as f32 - offset * 2.0;
        let h = size.height as f32 - offset * 2.0;
        let len_x = (w / b_spacing).ceil() - 1.0;
        let len_y = (h / b_spacing).ceil() - 1.0;

        let mut boundary = Vec::new();
        for i in successors(Some(0.5), |v| Some(v + 1.0)).take_while(|v| *v < len_x) {
            let x = (w * i / len_x + offset).ceil();
            boundary.push(Point::new(x, offset));
            boundary.push(Point::new(x, h + offset));
        }
        for i in successors(Some(0.5), |v| Some(v + 1.0)).take_while(|v| *v < len_y) {
            let y = (h * i / len_y + offset).ceil();
            boundary.push(Point::new(offset, y));
            boundary.push(Point::new(w + offset, y));
        }
        boundary
    }

    fn generate_jittered_grid(size: Size, spacing: f32, rng: &mut StdRng) -> Vec<Point> {
        // Square radius
        let radius = spacing / 2.0;
        // Max deviation
        let jittering = radius * 0.9;
        let jitter_uniform = Uniform::new(-jittering, jittering);
        let mut jitter = move || jitter_uniform.sample(rng);;

        let width = size.width as f32;
        let height = size.height as f32;

        let mut points = Vec::new();
        for y in successors(Some(radius), |v| Some(v + spacing)).take_while(|v| *v < width) {
            for x in successors(Some(radius), |v| Some(v + spacing)).take_while(|v| *v < height) {
                let xj = (x + jitter()).round_decimals(2).min(width);
                let yj = (y + jitter()).round_decimals(2).min(height);
                points.push(Point::new(xj, yj));
            }
        }
        points
    }

    // Return cell index on a regular square grid.
    pub fn coords_to_cell_index(&self, x: f32, y: f32) -> PointIndex {
        ((
            ((x / self.point_spacing) as u32).min(self.cells_x - 1)
                + ((y / self.point_spacing) as u32).min(self.cells_y - 1) * self.cells_x
        ) as usize).into()
    }

    pub fn mark_features(&mut self, rng: &mut StdRng, seed: u64) {
        time_start!("mark_features");

        *rng = StdRng::seed_from_u64(seed);
        self.coasts = vec![Coast::None; self.voronoi.cells.len()];
        self.feature_map = vec![None; self.voronoi.cells.len()];
        self.features.clear();

        let mut i = 0;
        let mut queue = VecDeque::new();
        queue.push_back(0);
        loop {
            self.feature_map[queue[0]] = Some(i);
            let land = self.heights[queue[0]] >= OCEAN_HEIGHT;
            let mut border = false;

            while !queue.is_empty() {
                let q = queue.pop_front().unwrap();
                let cell = &self.voronoi.cells[&q.into()];
                if cell.border_cell {
                    border = true;
                }
                for a in cell.adjacent_cells.iter() {
                    let adj_land = self.heights[a.as_usize()] >= OCEAN_HEIGHT;
                    if land == adj_land && self.feature_map[a.as_usize()].is_none() {
                        self.feature_map[a.as_usize()] = Some(i);
                        queue.push_back(a.as_usize());
                    }
                    if land && !adj_land {
                        self.coasts[q] = Coast::Beach;
                        self.coasts[a.as_usize()] = Coast::Shallows;
                    }
                }
            }

            // TODO: get the right groups
            let ty = match (land, border) {
                (true, _) => FeatureType::Island(IslandGroup::Island),
                (false, true) => FeatureType::Ocean,
                (false, false) => FeatureType::Lake(LakeGroup::Freshwater),
            };
            self.features.push(Feature {
                index: i,
                land,
                border,
                ty,
            });

            if let Some((idx, _)) = self.feature_map.iter().enumerate().skip_while(|(_, f)| f.is_some()).next() {
                queue.push_back(idx);
                i += 1;
            } else {
                break;
            }
        }

        time_end!("mark_features");
    }
}

struct Map {
    seed: u64,
    rng: StdRng,
    grid: Grid,
}

impl Map {
    fn generate(graph_size: Size, density: NonZeroU32) -> Self {
        let seed = random();
        Self::generate_with_seed(graph_size, density, seed)
    }

    // TODO: stuff to happen before this function
    // invoke active zooming
    // generate a new seed if needed
    // update the map size
    // randomizing the options
    fn generate_with_seed(graph_size: Size, density: NonZeroU32, seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);

        let mut grid = Grid::new(graph_size, density, &mut rng);

        time_start!("generate_hightmap");
        HeightmapGenerator::generate_with_template(&mut grid, &mut rng, Template::Isthmus);
        time_end!("generate_hightmap");

        grid.mark_features(&mut rng, seed);
        // TODO: open near sea lakes

        draw_coastline(
            &grid.voronoi,
            &grid.heights,
            &grid.feature_map,
            &grid.features,
            &grid.coasts,
        );
        draw_heightmap(&grid);
        draw_cells(&grid);

        // TODO: calculate map coords
        // TODO: calculate temperatures
        // TODO: generate precipitation
        // TODO: pack voronoi graph?
        // TODO: draw coastline

        // TODO: elevate lakes
        // TODO: generate rivers
        // TODO: define biomes

        // TODO: rank cells?
        // TODO: populate cultures
        // TODO: expand cultures
        // TODO: generate burgs and states
        // TODO: generate religions

        // TODO: draw states
        // TODO: draw borders
        // TODO: draw state labels
        // TODO: add zone?
        // TODO: add markers

        // TODO: print stats

        Map {
            seed,
            rng,
            grid,
        }
    }
    // TODO: stuff to happen after function
    // draw the scale bar
    // TODO: draw ocean layers
}

fn draw_cells(grid: &Grid) {
    let mut data = Data::new();
    for mut vertices in grid.voronoi.get_cell_vertex_coords() {
        if let Some(start) = vertices.next() {
            data = data.move_to((start.x, start.y));
            for vertex in vertices {
                data = data.line_to((vertex.x, vertex.y));
            }
            data = data.close();
        }
    }

    let data: Value = data.into();

    __draw_cells(data.to_string());
}

fn draw_coastline(
    voronoi: &Voronoi,
    heights: &[u8],
    feature_map: &[Option<usize>],
    features: &[Feature],
    coasts: &[Coast],
) {
    time_start!("draw_coastline");

    let mut used = vec![false; features.len()];
    let mut land_mask_paths = Vec::new();
    let mut water_mask_paths = Vec::new();
    let mut lake_paths = Vec::new();
    let mut lake_groups = Vec::new();
    let mut lake_ids = Vec::new();
    let mut coastline_paths = Vec::new();
    let mut coastline_ids = Vec::new();

    let find_start = |i: usize, ty: Coast| {
        let cell = &voronoi.cells[&i.into()];
        if ty == Coast::Shallows && cell.border_cell {
            // Use a border cell.
            cell.vertices
                .iter()
                .find(|v|
                    voronoi.vertices[v]
                        .connected_cells
                        .iter()
                        .any(|&c| voronoi.is_border_point(c))
                )
                .copied()
        } else {
            cell.adjacent_cells
                .iter()
                .enumerate()
                .filter(|(_, c)| coasts[c.as_usize()] == ty)
                .min_by_key(|(_, c)| c.as_usize())
                .map(|(i, _)| i)
                .map(|i| cell.vertices[i])
        }
    };

    let connect_vertices = |start: EdgeIndex, ty: Coast| {
        let mut chain = Vec::new();

        let mut current = start;
        for _ in 0..10_000 {
            let prev = chain.last().copied();

            chain.push(current);

            let c = &voronoi.vertices[&current].connected_cells;
            let v = &voronoi.vertices[&current].connected_vertices;

            // Is this vertex is on the border or for a cell of the right coast type
            let c0 = voronoi.is_border_point(c[0]) || coasts[c[0].as_usize()] == ty;
            let c1 = voronoi.is_border_point(c[1]) || coasts[c[1].as_usize()] == ty;
            let c2 = voronoi.is_border_point(c[2]) || coasts[c[2].as_usize()] == ty;

            // If the connected vertex is not the previous in the chain and it
            // is between coast types (or next to a border), make it the next
            // vertex
            if v[0] != prev && c0 != c1 {
                current = v[0].expect("Tried unwrapping connected vertex");
            } else if v[1] != prev && c1 != c2 {
                current = v[1].expect("Tried unwrapping connected vertex");
            } else if v[2] != prev && c2 != c0 {
                current = v[2].expect("Tried unwrapping connected vertex");
            }

            if current == *chain.last().unwrap() {
                err!("Next vertex not found");
                break;
            }
            if current == start {
                break;
            }
        }

        // Make the chain circular.
        chain.push(chain[0]);
        chain
    };

    for i in 0..voronoi.cells.len() {
        let start_from_edge = i == 0 && heights[i] >= OCEAN_HEIGHT;
        if !start_from_edge && coasts[i] == Coast::None {
            // non-edge cell
            continue;
        }

        let f = feature_map[i].expect("No mapped feature");
        if used[f] || features[f].ty == FeatureType::Ocean {
            continue;
        }

        let ty = if let FeatureType::Lake(_) = features[f].ty {
            Coast::Beach
        } else {
            Coast::Shallows
        };
        let start = find_start(i, ty);
        if start.is_none() {
            continue;
        }
        let start = start.unwrap();

        let connected_vertices = connect_vertices(start, ty);

        used[f] = true;
        let points: Vec<_> = connected_vertices
            .iter()
            .map(|v| voronoi.vertices[v].coords)
            .collect();

        // TODO: round coordinates
        let path: Value = basis_curve_closed_line_gen(&points).into();
        let id = format!("{}{}", features[f].ty, features[f].index);
        if let FeatureType::Lake(_) = features[f].ty {
            land_mask_paths.push(path.to_string());
            lake_paths.push(path.to_string());
            lake_groups.push(format!("{}", features[f].ty));
            lake_ids.push(id);
        } else {
            land_mask_paths.push(path.to_string());
            water_mask_paths.push(path.to_string());
            coastline_paths.push(path.to_string());
            coastline_ids.push(id);
        }
    }

    _draw_coastline(
        &land_mask_paths,
        &water_mask_paths,
        &coastline_paths,
        &coastline_ids,
        &lake_groups,
        &lake_paths,
        &lake_ids,
    );

    time_end!("draw_coastline");
}

#[cfg(target_arch = "wasm32")]
fn _draw_coastline(
    land_mask_paths: &[String],
    water_mask_paths: &[String],
    coastline_paths: &[String],
    coastline_ids: &[String],
    lake_groups: &[String],
    lake_paths: &[String],
    lake_ids: &[String],
) {
    __draw_coastline(
        land_mask_paths.iter().map(|s| JsString::from(s.as_str())).collect(),
        water_mask_paths.iter().map(|s| JsString::from(s.as_str())).collect(),
        coastline_paths.iter().map(|s| JsString::from(s.as_str())).collect(),
        coastline_ids.iter().map(|s| JsString::from(s.as_str())).collect(),
        lake_groups.iter().map(|s| JsString::from(s.as_str())).collect(),
        lake_paths.iter().map(|s| JsString::from(s.as_str())).collect(),
        lake_ids.iter().map(|s| JsString::from(s.as_str())).collect(),
    );
}

// TODO: skip parameter
fn draw_heightmap(grid: &Grid) {
    time_start!("draw_heightmap");

    let mut used = vec![false; grid.voronoi.cells.len()];
    let mut height_paths = Vec::new();
    let mut height_colors = Vec::new();
    let mut height_values = Vec::new();

    let skip = 5;

    let mut current_layer = OCEAN_HEIGHT;
    let mut ordered_cells: Vec<_> = (0..grid.voronoi.cells.len()).collect();
    ordered_cells.sort_by_key(|&i| grid.heights[i]);
    for i in ordered_cells {
        let h = grid.heights[i];
        if h > current_layer {
            current_layer += skip;
            if current_layer > WORLD_MAX {
                break;
            }
        }
        if h < current_layer {
            continue;
        }
        if used[i] {
            continue;
        }

        let on_border = grid
            .voronoi
            .cells[&i.into()]
            .adjacent_cells
            .iter()
            .any(|i| grid.heights[i.as_usize()] < h);
        if !on_border {
            continue;
        }
        let vertex = grid
            .voronoi
            .cells[&i.into()]
            .vertices
            .iter()
            .find(|v|
                grid.voronoi
                    .vertices[v]
                    .connected_cells
                    .iter()
                    .any(|&c| !grid.voronoi.is_border_point(c) && grid.heights[c.as_usize()] < h)
            )
            .expect("No border vertex found though used for border cell");

        let chain = {
            let mut chain = Vec::new();

            let start = *vertex;
            let mut current = start;
            for _ in 0..20_000 {
                let prev = chain.last().copied();

                chain.push(current);

                let c = &grid.voronoi.vertices[&current].connected_cells;
                let v = &grid.voronoi.vertices[&current].connected_vertices;

                for cell in c.iter() {
                    if !grid.voronoi.is_border_point(*cell) && grid.heights[cell.as_usize()] == h {
                        used[cell.as_usize()] = true;
                    }
                }
                // Is this vertex connected to a border cell or for a cell of the
                // right coast type
                let c0 = grid.voronoi.is_border_point(c[0]) || grid.heights[c[0].as_usize()] < h;
                let c1 = grid.voronoi.is_border_point(c[1]) || grid.heights[c[1].as_usize()] < h;
                let c2 = grid.voronoi.is_border_point(c[2]) || grid.heights[c[2].as_usize()] < h;

                // If the connected vertex is not the previous in the chain and it
                // is between coast types (or next to a border), make it the next
                // vertex
                if v[0] != prev && c0 != c1 {
                    current = v[0].expect("Tried unwrapping connected vertex");
                } else if v[1] != prev && c1 != c2 {
                    current = v[1].expect("Tried unwrapping connected vertex");
                } else if v[2] != prev && c2 != c0 {
                    current = v[2].expect("Tried unwrapping connected vertex");
                }

                if current == *chain.last().unwrap() {
                    err!("Next vertex not found");
                    break;
                }
                if current == start {
                    break;
                }
            }

            chain
        };
        if chain.len() < 3 {
            continue;
        }
        // TODO: line simplification
        let points: Vec<_> = chain.iter().map(|e| grid.voronoi.vertices[e].coords).collect();

        let path: Value = basis_curve_closed_line_gen(&points).into();
        height_paths.push(path.to_string());
        height_colors.push(format!{"#00{:02x}00", current_layer * 2});
        height_values.push(h);
    }

    _draw_heightmap(
        &height_paths,
        &height_colors,
        &height_values,
    );

    time_end!("draw_heightmap");
}

#[cfg(target_arch = "wasm32")]
fn _draw_heightmap(
    height_paths: &[String],
    height_colors: &[String],
    height_values: &[u8],
) {
    __draw_heightmap(
        height_paths.iter().map(|s| JsString::from(s.as_str())).collect(),
        height_colors.iter().map(|s| JsString::from(s.as_str())).collect(),
        &height_values,
    );
}

fn basis_curve_closed_line_gen(points: &[Point]) -> Data {
    let mut data = Data::new();

    match points.len() {
        0 => {}
        1 => {
            data = data
                .move_to((points[0].x, points[0].y))
                .close();
        }
        2 => {
            data = data
                .move_to((
                    (points[0].x + 2.0 * points[1].x) / 3.0,
                    (points[0].y + 2.0 * points[1].y) / 3.0,
                ))
                .line_to((
                    (points[1].x + 2.0 * points[0].x) / 3.0,
                    (points[1].y + 2.0 * points[0].y) / 3.0,
                ))
                .close();
        }
        _ => {
            let mut cycle_points = points.to_vec();
            cycle_points.push(points[0]);
            cycle_points.push(points[1]);
            cycle_points.push(points[2]);

            let mut windows = cycle_points.windows(3);
            if let Some(p) = windows.next() {
                data = data.move_to((
                    (p[0].x + 4.0 * p[1].x + p[2].x) / 6.0,
                    (p[0].y + 4.0 * p[1].y + p[2].y) / 6.0,
                ));

                for p in windows {
                    data = data.cubic_curve_to((
                        (2.0 * p[0].x + p[1].x) / 3.0,
                        (2.0 * p[0].y + p[1].y) / 3.0,
                        (p[0].x + 2.0 * p[1].x) / 3.0,
                        (p[0].y + 2.0 * p[1].y) / 3.0,
                        (p[0].x + 4.0 * p[1].x + p[2].x) / 6.0,
                        (p[0].y + 4.0 * p[1].y + p[2].y) / 6.0,
                    ));
                }
            }
        }
    }

    data
}
