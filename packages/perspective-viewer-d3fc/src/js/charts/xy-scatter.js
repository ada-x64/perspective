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
import { select } from "d3";
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
import { gridLayoutMultiChart } from "../layout/gridLayoutMultiChart";
import xyScatterSeries from "../series/xy-scatter/xyScatterSeries";

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

    const axisDefault = () =>
        axisFactory(settings)
            .settingName("mainValues")
            .paddingStrategy(hardLimitZeroPadding())
            .pad([0.1, 0.1]);

    // TODO: Axis labels on the grid
    const xAxis = axisDefault()
        .settingValue(settings.mainValues[0].name)
        .memoValue(settings.axisMemo[0])
        .valueName("x")(data);

    const yAxis = axisDefault()
        .orient("vertical")
        .settingValue(settings.mainValues[1].name)
        .memoValue(settings.axisMemo[1])
        .valueName("y")(data);

    const xyGrid = gridLayoutMultiChart()
        .svg(false)
        .elementsPrefix("xy-scatter");
    container.datum(data).call(xyGrid);

    const xyContainer = xyGrid.chartContainer();
    const xyEnter = xyGrid.chartEnter();
    const xyDiv = xyGrid.chartDiv();
    const xyTitle = xyGrid.chartTitle();
    const containerSize = xyGrid.containerSize();

    // TODO: This isn't rendering
    if (legend) xyContainer.call(legend);

    xyTitle.each((d, i, nodes) => select(nodes[i]).text(d.key));
    xyEnter
        .merge(xyDiv)
        .attr(
            "transform",
            `translate(${containerSize.width / 2}, ${containerSize.height / 2})`
        )
        .each(function (data) {
            const xyElement = select(this);
            xyScatterSeries()
                .settings(settings)
                .data([data])
                .color(color)
                .symbols(symbols)(xyElement);
        });
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
