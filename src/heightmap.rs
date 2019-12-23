use std::collections::VecDeque;
use std::ops::{Bound, Range, RangeBounds};
use std::num::NonZeroU32;

use rand::Rng;
use rand::distributions::Distribution;
use rand::distributions::uniform::Uniform;
use rand::rngs::StdRng;
use triangulation::PointIndex;

use crate::Grid;

pub const WORLD_MAX: u8 = 100;
pub const OCEAN_HEIGHT: u8 = 20;

// TODO: impl rand distribution
// TODO: support custom template
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Template {
    Archipelago,
    Atoll,
    Continents,
    HighIsland,
    Isthmus,
    LowIsland,
    Mediterranean,
    Pangaea,
    Peninsula,
    Volcano,
}

pub struct HeightmapGenerator;

impl HeightmapGenerator {
    pub fn generate(
        grid: &mut Grid,
        rng: &mut StdRng,
    ) {
        unimplemented!()
    }

    pub fn generate_with_template(
        grid: &mut Grid,
        rng: &mut StdRng,
        template: Template
    ) {
        // Clear the existing heights.
        grid.heights = vec![0; grid.voronoi.cells.len()];

        match template {
            Template::Archipelago => generate_archipelago(grid, rng),
            Template::Atoll => generate_atoll(grid, rng),
            Template::Continents => generate_continents(grid, rng),
            Template::HighIsland => generate_high_island(grid, rng),
            Template::Isthmus => generate_isthmus(grid, rng),
            Template::LowIsland => generate_low_island(grid, rng),
            Template::Mediterranean => generate_mediterranean(grid, rng),
            Template::Pangaea => generate_pangaea(grid, rng),
            Template::Peninsula => generate_peninsula(grid, rng),
            Template::Volcano => generate_volcano(grid, rng),
        }
    }
}

// Template generation functions
fn generate_archipelago(grid: &mut Grid, rng: &mut StdRng) {
    unimplemented!()
}

fn generate_atoll(grid: &mut Grid, rng: &mut StdRng) {
    unimplemented!()
}

fn generate_continents(grid: &mut Grid, rng: &mut StdRng) {
    unimplemented!()
}

fn generate_high_island(grid: &mut Grid, rng: &mut StdRng) {
    unimplemented!()
}

fn generate_isthmus(grid: &mut Grid, rng: &mut StdRng) {
    hill(grid, rng, 5..10, 15..30, 0.0..30.0, 0.0..20.0);
    hill(grid, rng, 5..10, 15..30, 10.0..50.0, 20.0..40.0);
    hill(grid, rng, 5..10, 15..30, 30.0..70.0, 40.0..60.0);
    hill(grid, rng, 5..10, 15..30, 50.0..90.0, 60.0..80.0);
    hill(grid, rng, 5..10, 15..30, 70.0..100.0, 80.0..100.00);
    smooth(grid, rng, 2);
    trough(grid, rng, 4..8, 15..30, 0.0..30.0, 0.0..20.0);
    trough(grid, rng, 4..8, 15..30, 10.0..50.0, 20.0..40.0);
    trough(grid, rng, 4..8, 15..30, 30.0..70.0, 40.0..60.0);
    trough(grid, rng, 4..8, 15..30, 50.0..90.0, 60.0..80.0);
    trough(grid, rng, 4..8, 15..30, 70.0..100.0, 80.0..100.00);
}

fn generate_low_island(grid: &mut Grid, rng: &mut StdRng) {
    unimplemented!()
}

fn generate_mediterranean(grid: &mut Grid, rng: &mut StdRng) {
    unimplemented!()
}

fn generate_pangaea(grid: &mut Grid, rng: &mut StdRng) {
    unimplemented!()
}

fn generate_peninsula(grid: &mut Grid, rng: &mut StdRng) {
    unimplemented!()
}

fn generate_volcano(grid: &mut Grid, rng: &mut StdRng) {
    unimplemented!()
}

// Feature generation functions
// TODO: refactor paired generators
// TODO: check that ranges are from low to high

enum Direction {
    Raise,
    Lower,
}

fn hill<C: RangeBounds<f32>>(
    grid: &mut Grid,
    rng: &mut StdRng,
    // Number of hills to place, more or less
    count: C,
    // Amount to move height up by to create the center point of the hill
    change_height: Range<u8>,
    // Horizontal range to place hills in by percent of horizontal size
    range_x: Range<f32>,
    // Vertical range to place hills in by percent of vertical size
    range_y: Range<f32>,
) {
    alter_point(
        grid,
        rng,
        count,
        change_height,
        range_x,
        range_y,
        Direction::Raise,
    )
}

fn pit<C: RangeBounds<f32>>(
    grid: &mut Grid,
    rng: &mut StdRng,
    count: C,
    change_height: Range<u8>,
    range_x: Range<f32>,
    range_y: Range<f32>,
) {
    alter_point(
        grid,
        rng,
        count,
        change_height,
        range_x,
        range_y,
        Direction::Lower,
    )
}

