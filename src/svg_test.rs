use svg::Document;
use svg::node::element::Path;

pub fn remove_loading() {}

pub fn undraw_all() {}

pub fn unfog() {}

pub fn __draw_cells(path: String) {
    let path = Path::new()
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 0.1)
        .set("d", path);
    let doc = Document::new().set("background-color", "white").add(path);
    svg::save("cells.svg", &doc).expect("SVG failed to save");
}

pub fn clear_cells() {}

// All inputs are arrays of strings
pub fn _draw_coastline(
    land_mask_paths: &[String],
    water_mask_paths: &[String],
    coastline_paths: &[String],
    coastline_ids: &[String],
    lake_groups: &[String],
    lake_paths: &[String],
    lake_ids: &[String],
) {
    let mut doc = Document::new().set("background-color", "white");
    for path in land_mask_paths {
        let path = Path::new()
            //.set("fill", "white")
            .set("stroke-width", 0.1)
            .set("stroke", "green")
            .set("d", path.as_str());
        doc = doc.add(path);
    }
    for path in water_mask_paths {
        let path = Path::new()
            //.set("fill", "black")
            .set("stroke-width", 0.1)
            .set("stroke", "blue")
            .set("d", path.as_str());
        doc = doc.add(path);
    }
    for i in 0..coastline_paths.len() {
        let path = Path::new()
            .set("stroke", "black")
            .set("stroke-width", 0.1)
            .set("id", coastline_ids[i].as_str())
            .set("d", coastline_paths[i].as_str());
        doc = doc.add(path);
    }
    for i in 0..lake_paths.len() {
        // TODO: lake groups
        let path = Path::new()
            .set("stroke", "cyan")
            .set("stroke-width", 0.1)
            .set("id", lake_ids[i].as_str())
            .set("d", lake_paths[i].as_str());
        doc = doc.add(path);
    }
    svg::save("coastline.svg", &doc).expect("SVG failed to save");
}

pub fn _draw_heightmap(
    height_paths: &[String],
    height_colors: &[String],
    height_values: &[u8],
) {
    let mut doc = Document::new().set("background-color", "white");
    for i in 0..height_paths.len() {
        let path = Path::new()
            .set("d", height_paths[i].as_str())
            .set("fill", height_colors[i].as_str())
            .set("data-height", height_values[i]);
        doc = doc.add(path);
    }
    svg::save("heightmap.svg", &doc).expect("SVG failed to save");
}

pub fn clear_heightmap() {}
