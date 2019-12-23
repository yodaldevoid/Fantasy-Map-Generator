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
    Archipelago, // TODO: needs work
    Atoll, // TODO: almost right, not lowering enough?
    Continents, // TODO: almost right, continents not getting separated
    HighIsland, // TODO: crashes
    Isthmus, // TODO: too flat, not enough islandy bits
    LowIsland, // TODO: maybe too high, revisit after fixing others
    Mediterranean, // TODO: too high, sea not always created
    Pangaea, // TODO: Too samey, middle seems to be clipping
    Peninsula, // TODO: Far too high, sea never created
    Volcano, // TODO: seems good, maybe too high as middle is always clipping
}

// Not really a word. Derived from "Cartesian coordinate system".
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Cartesianality {
    Horizontal,
    Vertical,
}

pub enum HeightRange {
    All,
    Land,
    Ocean,
    Range(u8, u8),
}

impl HeightRange {
    fn max(&self) -> u8 {
        match self {
            HeightRange::All => WORLD_MAX,
            HeightRange::Land => WORLD_MAX,
            HeightRange::Ocean => OCEAN_HEIGHT.saturating_sub(1),
            HeightRange::Range(_, end) => *end,
        }
    }

    fn min(&self) -> u8 {
        match self {
            HeightRange::All => 0,
            HeightRange::Land => OCEAN_HEIGHT,
            HeightRange::Ocean => 0,
            HeightRange::Range(start, _) => *start,
        }
    }
}

impl<R: RangeBounds<u8>> From<R> for HeightRange {
    fn from(range: R) -> Self {
        match (range.start_bound(), range.end_bound()) {
            (Bound::Included(&s), Bound::Excluded(&e)) =>
                HeightRange::Range(s, e.saturating_sub(1)),
            (Bound::Included(&s), Bound::Included(&e)) => HeightRange::Range(s, e),
            (Bound::Included(&v), Bound::Unbounded) => HeightRange::Range(v, WORLD_MAX),
            (Bound::Excluded(&v), Bound::Unbounded) =>
                HeightRange::Range((v + 1).min(WORLD_MAX), WORLD_MAX),
            (Bound::Unbounded, Bound::Excluded(&v)) => HeightRange::Range(0, v.saturating_sub(1)),
            (Bound::Unbounded, Bound::Included(&v)) => HeightRange::Range(0, v),
            (Bound::Unbounded, Bound::Unbounded) => HeightRange::All,

            // I'm not sure this will ever be hit...
            (Bound::Excluded(&s), Bound::Excluded(&e)) =>
                HeightRange::Range((s + 1).min(WORLD_MAX), e.saturating_sub(1)),
            (Bound::Excluded(&s), Bound::Included(&e)) =>
                HeightRange::Range((s + 1).min(WORLD_MAX), e),
        }
    }
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
    add(grid, rng, HeightRange::All, 11);
    range(grid, rng, 2.0..3.0, 40..60, 20.0..80.0, 20.0..80.0);
    hill(grid, rng, 5.0.., 15..20, 10.0..90.0, 30.0..70.0);
    hill(grid, rng, 2.0.., 10..15, 10.0..30.0, 20.0..80.0);
    hill(grid, rng, 2.0.., 10..15, 60.0..90.0, 20.0..80.0);
    smooth(grid, rng, 3);
    trough(grid, rng, 10.0.., 20..30, 5.0..95.0, 5.0..95.0);
    strait(grid, rng, 2.0.., Cartesianality::Vertical);
    strait(grid, rng, 2.0.., Cartesianality::Horizontal);
}

fn generate_atoll(grid: &mut Grid, rng: &mut StdRng) {
    hill(grid, rng, 1.0.., 75..80, 50.0..60.0, 45.0..55.0);
    hill(grid, rng, 1.5.., 30..50, 25.0..75.0, 30.0..70.0);
    hill(grid, rng, 0.5.., 30..50, 25.0..35.0, 30.0..70.0);
    smooth(grid, rng, 1);
    multiply(grid, rng, (25..100).into(), 0.2);
    hill(grid, rng, 0.5.., 10..20, 50.0..55.0, 48.0..52.0);
}