fn alter_point<C: RangeBounds<f32>>(
    grid: &mut Grid,
    rng: &mut StdRng,
    count: C,
    change_height: Range<u8>,
    range_x: Range<f32>,
    range_y: Range<f32>,
    direction: Direction,
) {
    let cells = &grid.voronoi.cells;

    let count = match (count.start_bound(), count.end_bound()) {
        (Bound::Included(s), Bound::Excluded(e)) => Uniform::new(s, e).sample(rng),
        (Bound::Included(s), Bound::Included(e)) => Uniform::new_inclusive(s, e).sample(rng),
        (Bound::Included(v), Bound::Unbounded)
        | (Bound::Excluded(v), Bound::Unbounded)
        | (Bound::Unbounded, Bound::Excluded(v))
        | (Bound::Unbounded, Bound::Included(v)) => *v,
        _ => unreachable!(),
    };
    let count = if rng.gen::<f32>() < count.fract() {
        count.trunc() as u32 + 1
    } else {
        count.trunc() as u32
    };

    let power = get_blob_power(grid.density);
    let height_uniform = Uniform::from(change_height);
    let change_uniform = Uniform::new(0.9, 1.1);
    let x_uniform = Uniform::new(
            range_x.start * grid.size.width as f32 / 100.0,
            range_x.end * grid.size.width as f32 / 100.0,
    );
    let y_uniform = Uniform::new(
        range_y.start * grid.size.height as f32 / 100.0,
        range_y.end * grid.size.height as f32 / 100.0,
    );
    for _ in 0..count {
        let h = height_uniform.sample(rng).min(WORLD_MAX);

        let mut start = 0;
        // Search for a seed cell that if changed by the maximum won't go too
        // high. Stop searching after 50 tries.
        for _ in 0..50 {
            let x = x_uniform.sample(rng);
            let y = y_uniform.sample(rng);
            start = grid.coords_to_cell_index(x, y).into();

            let good = match direction {
                Direction::Raise => grid.heights[start] + h <= ((WORLD_MAX as u32 * 9) / 10) as u8,
                // TODO: alter stuff below the ocean as well
                Direction::Lower => grid.heights[start] >= OCEAN_HEIGHT,
            };
            if good {
                break;
            }
        }

        let mut change = vec![0; cells.len()];
        change[start] = h;
        let mut queue = VecDeque::new();
        queue.push_back(start);
        while !queue.is_empty() {
            let q = queue.pop_front().unwrap();
            let h = match direction {
                Direction::Raise => (change[q] as f32).powf(power),
                // TODO: how does removing this extra randomization affect things?
                Direction::Lower => (change[q] as f32).powf(power) * change_uniform.sample(rng),
            };

            for adjacent in cells[&q.into()].adjacent_cells.iter() {
                if change[adjacent.as_usize()] != 0 {
                    continue;
                }
                change[adjacent.as_usize()] = (h * change_uniform.sample(rng)) as u8;
                if change[adjacent.as_usize()] > 1 {
                    queue.push_back(adjacent.as_usize())
                }
            }
        }

        for i in 0..grid.heights.len() {
            match direction {
                Direction::Raise => grid.heights[i] = grid.heights[i].saturating_add(change[i]),
                Direction::Lower => grid.heights[i] = grid.heights[i].saturating_sub(change[i]),
            }
        }
    }
}

