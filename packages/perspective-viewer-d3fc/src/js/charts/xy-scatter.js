// ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
// ┃ ██████ ██████ ██████       █      █      █      █      █ █▄  ▀███ █       ┃
// ┃ ▄▄▄▄▄█ █▄▄▄▄▄ ▄▄▄▄▄█  ▀▀▀▀▀█▀▀▀▀▀ █ ▀▀▀▀▀█ ████████▌▐███ ███▄  ▀█ █ ▀▀▀▀▀ ┃
// ┃ █▀▀▀▀▀ █▀▀▀▀▀ █▀██▀▀ ▄▄▄▄▄ █ ▄▄▄▄▄█ ▄▄▄▄▄█ ████████▌▐███ █████▄   █ ▄▄▄▄▄ ┃
// ┃ █      ██████ █  ▀█▄       █ ██████      █      ███▌▐███ ███████▄ █       ┃
// ┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫
// ┃ Copyright (c) 2017, the Perspective Authors.                              ┃
// ┃ ╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌ ┃
// ┃ This file is part of the Perspective library, distributed under the terms ┃
// ┃ of the [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0). ┃
// ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

import * as fc from "d3fc";
import { axisFactory } from "../axis/axisFactory";
import { chartCanvasFactory } from "../axis/chartFactory";
import {
    pointSeriesCanvas,
    symbolTypeFromColumn,
} from "../series/pointSeriesCanvas";
import { pointData } from "../data/pointData";
import {
    seriesColorsFromColumn,
    seriesColorsFromDistinct,
    colorScale,
} from "../series/seriesColors";
import { seriesLinearRange, seriesColorRange } from "../series/seriesRange";
import { symbolLegend, colorLegend, colorGroupLegend } from "../legend/legend";
import { colorRangeLegend } from "../legend/colorRangeLegend";
import { filterDataByGroup } from "../legend/filter";
import withGridLines from "../gridlines/gridlines";
import { hardLimitZeroPadding } from "../d3fc/padding/hardLimitZero";
import zoomableChart from "../zoom/zoomableChart";
import nearbyTip from "../tooltip/nearbyTip";
import { symbolsObj } from "../series/seriesSymbols";

/**
 * Define a clamped scaling factor based on the container size for bubble plots.
 *
 * @param {Array} p1 a point as a tuple of `Number`
 * @param {Array} p2 a second point as a tuple of `Number`
 * @returns a function `container -> integer` which calculates a scaling factor
 * from the linear function (clamped) defgined by the input points
 */
function interpolate_scale([x1, y1], [x2, y2]) {
    const m = (y2 - y1) / (x2 - x1);
    const b = y2 - m * x2;
    return function (container) {
        const node = container.node();
        const shortest_axis = Math.min(node.clientWidth, node.clientHeight);
        return Math.min(y2, Math.max(y1, m * shortest_axis + b));
    };
}

/**
 * Overrides specific symbols based on plugin settings. This modifies in-place _and_ returns the value.
 * @param {any} settings
 * @param {d3.ScaleOrdinal} symbols
 */
function overrideSymbols(settings, symbols) {
    if (!symbols) {
        return;
    }
    const symbolCol = settings.realValues[4];
    const columnType = settings.mainValues.find(
        (val) => val.name === symbolCol
    )?.type;
    let domain = symbols.domain();
    let range = symbols.range();
    let len = range.length;
    for (let i in domain) {
        range[i] = range[i % len];
    }
    settings.columns?.[symbolCol]?.symbols?.forEach(({ key, value }) => {
        // TODO: Define custom symbol types based on the values passed in here.
        // https://d3js.org/d3-shape/symbol#custom-symbols
        let symbolType = symbolsObj[value] ?? d3.symbolCircle;

        let i = domain.findIndex((val) => {
            switch (columnType) {
                case "date":
                case "datetime":
                    return Date(val) === Date(key);
                default:
                    return String(val) === String(key);
            }
        });
        if (i === -1) {
            console.error(
                `Could not find row with value ${key} when overriding symbols!`
            );
        }
        range[i] = symbolType;
    });
    symbols.range(range);
    return symbols;
}

