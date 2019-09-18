import {default as mapgen_init} from "./pkg/mapgen.js"

"use strict";

// TODO: version and document title

// TODO: welcome message

// append svg layers (in default order)
let svg = d3.select("#map");
let defs = svg.select("#deftemp");
let viewbox = svg.select("#viewbox");
let scaleBar = svg.select("#scaleBar");
let legend = svg.append("g").attr("id", "legend");
let ocean = viewbox.append("g").attr("id", "ocean");
let oceanLayers = ocean.append("g").attr("id", "oceanLayers");
let oceanPattern = ocean.append("g").attr("id", "oceanPattern");
let lakes = viewbox.append("g").attr("id", "lakes");
let landmass = viewbox.append("g").attr("id", "landmass");
let texture = viewbox.append("g").attr("id", "texture");
let terrs = viewbox.append("g").attr("id", "terrs");
let biomes = viewbox.append("g").attr("id", "biomes");
let cells = viewbox.append("g").attr("id", "cells");
let gridOverlay = viewbox.append("g").attr("id", "gridOverlay");
let coordinates = viewbox.append("g").attr("id", "coordinates");
let compass = viewbox.append("g").attr("id", "compass");
let rivers = viewbox.append("g").attr("id", "rivers");
let terrain = viewbox.append("g").attr("id", "terrain");
let relig = viewbox.append("g").attr("id", "relig");
let cults = viewbox.append("g").attr("id", "cults");
let regions = viewbox.append("g").attr("id", "regions");
let statesBody = regions.append("g").attr("id", "statesBody");
let statesHalo = regions.append("g").attr("id", "statesHalo");
let provs = viewbox.append("g").attr("id", "provs");
let zones = viewbox.append("g").attr("id", "zones").attr("display", "none");
let borders = viewbox.append("g").attr("id", "borders");
let stateBorders = borders.append("g").attr("id", "stateBorders");
let provinceBorders = borders.append("g").attr("id", "provinceBorders");
let routes = viewbox.append("g").attr("id", "routes");
let roads = routes.append("g").attr("id", "roads");
let trails = routes.append("g").attr("id", "trails");
let searoutes = routes.append("g").attr("id", "searoutes");
let temperature = viewbox.append("g").attr("id", "temperature");
let coastline = viewbox.append("g").attr("id", "coastline");
let prec = viewbox.append("g").attr("id", "prec").attr("display", "none");
let population = viewbox.append("g").attr("id", "population");
let labels = viewbox.append("g").attr("id", "labels");
let icons = viewbox.append("g").attr("id", "icons");
let burgIcons = icons.append("g").attr("id", "burgIcons");
let anchors = icons.append("g").attr("id", "anchors");
let markers = viewbox.append("g").attr("id", "markers").attr("display", "none");
let fogging = viewbox.append("g").attr("id", "fogging-cont").attr("mask", "url(#fog)")
    .append("g").attr("id", "fogging").attr("display", "none");
let ruler = viewbox.append("g").attr("id", "ruler").attr("display", "none");
let debug = viewbox.append("g").attr("id", "debug");

let freshwater = lakes.append("g").attr("id", "freshwater");
let salt = lakes.append("g").attr("id", "salt");

labels.append("g").attr("id", "states");
labels.append("g").attr("id", "addedLabels");

let burgLabels = labels.append("g").attr("id", "burgLabels");
burgIcons.append("g").attr("id", "cities");
burgLabels.append("g").attr("id", "cities");
anchors.append("g").attr("id", "cities");

burgIcons.append("g").attr("id", "towns");
burgLabels.append("g").attr("id", "towns");
anchors.append("g").attr("id", "towns");

// population groups
population.append("g").attr("id", "rural");
population.append("g").attr("id", "urban");

// fogging
fogging.append("rect").attr("x", 0).attr("y", 0).attr("width", "100%").attr("height", "100%");

// assign events separately as not a viewbox child
scaleBar.on("mousemove", () => tip("Click to open Units Editor"));
legend.on("mousemove", () => tip("Drag to change the position. Click to hide the legend")).on("click", () => clearLegend());

// d3 zoom behavior
let scale = 1, viewX = 0, viewY = 0;
const zoom = d3.zoom().scaleExtent([1, 20]).on("zoom", zoomed);