fn range<C: RangeBounds<f32>>(
    grid: &mut Grid,
    rng: &mut StdRng,
    count: C,
    change_height: Range<u8>,
    range_x: Range<f32>,
    range_y: Range<f32>,
) {
    let count = match (count.start_bound(), count.end_bound()) {
        (Bound::Included(s), Bound::Excluded(e)) => Uniform::new(s, e).sample(rng),
        (Bound::Included(s), Bound::Included(e)) => Uniform::new_inclusive(s, e).sample(rng),
        (Bound::Included(v), Bound::Unbounded)
        | (Bound::Excluded(v), Bound::Unbounded)
        | (Bound::Unbounded, Bound::Excluded(v))
        | (Bound::Unbounded, Bound::Included(v)) => *v,
        _ => unreachable!(),
    };
    let count = if rng.gen::<f32>() < count.fract() {
        count.trunc() as u32 + 1
    } else {
        count.trunc() as u32
    };

    let height_uniform = Uniform::from(change_height);
    let start_x_uniform = Uniform::new(
        range_x.start * grid.size.width as f32 / 100.0,
        range_x.end * grid.size.width as f32 / 100.0,
    );
    let start_y_uniform = Uniform::new(
        range_y.start * grid.size.height as f32 / 100.0,
        range_y.end * grid.size.height as f32 / 100.0,
    );
    let end_x_uniform = Uniform::new(grid.size.width as f32 * 0.1, grid.size.width as f32 * 0.9);
    let end_y_uniform = Uniform::new(
        grid.size.height as f32 * 0.15,
        grid.size.height as f32 * 0.85,
    );
    for _ in 0..count {
        let mut h = height_uniform.sample(rng).min(WORLD_MAX) as f32;

        let start_x = start_x_uniform.sample(rng);
        let start_y = start_y_uniform.sample(rng);

        let mut end_x = 0.0;
        let mut end_y = 0.0;
        for _ in 0..50 {
            end_x = end_x_uniform.sample(rng);
            end_y = end_y_uniform.sample(rng);
            let dist = (end_x - start_x).abs() + (end_y - start_y).abs();
            if dist >= grid.size.width as f32 / 8.0 && dist <= grid.size.width as f32 / 3.0 {
                break;
            }
        }

        let mut used = vec![false; grid.voronoi.cells.len()];
        let range = get_range(
            grid,
            &mut used,
            rng,
            grid.coords_to_cell_index(start_x, start_y),
            grid.coords_to_cell_index(end_x, end_y),
            0.85,
        );

        let power = get_line_power(grid.density);
        let mut queue = range.clone();
        let mut ridge_depth = 0;
        while !queue.is_empty() {
            let mut new_queue = VecDeque::new();
            ridge_depth += 1;

            let change_uniform = Uniform::new(0.85, h * 0.3 + 0.85);
            for idx in &queue {
                grid.heights[idx.as_usize()] =
                    ((grid.heights[idx.as_usize()] as f32 + change_uniform.sample(rng)) as u8).min(WORLD_MAX);
            }

            h = h.powf(power) - 1.0;
            if h < 2.0 {
                break;
            }

            for idx in &queue {
                for &adjacent in &grid.voronoi.cells[idx].adjacent_cells {
                    if !used[adjacent.as_usize()] {
                        new_queue.push_back(adjacent);
                        used[adjacent.as_usize()] = true;
                    }
                }
            }

            queue = new_queue;
        }

        // generate prominences
        for (d, cell) in range.iter().enumerate() {
            let mut cur = cell;
            if d % 6 != 0 {
                continue;
            }
            for _ in 0..ridge_depth {
                // Find the downhill cell.
                let min = grid
                    .voronoi
                    .cells[cur]
                    .adjacent_cells
                    .iter()
                    .min_by(|a, b| grid.heights[a.as_usize()].cmp(&grid.heights[b.as_usize()]))
                    .unwrap();
                grid.heights[min.as_usize()] =
                    ((grid.heights[cur.as_usize()] as u32 * 2 + grid.heights[min.as_usize()] as u32) / 3) as u8;
                cur = min;
            }
        }
    }
}

