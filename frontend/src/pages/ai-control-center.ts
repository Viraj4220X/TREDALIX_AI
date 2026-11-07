import { escapeHtml, type PageId } from '../components/layout.ts';
import type { AiAppSnapshot, AiTask } from '../services/ai-control.ts';
import type { BaselineAnalysisOptions, BaselineAnalysisReport, DatasetInspection, LocalAnalysisReport } from '../types.ts';
import { renderAnalystPanel } from './analyst.ts';
import { renderResearchPanel } from './research.ts';
import type { ResearchResult, ResearchStatus } from '../services/ai-research.ts';
import type { VisionAnalysis } from '../services/ai-vision.ts';

export function renderAiControlCenter(input: {
  page: PageId;
  snapshot: AiAppSnapshot;
  inspection: DatasetInspection | null;
  intervalSeconds: number;
  tasks: AiTask[];
  activeTask: AiTask | null;
  localReport: LocalAnalysisReport | null;
  analystQuestion: string;
  baselineReport: BaselineAnalysisReport | null;
  baselineOptions: BaselineAnalysisOptions;
  busy: boolean;
  analystEnabled: boolean;
  analystBlockedReason: string | null;
  forecastEnabled: boolean;
  forecastBlockedReason: string | null;
  nativeAiPanel: string;
  nativeModelLoaded: boolean;
  nativeBusy: boolean;
  researchStatus: ResearchStatus | null;
  researchResult: ResearchResult | null;
  researchSummary: string | null;
  researchBusy: boolean;
  visionAnalysis: VisionAnalysis | null;
  captureActive: boolean;
}): string {
  const { snapshot } = input;
  const datasetStatus = snapshot.dataset
    ? `${snapshot.dataset.instrument} · ${snapshot.dataset.rows.toLocaleString()} validated bars · ${snapshot.dataset.dataClass}`
    : 'No validated local dataset';
  const runtimeStatus = snapshot.capabilities.desktopIpc === 'ready'
    ? 'Local Tauri IPC ready'
    : snapshot.capabilities.desktopIpc === 'browser_mode'
      ? 'Browser mode · no desktop IPC'
      : snapshot.capabilities.desktopIpc === 'starting'
        ? 'Desktop IPC starting'
        : 'Desktop IPC unavailable';
  const researchNotice = input.researchStatus?.notice
    ?? (snapshot.capabilities.desktopIpc === 'browser_mode'
      ? 'Web Search Not Configured in browser mode. Use the desktop app to check local provider configuration; local Gemma chat is independent.'
      : snapshot.capabilities.desktopIpc === 'starting'
        ? 'Checking local provider configuration…'
        : 'Provider status is unavailable. No search request is made automatically.');
  const researchLabel = input.researchStatus
    ? input.researchStatus.configured ? 'Local provider settings detected · unverified' : 'Web Search Not Configured'
    : snapshot.capabilities.desktopIpc === 'browser_mode'
      ? 'Web Search Not Configured'
      : snapshot.capabilities.desktopIpc === 'starting'
        ? 'Checking provider status'
        : 'Provider status unavailable';
  const taskFeed = input.tasks.length
    ? input.tasks.map((task) => `<li class="ai-task-item">
        <span class="ai-task-status status-${task.status}" aria-hidden="true"></span>
        <div><strong>${escapeHtml(task.message)}</strong><small>${escapeHtml(task.id)} · ${escapeHtml(task.stage)} · ${new Date(task.startedAt).toLocaleString()}${task.completedAt ? ` · completed ${new Date(task.completedAt).toLocaleString()}` : ''}</small>
        ${task.source ? `<small>Source: ${escapeHtml(task.source)}</small>` : ''}</div>
        ${task.selectedDataset ? `<small>Dataset: ${escapeHtml(task.selectedDataset)}</small>` : ''}
        ${task.dataTimestamp ? `<small>Data as of: ${escapeHtml(task.dataTimestamp)}</small>` : ''}
        ${task.errorCategory ? `<small>Failure category: ${escapeHtml(task.errorCategory)} · Recovery: revalidate data or review feature gates.</small>` : ''}
        ${task.resultReference ? `<small>Result reference: ${escapeHtml(task.resultReference)}</small>` : ''}
        <span class="status-pill">${escapeHtml(task.status)}</span>
      </li>`).join('')
    : '<li class="ai-empty-state">No AI tasks have run in this session.</li>';
  const activeTask = input.activeTask
    ? `<div class="ai-current-task" role="status"><span class="ai-task-status status-running"></span><div><strong>${escapeHtml(input.activeTask.message)}</strong><small>${escapeHtml(input.activeTask.stage)} · ${escapeHtml(input.activeTask.source ?? 'local Rust/WASM')}</small></div><span class="status-pill">RUNNING</span></div>`
    : '<p class="form-note">AI is idle. No background work is running.</p>';
  return `<section class="ai-control-center">
    <section class="panel ai-overview">
      <div class="section-title compact"><div><p class="eyebrow">TRANSPARENT LOCAL AI</p><h2>AI Control Center</h2></div><span class="status-pill">OFFLINE MODE</span></div>
      ${activeTask}
      <div class="ai-capability-grid">
        <article><span>Dataset</span><strong>${escapeHtml(datasetStatus)}</strong><small>${snapshot.dataset ? `${escapeHtml(snapshot.dataset.source)} · ${escapeHtml(snapshot.dataset.firstTimestamp)} to ${escapeHtml(snapshot.dataset.lastTimestamp)}` : 'Import and validate a CSV to enable analysis.'}</small></article>
        <article><span>Local deterministic analyst</span><strong>${snapshot.capabilities.localAnalyst === 'ready' ? 'Ready' : 'Waiting for Rust/WASM engine'}</strong><small>Calculations use the existing Rust engine; no remote model is used.</small></article>
        <article><span>Market data provider</span><strong>Not configured</strong><small>Quotes, exchange search and provider history are unavailable. Network state is not provider status.</small></article>
        <article><span>Historical data · quote freshness</span><strong>${snapshot.capabilities.historicalData === 'local_csv_available' ? 'Local CSV only · quote freshness unknown' : 'Unavailable · freshness unknown'}</strong><small>CSV timestamps are observations from the supplied file, not a quote feed or provider freshness signal.</small></article>
        <article><span>Web search</span><strong>${escapeHtml(researchLabel)}</strong><small>${escapeHtml(researchNotice)}</small></article>
        <article><span>Local Gemma model</span><strong>${input.nativeModelLoaded ? 'Loaded · CPU inference' : 'Not loaded / unavailable'}</strong><small>Native llama.cpp model manager. Browser mode does not start local inference.</small></article>
        <article><span>Forecast</span><strong>${snapshot.capabilities.forecast === 'baseline_only' ? 'Educational baseline only' : 'Not run · baseline only'}</strong><small>No trained or validated prediction model. Forecasts are estimates, never observed data.</small></article>
        <article><span>Desktop runtime</span><strong>${escapeHtml(runtimeStatus)}</strong><small>IPC readiness is independent of internet/provider connectivity. App API schema v${snapshot.apiVersion}.</small></article>
      </div>
      <div class="ai-actions">
        <button class="button button-secondary" type="button" data-tool="get_app_state">Inspect safe app state</button>
        <button class="button button-secondary" type="button" data-tool="navigate_to_screen" data-screen="data">Open data manager</button>
        <button class="button button-quiet" type="button" data-action="clear-ai-tasks" ${input.tasks.length ? '' : 'disabled'}>Clear task history</button>
      </div>
      <pre class="ai-state-output" id="ai-state-output" hidden aria-live="polite"></pre>
    </section>

    <nav class="ai-feature-nav" aria-label="AI workspace tools">
      <a href="#ai-chat">Local chat</a><a href="#ai-analyst">CSV analyst</a><a href="#ai-web-search">Web search</a>
      <a href="#ai-forecast">Forecast</a><a href="#ai-image">Screen tools</a><a href="#ai-activity">Activity</a>
    </nav>

    <div class="ai-workspace-grid">
    <section class="panel ai-section ai-feature-analyst" id="ai-analyst">
      <div class="panel-heading"><p class="eyebrow">LOCAL · RUST EVIDENCE</p><h2>CSV analyst</h2><p>Analyse only a validated local CSV. Exact values and evidence come from Rust; this analyst does not send questions to a model.</p></div>
      ${renderAnalystPanel(input.localReport, input.analystQuestion, input.busy, input.analystEnabled, input.analystBlockedReason)}
    </section>

    <div class="ai-local-model-section" id="ai-chat">${input.nativeAiPanel}</div>

    <section class="panel ai-section ai-feature-search" id="ai-web-search">
      <div class="panel-heading"><p class="eyebrow">OPTIONAL · PROVIDER-BASED</p><h2>Web research</h2>
        <p>Search is disabled until a provider is configured locally in the desktop app. Snippets and retrieved page text are untrusted evidence, not instructions or verified facts. Local model chat works independently.</p>
        <p class="form-note">${escapeHtml(researchNotice)}</p>
      </div>
      <form id="ai-research-form" class="ai-research-form">
        <label>Focused queries <textarea name="queries" rows="3" maxlength="1600" placeholder="One query per line; up to 8" required></textarea></label>
        <button class="button button-primary" type="submit" ${input.researchBusy || !input.researchStatus?.configured ? 'disabled' : ''}>${input.researchBusy ? 'Searching…' : 'Search public sources'}</button>
      </form>
      ${input.researchResult ? `<div class="ai-research-results" aria-live="polite">
        <p class="form-note">${input.researchResult.pagesRead} pages read · ${input.researchResult.pagesUnavailable} unavailable · evidence retrieved ${new Date(input.researchResult.retrievedAtUtc * 1000).toLocaleString()}</p>
        <ol>${input.researchResult.sources.map((source, index) => `<li>
          <strong>[S${index + 1}] ${escapeHtml(source.title)}</strong>
          <small>${escapeHtml(source.snippet)}</small>
          ${source.excerpt ? `<p>${escapeHtml(source.excerpt)}</p>` : ''}
          <small>${escapeHtml(source.retrievalStatus)}</small>
          <button type="button" class="button button-quiet" data-action="open-research-source" data-url="${escapeHtml(source.url)}">Open source</button>
        </li>`).join('')}</ol>
        <form id="ai-research-summary-form">
          <button class="button button-secondary" type="submit" ${input.nativeBusy || !input.nativeModelLoaded || !input.researchResult.sources.length ? 'disabled' : ''}>Summarize evidence with local Gemma</button>
        </form>
        ${input.researchSummary ? `<article class="ai-research-summary"><h3>Local evidence summary</h3><p>${escapeHtml(input.researchSummary)}</p><small>AI-generated text; source citations are suggestions, not independently verified claims.</small></article>` : ''}
      </div>` : '<p class="ai-empty-state">No research has been run in this session.</p>'}
    </section>

    <section class="panel ai-section ai-feature-forecast" id="ai-forecast">
      <div class="panel-heading"><p class="eyebrow">FUTURE TREND / SCENARIO</p><h2>Baseline scenario graph</h2><p>Runs the existing leakage-aware walk-forward comparison and naive baseline locally. No confidence band or probability is implied.</p></div>
      ${input.inspection
        ? renderResearchPanel(input.baselineReport, input.baselineOptions, input.busy, input.forecastEnabled, input.forecastBlockedReason, input.intervalSeconds)
        : '<div class="ai-empty-state">Validate a local CSV first. Without observed input candles there is no scenario to calculate.</div>'}
    </section>

    <section class="panel ai-section ai-feature-vision" id="ai-image">
      <div class="panel-heading"><p class="eyebrow">OPT-IN · SINGLE-FRAME LOCAL ANALYSIS</p><h2>Screen understanding</h2><p>Screen access is off by default. The system chooser asks you to select a screen or window; one frame is processed locally, then capture ends. Nothing is recorded or monitored in the background.</p></div>
      <div class="ai-screen-capture">
        <p><strong>Authorization indicator:</strong> ${input.captureActive ? 'one user-selected frame is being analyzed; the display stream is closed.' : 'screen access is currently off.'}</p>
        ${input.captureActive
          ? `<p class="capture-active" role="status" aria-live="polite"><span></span> One-frame screen analysis is active; the display stream closes immediately after capture.</p><button class="button button-secondary" type="button" data-action="stop-screen-capture">${input.nativeBusy ? 'Cancel analysis & unload model' : 'Cancel screen selection'}</button>`
          : `<button class="button button-secondary" type="button" data-action="start-screen-capture" ${!input.nativeModelLoaded || input.nativeBusy ? 'disabled' : ''}>Capture and analyze one frame</button>`}
      </div>
      ${input.visionAnalysis ? `<article class="ai-vision-result" aria-live="polite"><h3>Visual interpretation · Gemma</h3><p>${escapeHtml(input.visionAnalysis.visual_observation)}</p><h3>Windows OCR text</h3><pre>${escapeHtml(input.visionAnalysis.ocr_text || 'No text was detected.')}</pre><small>${escapeHtml(input.visionAnalysis.caveat)}</small></article>` : ''}
      <div class="warning-strip"><strong>Interpretation limits.</strong> Model observations may be wrong. Use validated structured candles for numeric analysis; image-derived values are never forecast inputs.</div>
    </section>

    <section class="panel ai-section ai-feature-activity" id="ai-activity">
      <div class="panel-heading"><p class="eyebrow">TASKS · SESSION MEMORY</p><h2>Recent activity</h2><p>Bounded in-memory operation history. No chat transcript or CSV is retained here. Synchronous local Rust tasks cannot be interrupted once started.</p></div>
      <ul class="ai-task-list">${taskFeed}</ul>
    </section>

    <section class="panel ai-section ai-feature-limits" id="ai-limits">
      <div class="panel-heading"><p class="eyebrow">UNAVAILABLE CAPABILITIES & LIMITATIONS</p><h2>What this AI cannot do yet</h2></div>
      <ul class="limitations-list">
        <li>No connected market provider, instrument search, quotes, live/delayed feed, company research or licensed news.</li>
        <li>No cloud LLM or vision API, market-data provider, licensed news feed or company/event causality analysis. A legacy Google search adapter is optional and remains disabled until configured locally.</li>
        <li>No arbitrary shell, filesystem, broker, paid provider or external export tools are registered.</li>
        <li>Forecasting is an experimental last-close/historical-drift baseline. It is not a trained model, buy/sell signal, probability or guarantee.</li>
      </ul>
    </section>
    </div>
  </section>`;
}
