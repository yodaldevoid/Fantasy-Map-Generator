use wasm_bindgen::JsCast;
use web_sys::console;
use web_sys::{Element, Event};

use util::{ElementExt, NodeListExt};

fn clear_legend(legend: &Element) {
    for node in legend.child_nodes().iter() {
        if let Some(parent) = node.parent_node() {
            parent.remove_child(&node).ok();
        }
    }
    legend.remove_attribute("data").ok();
}

// TODO: remove notes
fn undraw(viewbox: &Element, defs: &Element, fogging: &Element) {
    unimplemented!()
}

fn create_svg() -> Result<(), JsValue> {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    let svg = document.query_selector("#map")?.expect("Did not find map!");
    let deftemp = svg.query_selector("#deftemp")?.expect("Did not find deftemp!");
    let viewbox = svg.query_selector("#viewbox")?.expect("Did not find viewbox!");
    let scale_bar = svg.query_selector("#scaleBar")?.expect("Did not find scaleBar!");

    let legend = svg.append("g")?.attr("id", "legend")?;
    let ocean = viewbox.append("g")?.attr("id", "ocean")?;
    let ocean_layers = ocean.append("g")?.attr("id", "oceanLayers")?;
    let ocean_pattern = ocean.append("g")?.attr("id", "oceanPattern")?;
    let lakes = viewbox.append("g")?.attr("id", "lakes")?;
    let landmass = viewbox.append("g")?.attr("id", "landmass")?;
    let texture = viewbox.append("g")?.attr("id", "texture")?;
    let terrs = viewbox.append("g")?.attr("id", "terrs")?;
    let biomes = viewbox.append("g")?.attr("id", "biomes")?;
    let cells = viewbox.append("g")?.attr("id", "cells")?;
    let grid_overlay = viewbox.append("g")?.attr("id", "gridOverlay")?;
    let coordinates = viewbox.append("g")?.attr("id", "coordinates")?;
    let compass = viewbox.append("g")?.attr("id", "compass")?;
    let rivers = viewbox.append("g")?.attr("id", "rivers")?;
    let terrain = viewbox.append("g")?.attr("id", "terrain")?;
    let relig = viewbox.append("g")?.attr("id", "relig")?;
    let cults = viewbox.append("g")?.attr("id", "cults")?;
    let regions = viewbox.append("g")?.attr("id", "regions")?;
    let states_body = regions.append("g")?.attr("id", "statesBody")?;
    let states_halo = regions.append("g")?.attr("id", "statesHalo")?;
    let provs = viewbox.append("g")?.attr("id", "provs")?;
    let zones = viewbox.append("g")?.attr("id", "zones")?.attr("display", "none")?;
    let borders = viewbox.append("g")?.attr("id", "borders")?;
    let state_borders = borders.append("g")?.attr("id", "stateBorders")?;
    let province_borders = borders.append("g")?.attr("id", "provinceBorders")?;
    let routes = viewbox.append("g")?.attr("id", "routes")?;
    let roads = routes.append("g")?.attr("id", "roads")?;
    let trails = routes.append("g")?.attr("id", "trails")?;
    let searoutes = routes.append("g")?.attr("id", "searoutes")?;
    let temperature = viewbox.append("g")?.attr("id", "temperature")?;
    let coastline = viewbox.append("g")?.attr("id", "coastline")?;
    let prec = viewbox.append("g")?.attr("id", "prec")?.attr("display", "none")?;
    let population = viewbox.append("g")?.attr("id", "population")?;
    let labels = viewbox.append("g")?.attr("id", "labels")?;
    let icons = viewbox.append("g")?.attr("id", "icons")?;
    let burg_icons = icons.append("g")?.attr("id", "burgIcons")?;
    let anchors = icons.append("g")?.attr("id", "anchors")?;
    let markers = viewbox.append("g")?.attr("id", "markers")?.attr("display", "none")?;
    let fogging = viewbox.append("g")?.attr("id", "fogging-cont")?.attr("mask", "url(#fog)")?
      .append("g")?.attr("id", "fogging")?.attr("display", "none")?;
    let ruler = viewbox.append("g")?.attr("id", "ruler")?.attr("display", "none")?;
    let debug = viewbox.append("g")?.attr("id", "debug")?;

    let freshwater = lakes.append("g")?.attr("id", "freshwater")?;
    let salt = lakes.append("g")?.attr("id", "salt")?;

    labels.append("g")?.attr("id", "states")?;
    labels.append("g")?.attr("id", "addedLabels")?;

    let burg_labels = labels.append("g")?.attr("id", "burgLabels")?;
    burg_icons.append("g")?.attr("id", "cities")?;
    burg_labels.append("g")?.attr("id", "cities")?;
    anchors.append("g")?.attr("id", "cities")?;

    burg_icons.append("g")?.attr("id", "towns")?;
    burg_labels.append("g")?.attr("id", "towns")?;
    anchors.append("g")?.attr("id", "towns")?;

    // population groups
    population.append("g")?.attr("id", "rural")?;
    population.append("g")?.attr("id", "urban")?;

    // assign events separately as not a viewbox child
    // TODO: use tip instead of console
    scale_bar.on("mousemove", Some(|_| console::log_1(&JsValue::from_str("Click to open Units Editor"))))?;
    legend.on("mousemove", Some(|_| console::log_1(&JsValue::from_str("Drag to change the position. Click to hide the legend"))))?;
    legend.on("click", Some(|e: Event| clear_legend(e.unchecked_ref())))?;

    // TODO: biomes
    // TODO: setup zoom
    // TODO: colorscheme

    Ok(())
}