fn trough<C: RangeBounds<f32>>(
    grid: &mut Grid,
    rng: &mut StdRng,
    count: C,
    change_height: Range<u8>,
    range_x: Range<f32>,
    range_y: Range<f32>,
) {
    let count = match (count.start_bound(), count.end_bound()) {
        (Bound::Included(s), Bound::Excluded(e)) => Uniform::new(s, e).sample(rng),
        (Bound::Included(s), Bound::Included(e)) => Uniform::new_inclusive(s, e).sample(rng),
        (Bound::Included(v), Bound::Unbounded)
        | (Bound::Excluded(v), Bound::Unbounded)
        | (Bound::Unbounded, Bound::Excluded(v))
        | (Bound::Unbounded, Bound::Included(v)) => *v,
        _ => unreachable!(),
    };
    let count = if rng.gen::<f32>() < count.fract() {
        count.trunc() as u32 + 1
    } else {
        count.trunc() as u32
    };

    let height_uniform = Uniform::from(change_height);
    let start_x_uniform = Uniform::new(
        range_x.start * grid.size.width as f32 / 100.0,
        range_x.end * grid.size.width as f32 / 100.0,
    );
    let start_y_uniform = Uniform::new(
        range_y.start * grid.size.height as f32 / 100.0,
        range_y.end * grid.size.height as f32 / 100.0,
    );
    let end_x_uniform = Uniform::new(grid.size.width as f32 * 0.1, grid.size.width as f32 * 0.9);
    let end_y_uniform = Uniform::new(
        grid.size.height as f32 * 0.15,
        grid.size.height as f32 * 0.85,
    );
    for _ in 0..count {
        let mut h = height_uniform.sample(rng).min(WORLD_MAX) as f32;

        let mut start_x = 0.0;
        let mut start_y = 0.0;
        for _ in 0..50 {
            start_x = start_x_uniform.sample(rng);
            start_y = start_y_uniform.sample(rng);
            let start = grid.coords_to_cell_index(start_x, start_y);
            if grid.heights[start.as_usize()] < OCEAN_HEIGHT {
                break;
            }
        }

        let mut end_x = 0.0;
        let mut end_y = 0.0;
        for _ in 0..50 {
            end_x = end_x_uniform.sample(rng);
            end_y = end_y_uniform.sample(rng);
            let dist = (end_x - start_x).abs() + (end_y - start_y).abs();
            if dist >= grid.size.width as f32 / 8.0 && dist <= grid.size.width as f32 / 2.0 {
                break;
            }
        }

        let mut used = vec![false; grid.voronoi.cells.len()];
        let range = get_range(
            grid,
            &mut used,
            rng,
            grid.coords_to_cell_index(start_x, start_y),
            grid.coords_to_cell_index(end_x, end_y),
            0.8,
        );

        let power = get_line_power(grid.density);
        let mut queue = range.clone();
        let mut ridge_depth = 0;
        while !queue.is_empty() {
            let mut new_queue = VecDeque::new();
            ridge_depth += 1;

            let change_uniform = Uniform::new(0.85, h * 0.3 + 0.85);
            for idx in &queue {
                grid.heights[idx.as_usize()] =
                    ((grid.heights[idx.as_usize()] as f32 - change_uniform.sample(rng)) as u8).min(WORLD_MAX);
            }

            h = h.powf(power) - 1.0;
            if h < 2.0 {
                break;
            }

            for idx in &queue {
                for &adjacent in &grid.voronoi.cells[idx].adjacent_cells {
                    if !used[adjacent.as_usize()] {
                        new_queue.push_back(adjacent);
                        used[adjacent.as_usize()] = true;
                    }
                }
            }

            queue = new_queue;
        }

        // generate prominences
        for (d, cell) in range.iter().enumerate() {
            let mut cur = cell;
            if d % 6 != 0 {
                continue;
            }
            for _ in 0..ridge_depth {
                // Find the downhill cell.
                let min = grid
                    .voronoi
                    .cells[cur]
                    .adjacent_cells
                    .iter()
                    .min_by(|a, b| grid.heights[a.as_usize()].cmp(&grid.heights[b.as_usize()]))
                    .unwrap();
                grid.heights[min.as_usize()] =
                    ((grid.heights[cur.as_usize()] as u32 * 2 + grid.heights[min.as_usize()] as u32) / 3) as u8;
                cur = min;
            }
        }
    }
}

// TODO: `modify` function

// TODO: `strait` function

fn smooth(grid: &mut Grid, rng: &mut StdRng, force: u32) {
    let heights = &mut grid.heights;
    let cells = &grid.voronoi.cells;

    for i in 0..heights.len() {
        let h = heights[i];
        let mut a = vec![h as u32];
        for cell_index in cells[&i.into()].adjacent_cells.iter() {
            a.push(heights[cell_index.as_usize()] as u32);
        }
        let height = h as u32 * (force - 1) + a.iter().sum::<u32>() / a.len() as u32 / force;
        heights[i] = height.min(WORLD_MAX as u32) as u8
    }
}

fn get_blob_power(density: NonZeroU32) -> f32 {
    match density.get() {
        1 => 0.98,
        2 => 0.985,
        3 => 0.987,
        4 => 0.9892,
        5 => 0.9911,
        6 => 0.9921,
        7 => 0.9934,
        8 => 0.9942,
        9 => 0.9946,
        10 => 0.995,
        _ => unreachable!(),
    }
}

fn get_line_power(density: NonZeroU32) -> f32 {
    match density.get() {
        1 => 0.81,
        2 => 0.82,
        3 => 0.83,
        4 => 0.84,
        5 => 0.855,
        6 => 0.87,
        7 => 0.885,
        8 => 0.91,
        9 => 0.92,
        10 => 0.93,
        _ => unreachable!(),
    }
}

fn get_range(
    grid: &Grid,
    used: &mut [bool],
    rng: &mut StdRng,
    cur: PointIndex,
    end: PointIndex,
    cmp_value: f32,
) -> VecDeque<PointIndex> {
    let cells = &grid.voronoi.cells;
    let points = &grid.points;
    let mut cur = cur;

    let mut range = VecDeque::new();
    range.push_back(cur);
    used[cur.as_usize()] = true;

    while cur != end {
        let mut min = std::f32::INFINITY;
        for &cell in &cells[&cur].adjacent_cells {
            if used[cell.as_usize()] {
                continue;;
            }
            let mut diff =
                (points[end].x - points[cell].x).powi(2) + (points[end].y - points[cell].y).powi(2);
            if rng.gen::<f32>() > cmp_value {
                diff = diff / 2.0;
            }
            if diff < min {
                min = diff;
                cur = cell;
            }
        }
        if min.is_infinite() {
            break;
        }
        range.push_back(cur);
        used[cur.as_usize()] = true;
    }

    range
}
