export function removeLoading() {
    d3.select("#loading").transition().duration(5000).style("opacity", 0).remove();
    d3.select("#initial").transition().duration(5000).attr("opacity", 0).remove();
    d3.select("#optionsContainer").transition().duration(3000).style("opacity", 1);
    d3.select("#tooltip").transition().duration(3000).style("opacity", 1);
}

export function undrawAll() {
    d3.select("#viewbox").selectAll("path, circle, polygon, line, text, use, #zones > g, #ruler > g").remove();
    d3.select("#deftemp").selectAll("path, clipPath").remove();
    // TODO: notes
    //notes = [];
    unfog();
}

export function unfog() {
    d3.select("#fog").selectAll("path").remove();
    let fogging = d3.select("#fogging");
    fogging.selectAll("path").remove();
    fogging.attr("display", "none");
}

export function drawCells(cellVertexCoords) {
    let cells = d3.select("#cells");
    cells.selectAll("path").remove();
    let path = "";
    cellVertexCoords.forEach(i => path += "M" + i);
    cells.append("path").attr("d", path);
}

export function clearCells() {
    d3.select("#cells").selectAll("path").remove();
}
