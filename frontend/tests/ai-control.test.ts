import assert from 'node:assert/strict';
import test from 'node:test';
import { renderLayout } from '../src/components/layout.ts';
import {
  aiToolDefinitions,
  completeAiTask,
  createAiTask,
  executeAiTool,
  isSupportedImageContent,
  makeAiSnapshot,
  retainAiTask,
} from '../src/services/ai-control.ts';
import { forecastSvgPath } from '../src/pages/research.ts';
import { defaultSettings } from '../src/settings/storage.ts';
import { renderSettings } from '../src/pages/settings.ts';
import { renderAiControlCenter } from '../src/pages/ai-control-center.ts';
import { renderNativeAiPanel } from '../src/pages/native-ai.ts';
import type { NativeModelStatus, NativeToolDefinition } from '../src/services/native-ai.ts';

const nativeModelStatus: NativeModelStatus = {
  model_name: 'Gemma 3 4B IT · Q4_K_M',
  model_dir: 'C:\\models\\gemma-3-4b',
  engine: 'llama.cpp CPU runtime',
  phase: 'unloaded',
  model: {
    file_name: 'gemma-3-4b-it-Q4_K_M.gguf',
    expected_bytes: 100,
    actual_bytes: 100,
    present: true,
    integrity: 'size_matches_hash_not_checked',
  },
  vision_projector: {
    file_name: 'mmproj-model-f16.gguf',
    expected_bytes: 50,
    actual_bytes: 50,
    present: true,
    integrity: 'size_matches_hash_not_checked',
  },
  runtime_available: true,
  loaded: false,
  execution_device: 'CPU',
  cpu_fallback: true,
  context_size: 4096,
  estimated_model_memory_mb: 200,
  total_memory_mb: 8192,
  available_memory_mb: 6000,
  model_process_memory_mb: null,
  minimum_available_memory_mb: 6144,
  last_error: null,
};

const nativeTool: NativeToolDefinition = {
  id: 'trading.options.black_scholes.price',
  name: 'Black-Scholes price and Greeks',
  description: 'Calculate European option outputs.',
  category: 'financial_calculation',
  version: 1,
  permission_scope: 'local_calculation',
  input_schema: { type: 'object' },
  output_schema: { type: 'object' },
  timeout_ms: 1000,
  availability: 'available',
};

test('AI action is visible beside the theme control without a feature gate', () => {
  const html = renderLayout('dashboard', defaultSettings(), '', false, { mode: 'browser' });
  assert.match(html, /data-page="ai" aria-label="Open AI Control Center"/);
  assert.ok(html.indexOf('data-page="ai"') < html.indexOf('data-action="toggle-theme"'));
});

test('settings truthfully reports web search as unconfigured without exposing configuration secrets', () => {
  const html = renderSettings(defaultSettings(), { mode: 'unavailable', message: 'Desktop service unavailable.' }, {
    configured: false,
    provider: 'not_configured',
    notice: 'Web Search Not Configured. No search requests will be made.',
  });
  assert.match(html, /Web search/);
  assert.match(html, /Web Search Not Configured/);
  assert.match(html, /No search requests will be made/);
  assert.doesNotMatch(html, /API_KEY|Search Engine ID value/i);
});

test('AI app snapshot is bounded and excludes CSV contents and credentials', () => {
  const snapshot = makeAiSnapshot({
    page: 'ai',
    inspection: null,
    activeIndicators: [],
    wasmReady: true,
    localAnalystAvailable: true,
    forecastAvailable: false,
    desktopIpc: 'browser_mode',
    enabledFeatures: ['local_analyst'],
    lastOperation: null,
  });
  assert.equal(snapshot.dataset, null);
  assert.equal(snapshot.capabilities.marketProvider, 'not_configured');
  assert.equal(snapshot.capabilities.vision, 'not_configured');
  assert.deepEqual(Object.keys(snapshot).sort(), [
    'activeIndicators',
    'activeScreen',
    'apiVersion',
    'appVersion',
    'capabilities',
    'dataset',
    'enabledFeatures',
    'lastOperation',
    'schemaVersion',
  ]);
  assert.doesNotMatch(JSON.stringify(snapshot), /api.?key|secret|password|csvContents/i);
});

test('registered AI tools validate arguments and reject unknown capabilities', async () => {
  let activeScreen = 'dashboard';
  const context = {
    snapshot: () => makeAiSnapshot({
      page: activeScreen as 'dashboard' | 'ai',
      inspection: null,
      activeIndicators: [],
      wasmReady: false,
      localAnalystAvailable: false,
      forecastAvailable: false,
      desktopIpc: 'browser_mode',
      enabledFeatures: [],
      lastOperation: null,
    }),
    navigate: (screen: 'dashboard' | 'data' | 'backtest' | 'reports' | 'features' | 'settings' | 'ai') => {
      activeScreen = screen;
    },
  };
  assert.deepEqual(aiToolDefinitions.map((tool) => tool.id), ['get_app_state', 'navigate_to_screen']);
  const invalid = await executeAiTool('navigate_to_screen', { screen: 'powershell' }, context);
  assert.equal(invalid.ok, false);
  const oversized = await executeAiTool('navigate_to_screen', { screen: 'data', extra: 'x'.repeat(100) }, context);
  assert.equal(oversized.ok, false);
  const unknown = await executeAiTool('run_shell_command', { command: 'anything' }, context);
  assert.equal(unknown.ok, false);
  const valid = await executeAiTool('navigate_to_screen', { screen: 'data' }, context);
  assert.deepEqual(valid, { ok: true, toolId: 'navigate_to_screen', value: { activeScreen: 'data' } });
});