fn generate_continents(grid: &mut Grid, rng: &mut StdRng) {
    hill(grid, rng, 1.0.., 80..85, 75.0..80.0, 40.0..60.0);
    hill(grid, rng, 1.0.., 80..85, 20.0..25.0, 40.0..60.0);
    multiply(grid, rng, (20..100).into(), 0.22);
    hill(grid, rng, 5.0..6.0, 15..20, 25.0..75.0, 20.0..82.0);
    range(grid, rng, 0.8.., 30..60, 5.0..15.0, 20.0..45.0);
    range(grid, rng, 0.8.., 30..60, 5.0..15.0, 55.0..80.0);
    range(grid, rng, 0.0..3.0, 30..60, 80.0..90.0, 20.0..80.0);
    trough(grid, rng, 3.0..4.0, 15..20, 15.0..85.0, 20.0..80.0);
    strait(grid, rng, 2.0.., Cartesianality::Vertical);
    smooth(grid, rng, 2);
    trough(grid, rng, 1.0..2.0, 5..10, 45.0..55.0, 45.0..55.0);
    pit(grid, rng, 3.0..4.0, 10..15, 15.0..85.0, 20.0..80.0);
    hill(grid, rng, 1.0.., 5..10, 40.0..60.0, 40.0..60.0);
}

fn generate_high_island(grid: &mut Grid, rng: &mut StdRng) {
    hill(grid, rng, 1.0.., 90..100, 65.0..75.0, 47.0..53.0);
    add(grid, rng, HeightRange::All.into(), 5);
    hill(grid, rng, 6.0.., 20..23, 25.0..55.0, 45.0..55.0);
    range(grid, rng, 1.0.., 40..50, 45.0..55.0, 45.0..55.0);
    smooth(grid, rng, 2);
    trough(grid, rng, 2.0..3.0, 20..30, 20.0..30.0, 20.0..30.0);
    trough(grid, rng, 2.0..3.0, 20..30, 60.0..80.0, 70.0..80.0);
    hill(grid, rng, 1.0.., 10..15, 60.0..60.0, 50.0..50.0);
    hill(grid, rng, 1.5.., 13..16, 15.0..20.0, 20.0..75.0);
    multiply(grid, rng, (20..100).into(), 0.8);
    range(grid, rng, 1.5.., 30..40, 15.0..85.0, 30.0..40.0);
    range(grid, rng, 1.5.., 30..40, 15.0..85.0, 60.0..70.0);
    pit(grid, rng, 2.0..3.0, 10..15, 15.0..85.0, 20.0..80.0);
}

fn generate_isthmus(grid: &mut Grid, rng: &mut StdRng) {
    hill(grid, rng, 5.0..10.0, 15..30, 0.0..30.0, 0.0..20.0);
    hill(grid, rng, 5.0..10.0, 15..30, 10.0..50.0, 20.0..40.0);
    hill(grid, rng, 5.0..10.0, 15..30, 30.0..70.0, 40.0..60.0);
    hill(grid, rng, 5.0..10.0, 15..30, 50.0..90.0, 60.0..80.0);
    hill(grid, rng, 5.0..10.0, 15..30, 70.0..100.0, 80.0..100.00);
    smooth(grid, rng, 2);
    trough(grid, rng, 4.0..8.0, 15..30, 0.0..30.0, 0.0..20.0);
    trough(grid, rng, 4.0..8.0, 15..30, 10.0..50.0, 20.0..40.0);
    trough(grid, rng, 4.0..8.0, 15..30, 30.0..70.0, 40.0..60.0);
    trough(grid, rng, 4.0..8.0, 15..30, 50.0..90.0, 60.0..80.0);
    trough(grid, rng, 4.0..8.0, 15..30, 70.0..100.0, 80.0..100.00);
}

fn generate_low_island(grid: &mut Grid, rng: &mut StdRng) {
    hill(grid, rng, 1.0.., 90..99, 60.0..80.0, 45.0..55.0);
    hill(grid, rng, 4.0..5.0, 25..35, 20.0..65.0, 40.0..60.0);
    range(grid, rng, 1.0.., 40..50, 45.0..55.0, 45.0..55.0);
    smooth(grid, rng, 3);
    trough(grid, rng, 1.5.., 20..30, 15.0..85.0, 20.0..30.0);
    trough(grid, rng, 1.5.., 20..30, 15.0..85.0, 70.0..80.0);
    hill(grid, rng, 1.5.., 10..15, 5.0..15.0, 20.0..80.0);
    hill(grid, rng, 1.0.., 10..15, 85.0..95.0, 70.0..80.0);
    pit(grid, rng, 3.0..5.0, 10..15, 15.0..85.0, 20.0..80.0);
    multiply(grid, rng, (20..100).into(), 0.4);
}

fn generate_mediterranean(grid: &mut Grid, rng: &mut StdRng) {
    range(grid, rng, 3.0..4.0, 30..50, 0.0..100.0, 0.0..10.0);
    range(grid, rng, 3.0..4.0, 30..50, 0.0..100.0, 90.0..100.0);
    hill(grid, rng, 5.0..6.0, 30..70, 0.0..100.0, 0.0..5.0);
    hill(grid, rng, 5.0..6.0, 30..70, 0.0..100.0, 95.0..100.0);
    smooth(grid, rng, 1);
    hill(grid, rng, 2.0..3.0, 30..70, 0.0..5.0, 20.0..80.0);
    hill(grid, rng, 2.0..3.0, 30..70, 95.0..100.0, 20.0..80.0);
    multiply(grid, rng, HeightRange::Land.into(), 0.8);
    trough(grid, rng, 3.0..5.0, 40..50, 0.0..100.0, 0.0..10.0);
    trough(grid, rng, 3.0..5.0, 40..50, 0.0..100.0, 90.0..100.0);
}

