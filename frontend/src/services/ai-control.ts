import type { DatasetInspection } from '../types.ts';
import type { PageId } from '../components/layout.ts';

export type AiTaskStatus = 'queued' | 'running' | 'succeeded' | 'failed' | 'cancelled' | 'blocked';
export type AiTaskKind = 'validate_dataset' | 'analyze_dataset' | 'create_scenario' | 'run_backtest' | 'tool_call';

export interface AiTask {
  id: string;
  kind: AiTaskKind;
  status: AiTaskStatus;
  stage: string;
  message: string;
  startedAt: number;
  updatedAt: number;
  completedAt?: number;
  source?: string;
  selectedDataset?: string;
  dataTimestamp?: string;
  resultReference?: string;
  errorCategory?: 'no_data' | 'unsupported' | 'invalid_input' | 'engine_error';
  cancellationAvailable: false;
}

export interface AiAppSnapshot {
  schemaVersion: 1;
  apiVersion: 1;
  appVersion: string;
  activeScreen: PageId;
  lastOperation: null | { status: AiTaskStatus; summary: string };
  dataset: null | {
    instrument: string;
    rows: number;
    dataClass: string;
    intervalSeconds: number;
    source: string;
    firstTimestamp: string;
    lastTimestamp: string;
  };
  activeIndicators: string[];
  capabilities: {
    localAnalyst: 'ready' | 'unavailable';
    marketProvider: 'not_configured';
    historicalData: 'local_csv_available' | 'unavailable';
    quoteFreshness: 'unknown';
    companyResearch: 'not_configured';
    llm: 'not_configured';
    vision: 'not_configured';
    forecast: 'baseline_only' | 'not_run';
    desktopIpc: 'ready' | 'unavailable' | 'starting' | 'browser_mode';
    internetConnectivity: 'unknown';
  };
  enabledFeatures: string[];
}

export interface VisionModelRequest {
  image: Blob;
  mimeType: 'image/png' | 'image/jpeg' | 'image/webp';
  externalTransferConsent: boolean;
}

export interface VisionModelResult {
  modelId: string;
  summary: string;
  observations: Array<{
    description: string;
    evidenceType: 'image_visual_approximate';
    numericValue: null;
    uncertainty: string;
  }>;
  warnings: string[];
}

export interface VisionModel {
  readonly mode: 'local' | 'remote';
  analyzeChartImage(request: VisionModelRequest): Promise<VisionModelResult>;
}

export interface AiToolContext {
  snapshot: () => AiAppSnapshot;
  navigate: (page: PageId) => void;
}

export type AiToolResult =
  | { ok: true; toolId: 'get_app_state'; value: AiAppSnapshot }
  | { ok: true; toolId: 'navigate_to_screen'; value: { activeScreen: PageId } }
  | { ok: false; category: 'unknown_tool' | 'invalid_input' | 'execution_failed'; message: string };

const screens: readonly PageId[] = ['dashboard', 'data', 'backtest', 'reports', 'features', 'settings', 'ai'];
const MAX_TASK_HISTORY = 20;
const PNG_SIGNATURE = [137, 80, 78, 71, 13, 10, 26, 10];

export const aiToolDefinitions = [
  {
    id: 'get_app_state',
    version: 1,
    permission: 'read_only',
    capability: 'local_app_state',
    inputSchema: { type: 'object', additionalProperties: false, properties: {} },
    outputSchema: { type: 'object', required: ['schemaVersion', 'activeScreen', 'dataset', 'capabilities'] },
    timeoutMs: 1_000,
    cancellationSupported: false,
    maxPayloadBytes: 0,
    safeErrorCategory: 'execution_failed',
    auditFields: ['tool_id', 'status', 'started_at', 'completed_at'],
  },
  {
    id: 'navigate_to_screen',
    version: 1,
    permission: 'reversible_ui',
    capability: 'local_navigation',
    inputSchema: {
      type: 'object',
      additionalProperties: false,
      required: ['screen'],
      properties: { screen: { type: 'string', enum: screens } },
    },
    outputSchema: { type: 'object', required: ['activeScreen'] },
    timeoutMs: 1_000,
    cancellationSupported: false,
    maxPayloadBytes: 64,
    safeErrorCategory: 'execution_failed',
    auditFields: ['tool_id', 'status', 'started_at', 'completed_at'],
  },
] as const;

export function createAiTask(
  kind: AiTaskKind,
  stage: string,
  message: string,
  source?: string,
  selectedDataset?: string,
): AiTask {
  const now = Date.now();
  return {
    id: globalThis.crypto?.randomUUID?.() ?? `task-${now}-${Math.random().toString(36).slice(2)}`,
    kind,
    status: 'running',
    stage,
    message,
    startedAt: now,
    updatedAt: now,
    ...(source ? { source } : {}),
    ...(selectedDataset ? { selectedDataset } : {}),
    cancellationAvailable: false,
  };
}