test('AI task history is bounded and records terminal state without fake progress', () => {
  let history = [];
  for (let index = 0; index < 25; index += 1) {
    const task = createAiTask('analyze_dataset', 'Local Rust analysis', `Task ${index}`);
    history = retainAiTask(history, completeAiTask(task, 'succeeded', 'Analysis completed.'));
  }
  assert.equal(history.length, 20);
  assert.equal(history[0]?.status, 'succeeded');
  assert.equal('progress' in history[0]!, false);
  assert.equal(history[0]?.cancellationAvailable, false);
});

test('forecast estimate series starts after observations and is not joined to history', () => {
  const path = forecastSvgPath([101, 102], 4, (index) => index * 10, (value) => 200 - value);
  assert.equal(path, 'M 40.00 99.00 L 50.00 98.00');
  assert.doesNotMatch(path, /M 30\.00/);
});

test('image header must match an allowed MIME type before local preview', () => {
  assert.equal(isSupportedImageContent(
    new Uint8Array([137, 80, 78, 71, 13, 10, 26, 10]),
    'image/png',
  ), true);
  assert.equal(isSupportedImageContent(
    new Uint8Array([137, 80, 78, 71, 13, 10, 26, 10]),
    'image/jpeg',
  ), false);
  assert.equal(isSupportedImageContent(
    new Uint8Array([82, 73, 70, 70, 0, 0, 0, 0, 87, 69, 66, 80]),
    'image/webp',
  ), true);
});

test('native model manager distinguishes downloaded assets from SHA-256 verification', () => {
  const html = renderNativeAiPanel({
    status: nativeModelStatus,
    busy: '',
    modelDir: nativeModelStatus.model_dir,
    messages: [],
    chatBusy: false,
    metrics: null,
    image: null,
    desktopMode: true,
    tools: [nativeTool],
    lastToolResult: null,
  });
  assert.match(html, /Present · integrity not checked/);
  assert.match(html, /Verify assets/);
  assert.match(html, /Remove downloaded assets/);
  assert.match(html, /Black-Scholes price and Greeks/);
  assert.doesNotMatch(html, /Verified calculation result/);
});

test('native chat stays disabled until model load and reports the active CPU loading state', () => {
  const html = renderNativeAiPanel({
    status: { ...nativeModelStatus, phase: 'loading' },
    busy: 'loading',
    modelDir: nativeModelStatus.model_dir,
    messages: [],
    chatBusy: false,
    metrics: null,
    image: null,
    desktopMode: true,
    tools: [nativeTool],
    lastToolResult: null,
  });
  assert.match(html, /Starting local llama\.cpp CPU inference/);
  assert.match(html, /The model is not loaded/);
  assert.match(html, /textarea[^>]+disabled/);
  assert.match(html, /CPU · 1 request at a time/);
});

test('native model panel clearly excludes browser inference and separates unverified math', () => {
  const html = renderNativeAiPanel({
    status: { ...nativeModelStatus, loaded: true, phase: 'ready' },
    busy: '',
    modelDir: nativeModelStatus.model_dir,
    messages: [{ role: 'assistant', content: 'Example output' }],
    chatBusy: false,
    metrics: null,
    image: null,
    desktopMode: false,
    tools: [nativeTool],
    lastToolResult: null,
  });
  assert.match(html, /Desktop-only · browser mode does not start native inference/);
  assert.match(html, /not independently cross-verified or confirmed financial results/);
  assert.doesNotMatch(html, /data-action="native-model-load"/);
});

test('native chat renders streaming output and an explicit cancel-and-unload action', () => {
  const html = renderNativeAiPanel({
    status: { ...nativeModelStatus, loaded: true, phase: 'ready' },
    busy: '',
    modelDir: nativeModelStatus.model_dir,
    messages: [{ role: 'user', content: 'Explain this screen' }],
    chatBusy: true,
    cancelRequested: false,
    metrics: null,
    image: null,
    desktopMode: true,
    tools: [nativeTool],
    lastToolResult: null,
  });
  assert.match(html, /id="native-stream-output"/);
  assert.match(html, /data-action="cancel-native-chat"/);
  assert.match(html, /Stop & unload model/);
  assert.match(html, /data-action="native-model-unload" disabled/);
});

test('screen understanding requires one user-selected frame and has no periodic capture control', () => {
  const snapshot = makeAiSnapshot({
    page: 'ai',
    inspection: null,
    activeIndicators: [],
    wasmReady: true,
    localAnalystAvailable: false,
    forecastAvailable: false,
    desktopIpc: 'ready',
    enabledFeatures: [],
    lastOperation: null,
  });
  const html = renderAiControlCenter({
    page: 'ai',
    snapshot,
    inspection: null,
    intervalSeconds: 60,
    tasks: [],
    activeTask: null,
    localReport: null,
    analystQuestion: '',
    baselineReport: null,
    baselineOptions: {
      exchange: '',
      currency: '',
      forecast_steps: 1,
      folds: 1,
      minimum_training_bars: 1,
      test_bars_per_fold: 1,
      horizons_bars: [1],
    },
    busy: false,
    analystEnabled: false,
    analystBlockedReason: null,
    forecastEnabled: false,
    forecastBlockedReason: null,
    nativeAiPanel: '<section></section>',
    nativeModelLoaded: true,
    nativeBusy: false,
    researchStatus: null,
    researchResult: null,
    researchSummary: null,
    researchBusy: false,
    visionAnalysis: null,
    captureActive: false,
  });
  assert.match(html, /Capture and analyze one frame/);
  assert.match(html, /system chooser asks you to select a screen or window/i);
  assert.doesNotMatch(html, /Capture interval|frames are sampled|setInterval/i);
});
