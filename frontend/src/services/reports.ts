import { requireFeatures } from '../settings/storage.ts';
import type { AppSettings, BacktestReport, RunHistoryItem } from '../types.ts';
import { REPORTS_KEY } from '../settings/storage.ts';

const HISTORY_LIMIT = 8;

export function loadRunHistory(storage: Storage): RunHistoryItem[] {
  const raw = storage.getItem(REPORTS_KEY);
  if (!raw) return [];
  const parsed: unknown = JSON.parse(raw);
  if (!Array.isArray(parsed) || !parsed.every(isHistoryItem)) {
    throw new Error('Saved report history is invalid; existing data was left unchanged.');
  }
  return parsed.slice(0, HISTORY_LIMIT);
}

function isHistoryItem(value: unknown): value is RunHistoryItem {
  if (typeof value !== 'object' || value === null) return false;
  const item = value as Record<string, unknown>;
  return (
    typeof item.id === 'string' &&
    typeof item.created_at === 'string' &&
    typeof item.instrument === 'string' &&
    typeof item.data_class === 'string' &&
    typeof item.dataset_fingerprint === 'string' &&
    typeof item.net_pnl === 'number' &&
    typeof item.return_fraction === 'number' &&
    typeof item.closed_trades === 'number'
  );
}

export function saveRunHistory(
  report: BacktestReport,
  storage: Storage,
  createdAt = new Date().toISOString(),
): RunHistoryItem[] {
  const existing = loadRunHistory(storage);
  const item: RunHistoryItem = {
    id: `${createdAt}:${report.dataset_fingerprint}`,
    created_at: createdAt,
    instrument: report.instrument,
    data_class: report.data_class,
    dataset_fingerprint: report.dataset_fingerprint,
    net_pnl: report.metrics.net_pnl,
    return_fraction: report.metrics.return_fraction,
    closed_trades: report.metrics.closed_trades,
  };
  const history = [item, ...existing].slice(0, HISTORY_LIMIT);
  storage.setItem(REPORTS_KEY, JSON.stringify(history));
  return history;
}

export function reportDownloads(
  report: BacktestReport,
  settings: AppSettings,
): Array<{ name: string; content: string; type: string }> {
  requireFeatures(settings, ['report_export']);
  const instrument = report.instrument.replace(/[^a-zA-Z0-9_-]/g, '_');
  return [
    {
      name: `${instrument}-backtest.json`,
      content: JSON.stringify(report, null, 2),
      type: 'application/json',
    },
    {
      name: `${instrument}-metrics.csv`,
      content: rowsToCsv({ ...report.metrics }),
      type: 'text/csv',
    },
    {
      name: `${instrument}-orders.csv`,
      content: recordsToCsv(report.orders),
      type: 'text/csv',
    },
    {
      name: `${instrument}-trades.csv`,
      content: recordsToCsv(report.trades),
      type: 'text/csv',
    },
    {
      name: `${instrument}-equity-curve.csv`,
      content: recordsToCsv(report.equity_curve),
      type: 'text/csv',
    },
  ];
}

function rowsToCsv(record: Record<string, unknown>): string {
  return `metric,value\r\n${Object.entries(record)
    .map(([key, value]) => `${escapeCsv(key)},${escapeCsv(value)}`)
    .join('\r\n')}\r\n`;
}

function recordsToCsv(records: Array<Record<string, unknown>>): string {
  if (records.length === 0) return 'no_records\r\n';
  const headers = [...new Set(records.flatMap((record) => Object.keys(record)))];
  const lines = records.map((record) =>
    headers.map((header) => escapeCsv(record[header])).join(','),
  );
  return `${headers.map(escapeCsv).join(',')}\r\n${lines.join('\r\n')}\r\n`;
}

function escapeCsv(value: unknown): string {
  const text = value === null || value === undefined ? '' : String(value);
  return `"${text.replaceAll('"', '""')}"`;
}

export function downloadFile(
  file: { name: string; content: string; type: string },
): void {
  const blob = new Blob([file.content], { type: `${file.type};charset=utf-8` });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement('a');
  anchor.href = url;
  anchor.download = file.name;
  anchor.click();
  URL.revokeObjectURL(url);
}