fn generate_pangaea(grid: &mut Grid, rng: &mut StdRng) {
    hill(grid, rng, 1.0..2.0, 25..40, 15.0..50.0, 0.0..10.0);
    hill(grid, rng, 1.0..2.0, 5..40, 50.0..85.0, 0.0..10.0);
    hill(grid, rng, 1.0..2.0, 25..40, 50.0..85.0, 90.0..100.0);
    hill(grid, rng, 1.0..2.0, 5..40, 15.0..50.0, 90.0..100.0);
    hill(grid, rng, 8.0..12.0, 20..40, 20.0..80.0, 48.0..52.0);
    smooth(grid, rng, 2);
    multiply(grid, rng, HeightRange::Land.into(), 0.7);
    trough(grid, rng, 3.0..4.0, 25..35, 5.0..95.0, 10.0..20.0);
    trough(grid, rng, 3.0..4.0, 25..35, 5.0..95.0, 80.0..90.0);
    range(grid, rng, 5.0..6.0, 30..40, 10.0..90.0, 35.0..65.0);
}

fn generate_peninsula(grid: &mut Grid, rng: &mut StdRng) {
    range(grid, rng, 2.0..3.0, 20..35, 40.0..50.0, 0.0..15.0);
    add(grid, rng, HeightRange::All.into(), 5);
    hill(grid, rng, 1.0.., 90..100, 10.0..90.0, 0.0..5.0);
    add(grid, rng, HeightRange::All.into(), 13);
    hill(grid, rng, 3.0..4.0, 3..5, 5.0..95.0, 80.0..100.0);
    hill(grid, rng, 1.0..2.0, 3..5, 5.0..95.0, 40.0..60.0);
    trough(grid, rng, 5.0..6.0, 10..25, 5.0..95.0, 5.0..95.0);
    smooth(grid, rng, 3);
}

fn generate_volcano(grid: &mut Grid, rng: &mut StdRng) {
    hill(grid, rng, 1.0.., 90..100, 44.0..56.0, 40.0..60.0);
    multiply(grid, rng, (50..100).into(), 8.0);
    range(grid, rng, 1.5.., 30..55, 45.0..55.0, 40.0..60.0);
    smooth(grid, rng, 2);
    hill(grid, rng, 1.5.., 25..35, 25.0..30.0, 20.0..75.0);
    hill(grid, rng, 1.0.., 25..35, 75.0..80.0, 25.0..75.0);
    hill(grid, rng, 0.5.., 20..25, 10.0..15.0, 20.0..25.0);
}

// Feature generation functions
// TODO: refactor paired generators
// TODO: check that ranges are from low to high

enum ModifyDirection {
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
        ModifyDirection::Raise,
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
        ModifyDirection::Lower,
    )
}