// TODO: stored options

// voronoi graph extention, should be stable for each map
let graphWidth = 1000; //+mapWidthInput.value;
let graphHeight = 1000; //+mapHeightInput.value;

let svgWidth = graphWidth, svgHeight = graphHeight; // svg canvas resolution, can vary for each map
landmass.append("rect").attr("x", 0).attr("y", 0).attr("width", graphWidth).attr("height", graphHeight);
oceanPattern.append("rect").attr("fill", "url(#oceanic)").attr("x", 0).attr("y", 0).attr("width", graphWidth).attr("height", graphHeight);
oceanLayers.append("rect").attr("id", "oceanBase").attr("x", 0).attr("y", 0).attr("width", graphWidth).attr("height", graphHeight);

void function applyDefaultStyle() {
    biomes.attr("opacity", null).attr("filter", null);
    stateBorders.attr("opacity", .8).attr("stroke", "#56566d").attr("stroke-width", 1).attr("stroke-dasharray", "2").attr("stroke-linecap", "butt").attr("filter", null);
    provinceBorders.attr("opacity", .8).attr("stroke", "#56566d").attr("stroke-width", .2).attr("stroke-dasharray", "1").attr("stroke-linecap", "butt").attr("filter", null);
    cells.attr("opacity", null).attr("stroke", "#808080").attr("stroke-width", .1).attr("filter", null).attr("mask", null);

    gridOverlay.attr("opacity", .8).attr("stroke", "#808080").attr("stroke-width", .5).attr("stroke-dasharray", null).attr("transform", null).attr("filter", null).attr("mask", null);
    coordinates.attr("opacity", 1).attr("data-size", 12).attr("font-size", 12).attr("stroke", "#d4d4d4").attr("stroke-width", 1).attr("stroke-dasharray", 5).attr("filter", null).attr("mask", null);
    compass.attr("opacity", .8).attr("transform", null).attr("filter", null).attr("mask", "url(#water)").attr("shape-rendering", "optimizespeed");
    if (!d3.select("#initial").size()) d3.select("#rose").attr("transform", "translate(80 80) scale(.25)");

    coastline.attr("opacity", .5).attr("stroke", "#1f3846").attr("stroke-width", .7).attr("filter", "url(#dropShadow)");
    styleCoastlineAuto.checked = true;
    relig.attr("opacity", .6).attr("stroke", "#777777").attr("stroke-width", .2).attr("filter", null).attr("fill-rule", "evenodd");
    cults.attr("opacity", .6).attr("stroke", "#777777").attr("stroke-width", .5).attr("filter", null).attr("fill-rule", "evenodd");
    icons.selectAll("g").attr("opacity", null).attr("fill", "#ffffff").attr("stroke", "#3e3e4b").attr("filter", null).attr("mask", null);
    landmass.attr("opacity", 1).attr("fill", "#eef6fb").attr("filter", null);
    markers.attr("opacity", null).attr("filter", "url(#dropShadow01)");
    styleRescaleMarkers.checked = true;
    prec.attr("opacity", null).attr("stroke", "#000000").attr("stroke-width", .1).attr("fill", "#003dff").attr("filter", null);
    population.attr("opacity", null).attr("stroke-width", 1.6).attr("stroke-dasharray", null).attr("stroke-linecap", "butt").attr("filter", null);
    population.select("#rural").attr("stroke", "#0000ff");
    population.select("#urban").attr("stroke", "#ff0000");

    freshwater.attr("opacity", .5).attr("fill", "#a6c1fd").attr("stroke", "#5f799d").attr("stroke-width", .7).attr("filter", null);
    salt.attr("opacity", .5).attr("fill", "#409b8a").attr("stroke", "#388985").attr("stroke-width", .7).attr("filter", null);

    terrain.attr("opacity", null).attr("filter", null).attr("mask", null);
    rivers.attr("opacity", null).attr("fill", "#5d97bb").attr("filter", null);
    roads.attr("opacity", .9).attr("stroke", "#d06324").attr("stroke-width", .7).attr("stroke-dasharray", "2").attr("stroke-linecap", "butt").attr("filter", null);
    ruler.attr("opacity", null).attr("filter", null);
    searoutes.attr("opacity", .8).attr("stroke", "#ffffff").attr("stroke-width", .45).attr("stroke-dasharray", "1 2").attr("stroke-linecap", "round").attr("filter", null);

    regions.attr("opacity", .4).attr("filter", null);
    statesHalo.attr("stroke-width", 10).attr("opacity", 1);
    provs.attr("opacity", .6).attr("filter", null);

    temperature.attr("opacity", null).attr("fill", "#000000").attr("stroke-width", 1.8).attr("fill-opacity", .3).attr("font-size", "8px").attr("stroke-dasharray", null).attr("filter", null).attr("mask", null);
    texture.attr("opacity", null).attr("filter", null).attr("mask", "url(#land)");
    texture.select("image").attr("x", 0).attr("y", 0);
    zones.attr("opacity", .6).attr("stroke", "#333333").attr("stroke-width", 0).attr("stroke-dasharray", null).attr("stroke-linecap", "butt").attr("filter", null).attr("mask", null);
    trails.attr("opacity", .9).attr("stroke", "#d06324").attr("stroke-width", .25).attr("stroke-dasharray", ".8 1.6").attr("stroke-linecap", "butt").attr("filter", null);

    // ocean and svg default style
    svg.attr("background-color", "#000000").attr("filter", null);
    const mapFilter = document.querySelector("#mapFilters .pressed");
    if (mapFilter) mapFilter.classList.remove("pressed");
    ocean.attr("opacity", null);
    oceanLayers.select("rect").attr("fill", "#53679f");
    oceanLayers.attr("filter", null);
    oceanPattern.attr("opacity", null);
    oceanLayers.selectAll("path").attr("display", null);
    styleOceanPattern.value = "url(#pattern1)";
    svg.select("#oceanic rect").attr("filter", "url(#pattern1)");

    // heightmap style
    terrs.attr("opacity", null).attr("filter", null).attr("mask", "url(#land)").attr("stroke", "none");
    const changed = styleHeightmapSchemeInput.value !== "bright" ||
                    styleHeightmapTerracingInput.value != 0 ||
                    styleHeightmapSkipInput.value != 5 ||
                    styleHeightmapSimplificationInput.value != 0 ||
                    styleHeightmapCurveInput.value != 0;
    styleHeightmapSchemeInput.value = "bright";
    styleHeightmapTerracingInput.value = styleHeightmapTerracingOutput.value = 0;
    styleHeightmapSkipInput.value = styleHeightmapSkipOutput.value = 5;
    styleHeightmapSimplificationInput.value = styleHeightmapSimplificationOutput.value = 0;
    styleHeightmapCurveInput.value = 0;
    if (changed) drawHeightmap();

    // legend
    legend.attr("font-family", "Almendra SC").attr("data-font", "Almendra+SC").attr("font-size", 13).attr("data-size", 13).attr("data-x", 99).attr("data-y", 93).attr("stroke-width", 2.5).attr("stroke", "#812929").attr("stroke-dasharray", "0 4 10 4").attr("stroke-linecap", "round");
    styleLegendBack.value = "#ffffff";
    styleLegendOpacity.value = styleLegendOpacityOutput.value = .8;
    styleLegendColItems.value = styleLegendColItemsOutput.value = 8;
    if (legend.selectAll("*").size() && window.redrawLegend) redrawLegend();

    const citiesSize = Math.max(rn(8 - regionsInput.value / 20), 3);
    burgLabels.select("#cities").attr("fill", "#3e3e4b").attr("opacity", 1).attr("font-family", "Almendra SC").attr("data-font", "Almendra+SC").attr("font-size", citiesSize).attr("data-size", citiesSize);
    burgIcons.select("#cities").attr("opacity", 1).attr("size", 1).attr("stroke-width", .24).attr("fill", "#ffffff").attr("stroke", "#3e3e4b").attr("fill-opacity", .7).attr("stroke-dasharray", "").attr("stroke-linecap", "butt");
    anchors.select("#cities").attr("opacity", 1).attr("fill", "#ffffff").attr("stroke", "#3e3e4b").attr("stroke-width", 1.2).attr("size", 2);

    burgLabels.select("#towns").attr("fill", "#3e3e4b").attr("opacity", 1).attr("font-family", "Almendra SC").attr("data-font", "Almendra+SC").attr("font-size", 3).attr("data-size", 4);
    burgIcons.select("#towns").attr("opacity", 1).attr("size", .5).attr("stroke-width", .12).attr("fill", "#ffffff").attr("stroke", "#3e3e4b").attr("fill-opacity", .7).attr("stroke-dasharray", "").attr("stroke-linecap", "butt");
    anchors.select("#towns").attr("opacity", 1).attr("fill", "#ffffff").attr("stroke", "#3e3e4b").attr("stroke-width", 1.2).attr("size", 1);

    const stateLabelSize = Math.max(rn(24 - regionsInput.value / 6), 6);
    labels.select("#states").attr("fill", "#3e3e4b").attr("opacity", 1).attr("stroke", "#3a3a3a").attr("stroke-width", 0).attr("font-family", "Almendra SC").attr("data-font", "Almendra+SC").attr("font-size", stateLabelSize).attr("data-size", stateLabelSize).attr("filter", null);
    labels.select("#addedLabels").attr("fill", "#3e3e4b").attr("opacity", 1).attr("stroke", "#3a3a3a").attr("stroke-width", 0).attr("font-family", "Almendra SC").attr("data-font", "Almendra+SC").attr("font-size", 18).attr("data-size", 18).attr("filter", null);
    invokeActiveZooming();

    fogging.attr("opacity", .8).attr("fill", "#000000").attr("stroke-width", 5);
}()