export function completeAiTask(
  task: AiTask,
  status: Extract<AiTaskStatus, 'succeeded' | 'failed' | 'cancelled' | 'blocked'>,
  message: string,
  errorCategory?: AiTask['errorCategory'],
): AiTask {
  const now = Date.now();
  return {
    ...task,
    status,
    stage: status === 'succeeded' ? 'Complete' : status === 'failed' ? 'Failed' : status === 'blocked' ? 'Blocked' : 'Cancelled',
    message,
    updatedAt: now,
    completedAt: now,
    ...(errorCategory ? { errorCategory } : {}),
  };
}

export function retainAiTask(history: readonly AiTask[], task: AiTask): AiTask[] {
  return [task, ...history.filter((item) => item.id !== task.id)].slice(0, MAX_TASK_HISTORY);
}

export function isSupportedImageContent(bytes: Uint8Array, mimeType: string): boolean {
  const isPng = mimeType === 'image/png' &&
    bytes.length >= PNG_SIGNATURE.length &&
    PNG_SIGNATURE.every((byte, index) => bytes[index] === byte);
  const isJpeg = mimeType === 'image/jpeg' &&
    bytes.length >= 3 && bytes[0] === 0xff && bytes[1] === 0xd8 && bytes[2] === 0xff;
  const isWebp = mimeType === 'image/webp' &&
    bytes.length >= 12 &&
    String.fromCharCode(...bytes.slice(0, 4)) === 'RIFF' &&
    String.fromCharCode(...bytes.slice(8, 12)) === 'WEBP';
  return isPng || isJpeg || isWebp;
}

export function makeAiSnapshot(input: {
  page: PageId;
  inspection: DatasetInspection | null;
  activeIndicators: string[];
  wasmReady: boolean;
  forecastAvailable: boolean;
  localAnalystAvailable: boolean;
  desktopIpc: AiAppSnapshot['capabilities']['desktopIpc'];
  enabledFeatures: string[];
  lastOperation: AiAppSnapshot['lastOperation'];
}): AiAppSnapshot {
  const { inspection } = input;
  return {
    schemaVersion: 1,
    apiVersion: 1,
    appVersion: '0.1.0',
    activeScreen: input.page,
    lastOperation: input.lastOperation,
    dataset: inspection
      ? {
          instrument: inspection.instrument,
          rows: inspection.row_count,
          dataClass: inspection.data_class,
          intervalSeconds: inspection.interval_seconds,
          source: inspection.source,
          firstTimestamp: inspection.first_timestamp,
          lastTimestamp: inspection.last_timestamp,
        }
      : null,
    activeIndicators: inspection ? [...input.activeIndicators].sort() : [],
    capabilities: {
      localAnalyst: input.localAnalystAvailable ? 'ready' : 'unavailable',
      marketProvider: 'not_configured',
      historicalData: inspection ? 'local_csv_available' : 'unavailable',
      quoteFreshness: 'unknown',
      companyResearch: 'not_configured',
      llm: 'not_configured',
      vision: 'not_configured',
      forecast: input.forecastAvailable ? 'baseline_only' : 'not_run',
      desktopIpc: input.desktopIpc,
      internetConnectivity: 'unknown',
    },
    enabledFeatures: [...input.enabledFeatures].sort(),
  };
}

export async function executeAiTool(
  id: string,
  input: unknown,
  context: AiToolContext,
): Promise<AiToolResult> {
  if (id === 'get_app_state') {
    if (input !== undefined && (!isPlainObject(input) || Object.keys(input).length !== 0)) {
      return { ok: false, category: 'invalid_input', message: 'get_app_state accepts no arguments.' };
    }
    try {
      return { ok: true, toolId: id, value: context.snapshot() };
    } catch {
      return { ok: false, category: 'execution_failed', message: 'Safe application state is temporarily unavailable.' };
    }
  }
  if (id === 'navigate_to_screen') {
    if (serializedInputBytes(input) > 64) {
      return { ok: false, category: 'invalid_input', message: 'Navigation input exceeds the allowed size.' };
    }
    if (!isPlainObject(input) || typeof input.screen !== 'string' || !screens.includes(input.screen as PageId)) {
      return { ok: false, category: 'invalid_input', message: 'Choose a supported application screen.' };
    }
    const screen = input.screen as PageId;
    try {
      context.navigate(screen);
      return { ok: true, toolId: id, value: { activeScreen: screen } };
    } catch {
      return { ok: false, category: 'execution_failed', message: 'The application could not open that screen.' };
    }
  }
  return { ok: false, category: 'unknown_tool', message: 'This app action is not registered or available.' };
}

function serializedInputBytes(input: unknown): number {
  try {
    return new TextEncoder().encode(JSON.stringify(input) ?? '').byteLength;
  } catch {
    return Number.POSITIVE_INFINITY;
  }
}

function isPlainObject(value: unknown): value is Record<string, unknown> {
  if (typeof value !== 'object' || value === null || Array.isArray(value)) return false;
  const prototype = Object.getPrototypeOf(value);
  return prototype === Object.prototype || prototype === null;
}
