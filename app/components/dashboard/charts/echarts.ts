import {
  GridComponent,
  LegendComponent,
  TooltipComponent,
} from "echarts/components";
import { init, use, type ECharts, type EChartsCoreOption } from "echarts/core";
import { CanvasRenderer } from "echarts/renderers";

export async function loadECharts() {
  const { BarChart, LineChart } = await import("./echartsCharts");
  use([
    BarChart,
    LineChart,
    GridComponent,
    LegendComponent,
    TooltipComponent,
    CanvasRenderer,
  ]);
  return init;
}

export type { ECharts, EChartsCoreOption };
