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