/**
 * @param {d3.Selection} container - d3.Selection of the outer div
 * @param {any} settings - settings as defined in the Update method in plugin.js
 */
function xyScatter(container, settings) {
    const colorBy = settings.realValues[2];
    let hasColorBy = !!colorBy;
    let isColoredByString =
        settings.mainValues.find((x) => x.name === colorBy)?.type === "string";

    let color = null;
    let legend = null;

    const symbolCol = settings.realValues[4];
    const symbols = overrideSymbols(
        settings,
        symbolTypeFromColumn(settings, symbolCol)
    );

    const data = pointData(settings, filterDataByGroup(settings));

    if (hasColorBy && isColoredByString) {
        if (!!symbolCol) {
            // TODO: Legend should have cartesian product labels (ColorBy|SplitBy)
            // For now, just use monocolor legends.
            color = seriesColorsFromDistinct(settings, data);
            legend = symbolLegend().settings(settings).scale(symbols);
        } else {
            color = seriesColorsFromColumn(settings, colorBy);
            legend = colorLegend().settings(settings).scale(color);
        }
    } else if (hasColorBy) {
        color = seriesColorRange(settings, data, "colorValue");
        legend = colorRangeLegend().scale(color);
    } else {
        // always use default color
        color = colorScale().settings(settings).domain([""])();
        legend = symbolLegend().settings(settings).scale(symbols);
    }

    const size = settings.realValues[3]
        ? seriesLinearRange(settings, data, "size").range([10, 10000])
        : null;

    const label = settings.realValues[5];

    const scale_factor = interpolate_scale([600, 0.1], [1600, 1])(container);
    const series = fc
        .seriesCanvasMulti()
        .mapping((data, index) => data[index])
        .series(
            data.map((series) =>
                pointSeriesCanvas(
                    settings,
                    symbolCol,
                    size,
                    color,
                    label,
                    symbols,
                    scale_factor
                )
            )
        );

    const axisDefault = () =>
        axisFactory(settings)
            .settingName("mainValues")
            .paddingStrategy(hardLimitZeroPadding())
            .pad([0.1, 0.1]);

    const xAxis = axisDefault()
        .settingValue(settings.mainValues[0].name)
        .memoValue(settings.axisMemo[0])
        .valueName("x")(data);

    const yAxis = axisDefault()
        .orient("vertical")
        .settingValue(settings.mainValues[1].name)
        .memoValue(settings.axisMemo[1])
        .valueName("y")(data);

    const chart = chartCanvasFactory(xAxis, yAxis)
        .xLabel(settings.mainValues[0].name)
        .yLabel(settings.mainValues[1].name)
        .plotArea(withGridLines(series, settings).canvas(true));

    chart.xNice && chart.xNice();
    chart.yNice && chart.yNice();

    const zoomChart = zoomableChart()
        .chart(chart)
        .settings(settings)
        .xScale(xAxis.scale)
        .yScale(yAxis.scale)
        .canvas(true);

    const toolTip = nearbyTip()
        .scaleFactor(scale_factor)
        .settings(settings)
        .canvas(true)
        .xScale(xAxis.scale)
        .xValueName("x")
        .yValueName("y")
        .yScale(yAxis.scale)
        .color(!hasColorBy && color)
        .size(size)
        .data(data);

    // render
    container.datum(data).call(zoomChart);
    container.call(toolTip);
    if (legend) container.call(legend);
}

xyScatter.plugin = {
    name: "X/Y Scatter",
    category: "X/Y Chart",
    max_cells: 50000,
    max_columns: 50,
    render_warning: true,
    initial: {
        type: "number",
        count: 2,
        names: [
            "X Axis",
            "Y Axis",
            "Color",
            "Size",
            "Symbol",
            "Label",
            "Tooltip",
        ],
    },
    selectMode: "toggle",
};

export default xyScatter;