// active zooming feature
function invokeActiveZooming() {
    if (styleCoastlineAuto.checked) {
        // toggle shade/blur filter for coatline on zoom
        let filter = scale > 2.6 ? "url(#blurFilter)" : "url(#dropShadow)";
        if (scale > 1.5 && scale <= 2.6) filter = null;
        coastline.attr("filter", filter);
    }

    // rescale lables on zoom
    if (labels.style("display") !== "none") {
        labels.selectAll("g").each(function(d) {
            if (this.id === "burgLabels") return;
            const desired = +this.dataset.size;
            const relative = Math.max(rn((desired + desired / scale) / 2, 2), 1);
            this.getAttribute("font-size", relative);
            const hidden = hideLabels.checked && (relative * scale < 6 || relative * scale > 50);
            if (hidden) this.classList.add("hidden"); else this.classList.remove("hidden");
        });
    }

    // turn off ocean pattern if scale is big (improves performance)
    oceanPattern.select("rect").attr("fill", scale > 10 ? "#fff" : "url(#oceanic)").attr("opacity", scale > 10 ? .2 : null);

    // change states halo width
    // TODO: if (!customization) {
        const haloSize = rn(styleStatesHaloWidth.value / scale, 1);
        statesHalo.attr("stroke-width", haloSize).style("display", haloSize > 3 ? "block" : "none");
    //}

    // rescale map markers
    if (styleRescaleMarkers.checked && markers.style("display") !== "none") {
        markers.selectAll("use").each(function(d) {
            const x = +this.dataset.x, y = +this.dataset.y, desired = +this.dataset.size;
            const size = Math.max(desired * 5 + 25 / scale, 1);
            d3.select(this).attr("x", x - size/2).attr("y", y - size).attr("width", size).attr("height", size);
        });
    }

    // rescale rulers to have always the same size
    if (ruler.style("display") !== "none") {
        const size = rn(1 / scale ** .3 * 2, 1);
        ruler.selectAll("circle").attr("r", 2 * size).attr("stroke-width", .5 * size);
        ruler.selectAll("rect").attr("stroke-width", .5 * size);
        ruler.selectAll("text").attr("font-size", 10 * size);
        ruler.selectAll("line, path").attr("stroke-width", size);
    }
}

function zoomed() {
  const transform = d3.event.transform;
  const scaleDiff = scale - transform.k;
  const positionDiff = viewX - transform.x | viewY - transform.y;
  scale = transform.k;
  viewX = transform.x;
  viewY = transform.y;
  viewbox.attr("transform", transform);

  // update grid only if view position
  // TODO: if (positionDiff) drawCoordinates();

  // rescale only if zoom is changed
  if (scaleDiff) {
    invokeActiveZooming();
    // TODO: drawScaleBar();
  }
}

function rn(v, d = 0) {
    const m = Math.pow(10, d);
    return Math.round(v * m) / m;
}

(async function run() {
    await mapgen_init();
}());
