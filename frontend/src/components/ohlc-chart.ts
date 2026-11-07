import {
  CandlestickSeries,
  ColorType,
  CrosshairMode,
  HistogramSeries,
  LineSeries,
  LineStyle,
  createChart,
  type IChartApi,
  type ISeriesApi,
  type UTCTimestamp,
} from 'lightweight-charts';
import type {
  BaselineAnalysisReport,
  ChartCandle,
  ChartLinePoint,
  DatasetInspection,
} from '../types.ts';

type Series = ISeriesApi<'Line'> | ISeriesApi<'Histogram'>;

function time(value: number): UTCTimestamp {
  return value as UTCTimestamp;
}

function lineData(points: ChartLinePoint[]): Array<{ time: UTCTimestamp; value: number }> {
  return points.map((point) => ({ time: time(point.time), value: point.value }));
}

function chartOptions(height: number) {
  const style = getComputedStyle(document.documentElement);
  const text = style.getPropertyValue('--muted').trim();
  const grid = style.getPropertyValue('--border').trim();
  const surface = style.getPropertyValue('--surface').trim();
  return {
    autoSize: true,
    height,
    layout: {
      background: { type: ColorType.Solid, color: surface },
      textColor: text,
    },
    grid: {
      vertLines: { color: grid },
      horzLines: { color: grid },
    },
    rightPriceScale: { borderColor: grid },
    timeScale: { borderColor: grid, timeVisible: true, secondsVisible: false },
    crosshair: { mode: CrosshairMode.Normal },
  };
}

function checkbox(container: HTMLElement, name: string): HTMLInputElement | null {
  return container.querySelector<HTMLInputElement>(`[data-indicator="${name}"]`);
}

function setVisible(control: HTMLInputElement | null, series: Series[]): void {
  const visible = control?.checked ?? true;
  for (const item of series) item.applyOptions({ visible });
  control?.addEventListener('change', () => {
    for (const item of series) item.applyOptions({ visible: control.checked });
  });
}

function makeChart(container: HTMLElement, height: number): IChartApi {
  return createChart(container, chartOptions(height));
}

function candleData(candles: ChartCandle[]) {
  return candles.map((candle) => ({
    time: time(candle.time),
    open: candle.open,
    high: candle.high,
    low: candle.low,
    close: candle.close,
  }));
}