fn alter_point<C: RangeBounds<f32>>(
    grid: &mut Grid,
    rng: &mut StdRng,
    count: C,
    change_height: Range<u8>,
    range_x: Range<f32>,
    range_y: Range<f32>,
    direction: ModifyDirection,
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
                ModifyDirection::Raise => grid.heights[start] + h <= ((WORLD_MAX as u32 * 9) / 10) as u8,
                // TODO: alter stuff below the ocean as well
                ModifyDirection::Lower => grid.heights[start] >= OCEAN_HEIGHT,
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
                ModifyDirection::Raise => (change[q] as f32).powf(power),
                // TODO: how does removing this extra randomization affect things?
                ModifyDirection::Lower => (change[q] as f32).powf(power) * change_uniform.sample(rng),
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
                ModifyDirection::Raise => grid.heights[i] = grid.heights[i].saturating_add(change[i]),
                ModifyDirection::Lower => grid.heights[i] = grid.heights[i].saturating_sub(change[i]),
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

fn add(
    grid: &mut Grid,
    rng: &mut StdRng,
    range: HeightRange,
    value: i16,
) {
    let min = range.min();
    let max = range.max();

    for h in &mut grid.heights {
        if *h >= min && *h <= max {
            *h = if min == OCEAN_HEIGHT {
                // TODO: decide if I want to keep this.
                // Maybe add flag to saturate within the range?
                (*h as i16 + value).min(WORLD_MAX as i16).max(OCEAN_HEIGHT as i16) as u8
            } else {
                (*h as i16 + value).min(WORLD_MAX as i16).max(0) as u8
            };
        }
    }
}

fn multiply(
    grid: &mut Grid,
    rng: &mut StdRng,
    range: HeightRange,
    value: f32,
) {
    let min = range.min();
    let max = range.max();

    for h in &mut grid.heights {
        if *h >= min && *h <= max {
            *h = if min == OCEAN_HEIGHT {
                // TODO: decide if I want to keep this.
                // Maybe add flag to saturate within the range?
                ((*h - OCEAN_HEIGHT) as f32) * value + OCEAN_HEIGHT as f32
            } else {
                *h as f32 * value
            }.min(WORLD_MAX as f32)
                .max(0.0) as u8
        }
    }
}

fn power(
    grid: &mut Grid,
    rng: &mut StdRng,
    range: HeightRange,
    value: f32,
) {
    unimplemented!()
}

fn strait<W: RangeBounds<f32>>(
    grid: &mut Grid,
    rng: &mut StdRng,
    width: W,
    direction: Cartesianality,
) {
    let mut width = match (width.start_bound(), width.end_bound()) {
        (Bound::Included(s), Bound::Excluded(e)) => Uniform::new(s, e).sample(rng),
        (Bound::Included(s), Bound::Included(e)) => Uniform::new_inclusive(s, e).sample(rng),
        (Bound::Included(v), Bound::Unbounded)
        | (Bound::Excluded(v), Bound::Unbounded)
        | (Bound::Unbounded, Bound::Excluded(v))
        | (Bound::Unbounded, Bound::Included(v)) => *v,
        _ => unreachable!(),
    }.min(grid.cells_x as f32 / 3.0);
    if width < 1.0 && rng.gen::<f32>() < width {
        return;
    }

    let mut used = vec![false; grid.voronoi.cells.len()];
    let (start_x, start_y, end_x, end_y) = if Cartesianality::Vertical == direction {
        let start_x = Uniform::new(
            grid.size.width as f32 * 0.3,
            grid.size.width as f32 * 0.7,
        ).sample(rng)
            .floor();

        let end_x = Uniform::new(
            (grid.size.width as f32 - start_x) - (grid.size.width as f32 * 0.1),
            (grid.size.width as f32 - start_x) + (grid.size.width as f32 * 0.1),
        ).sample(rng)
            .floor();

        (start_x, 5.0, end_x, (grid.size.height - 5) as f32)
    } else {
        let start_y = Uniform::new(
            grid.size.height as f32 * 0.3,
            grid.size.height as f32 * 0.7,
        ).sample(rng)
            .floor();

        let end_y = Uniform::new(
            (grid.size.height as f32 - start_y) - (grid.size.height as f32 * 0.1),
            (grid.size.height as f32 - start_y) + (grid.size.height as f32 * 0.1),
        ).sample(rng)
            .floor();

        (5.0, start_y, (grid.size.width - 5) as f32, end_y)
    };

    let start = grid.coords_to_cell_index(start_x, start_y);
    let end = grid.coords_to_cell_index(end_x, end_y);
    let mut range = get_range(grid, rng, start, end);

    let mut query = Vec::new();
    let step = 0.1 / width;
    while width > 0.0 {
        let exp = 0.9 - step * width;
        for r in range.iter() {
            for a in grid.voronoi.cells[r].adjacent_cells.iter() {
                if used[a.as_usize()] {
                    continue;
                }
                used[a.as_usize()] = true;
                query.push(*a);
                grid.heights[a.as_usize()] = (grid.heights[a.as_usize()] as f32).powf(exp) as u8;
                if grid.heights[a.as_usize()] > WORLD_MAX {
                    grid.heights[a.as_usize()] = 5;
                }
            }
        }

        range.clear();
        // Technically supposed to be `range.copy_from_slice(query.as_slice())`
        range.append(&mut query);

        width -= 1.0;
    }

    fn get_range(
        grid: &Grid,
        rng: &mut StdRng,
        cur: PointIndex,
        end: PointIndex,
    ) -> Vec<PointIndex> {
        let cells = &grid.voronoi.cells;
        let points = &grid.points;
        let mut cur = cur;

        let mut range = Vec::new();
        range.push(cur);

        while cur != end {
            let mut min = std::f32::INFINITY;
            for &cell in &cells[&cur].adjacent_cells {
                let mut diff = (points[end].x - points[cell].x).powi(2)
                    + (points[end].y - points[cell].y).powi(2);
                if rng.gen::<f32>() > 0.8 {
                    diff = diff / 2.0;
                }
                if diff < min {
                    min = diff;
                    cur = cell;
                }
            }
            range.push(cur);
        }

        range
    }
}

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