export function mountOhlcCharts(
  inspection: DatasetInspection | null,
  forecast: BaselineAnalysisReport | null,
  indicatorsEnabled: boolean,
): () => void {
  if (!inspection) return () => {};
  const priceElement = document.querySelector<HTMLElement>('#ohlc-chart');
  if (!priceElement) return () => {};
  const charts: IChartApi[] = [];
  const priceChart = makeChart(priceElement, 390);
  charts.push(priceChart);

  const style = getComputedStyle(document.documentElement);
  const green = style.getPropertyValue('--accent').trim();
  const red = style.getPropertyValue('--danger').trim();
  const cyan = style.getPropertyValue('--cyan').trim();
  const candles = priceChart.addSeries(CandlestickSeries, {
    upColor: green,
    downColor: red,
    borderUpColor: green,
    borderDownColor: red,
    wickUpColor: green,
    wickDownColor: red,
  });
  candles.setData(candleData(inspection.chart.candles));
  const sma = priceChart.addSeries(LineSeries, { color: cyan, lineWidth: 2, priceLineVisible: false, visible: indicatorsEnabled });
  sma.setData(indicatorsEnabled ? lineData(inspection.chart.sma_20) : []);
  const ema = priceChart.addSeries(LineSeries, { color: green, lineWidth: 2, priceLineVisible: false, visible: indicatorsEnabled });
  ema.setData(indicatorsEnabled ? lineData(inspection.chart.ema_20) : []);
  const upperBand = priceChart.addSeries(LineSeries, { color: `${cyan}88`, lineWidth: 1, priceLineVisible: false, visible: indicatorsEnabled });
  upperBand.setData(indicatorsEnabled ? lineData(inspection.chart.bollinger_upper_20) : []);
  const lowerBand = priceChart.addSeries(LineSeries, { color: `${cyan}88`, lineWidth: 1, priceLineVisible: false, visible: indicatorsEnabled });
  lowerBand.setData(indicatorsEnabled ? lineData(inspection.chart.bollinger_lower_20) : []);
  const volume = priceChart.addSeries(HistogramSeries, {
    priceScaleId: 'volume',
    priceFormat: { type: 'volume' },
    lastValueVisible: false,
    priceLineVisible: false,
  });
  volume.setData(inspection.chart.candles.flatMap((candle) =>
    candle.volume === null
      ? []
      : [{
          time: time(candle.time),
          value: candle.volume,
          color: candle.close >= candle.open ? `${green}66` : `${red}66`,
        }],
  ));
  const forecastSeries = forecast
    ? priceChart.addSeries(LineSeries, {
        color: style.getPropertyValue('--warning').trim(),
        lineWidth: 2,
        lineStyle: LineStyle.Dashed,
        title: 'MODEL ESTIMATE — NOT OBSERVED MARKET DATA',
        priceLineVisible: false,
      })
    : null;
  if (forecastSeries && forecast) {
    forecastSeries.setData(forecast.forecast.points.flatMap((item) =>
      item.point_estimate === null
        ? []
        : [{ time: time(item.timestamp_utc), value: item.point_estimate }],
    ));
  }
  priceChart.priceScale('volume').applyOptions({ scaleMargins: { top: 0.82, bottom: 0 } });
  priceChart.timeScale().fitContent();
  if (indicatorsEnabled) {
    setVisible(checkbox(priceElement, 'sma_20'), [sma]);
    setVisible(checkbox(priceElement, 'ema_20'), [ema]);
    setVisible(checkbox(priceElement, 'bollinger_20'), [upperBand, lowerBand]);
  }
  setVisible(checkbox(priceElement, 'volume'), [volume]);
  if (forecastSeries && priceElement.querySelector('[data-indicator="forecast"]')) {
    setVisible(checkbox(priceElement, 'forecast'), [forecastSeries]);
  }

  const rsiElement = document.querySelector<HTMLElement>('#rsi-chart');
  if (rsiElement) {
    const rsiChart = makeChart(rsiElement, 170);
    charts.push(rsiChart);
    const rsi = rsiChart.addSeries(LineSeries, { color: cyan, lineWidth: 2, priceLineVisible: false });
    rsi.setData(lineData(inspection.chart.rsi_14));
    rsi.createPriceLine({ price: 70, color: `${red}99`, lineWidth: 1, axisLabelVisible: false, title: '70' });
    rsi.createPriceLine({ price: 30, color: `${green}99`, lineWidth: 1, axisLabelVisible: false, title: '30' });
    rsiChart.timeScale().fitContent();
    setVisible(checkbox(priceElement, 'rsi_14'), [rsi]);
  }

  const macdElement = document.querySelector<HTMLElement>('#macd-chart');
  if (macdElement) {
    const macdChart = makeChart(macdElement, 170);
    charts.push(macdChart);
    const macdLine = macdChart.addSeries(LineSeries, { color: cyan, lineWidth: 2, priceLineVisible: false });
    macdLine.setData(lineData(inspection.chart.macd_line));
    const signal = macdChart.addSeries(LineSeries, { color: green, lineWidth: 2, priceLineVisible: false });
    signal.setData(lineData(inspection.chart.macd_signal));
    const histogram = macdChart.addSeries(HistogramSeries, { priceLineVisible: false });
    histogram.setData(inspection.chart.macd_histogram.map((point) => ({
      time: time(point.time),
      value: point.value,
      color: point.value >= 0 ? `${green}99` : `${red}99`,
    })));
    macdChart.timeScale().fitContent();
    setVisible(checkbox(priceElement, 'macd'), [macdLine, signal, histogram]);
  }

  const atrElement = document.querySelector<HTMLElement>('#atr-chart');
  if (atrElement) {
    const atrChart = makeChart(atrElement, 170);
    charts.push(atrChart);
    const atr = atrChart.addSeries(LineSeries, {
      color: cyan,
      lineWidth: 2,
      priceLineVisible: false,
    });
    atr.setData(lineData(inspection.chart.atr_14));
    atrChart.timeScale().fitContent();
    setVisible(checkbox(priceElement, 'atr_14'), [atr]);
  }

  let disposed = false;
  return () => {
    if (disposed) return;
    disposed = true;
    for (const chart of charts) chart.remove();
  };
}
