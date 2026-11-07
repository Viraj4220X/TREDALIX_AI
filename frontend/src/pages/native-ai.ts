import { escapeHtml } from '../components/layout.ts';
import type {
  NativeChatMessage,
  NativeModelStatus,
  NativeToolDefinition,
} from '../services/native-ai.ts';

function integrityLabel(integrity: string): string {
  if (integrity === 'sha256_verified') return 'SHA-256 verified';
  if (integrity === 'size_matches_hash_not_checked') return 'Present · integrity not checked';
  if (integrity === 'size_mismatch') return 'File size mismatch';
  return 'Not downloaded';
}

function renderMessages(messages: readonly NativeChatMessage[], streaming: boolean): string {
  if (!messages.length && !streaming) {
    return '<p class="native-ai-empty">No saved chat in this session. Start with a prompt after loading Gemma.</p>';
  }
  return `${messages.map((message) => `<article class="native-ai-message native-ai-${message.role}">
    <span>${message.role === 'assistant' ? 'GEMMA 3 4B' : 'YOU'}</span>
    <p>${escapeHtml(message.content).replaceAll('\n', '<br>')}</p>
  </article>`).join('')}${streaming ? `<article class="native-ai-message native-ai-assistant native-ai-streaming">
    <span>GEMMA 3 4B · STREAMING</span><p id="native-stream-output" aria-live="polite"></p>
  </article>` : ''}`;
}

function modelManager(status: NativeModelStatus | null, busy: string, modelDir: string, desktopMode: boolean, chatBusy: boolean): string {
  const missingModel = !status?.model.present || !status.vision_projector.present;
  const loaded = status?.loaded === true;
  const memoryMinimum = status?.minimum_available_memory_mb ?? 6_144;
  const enoughMemory = status?.available_memory_mb !== null
    && status?.available_memory_mb !== undefined
    && status.available_memory_mb >= memoryMinimum;
  const removable = Boolean(
    status && (status.model.actual_bytes > 0 || status.vision_projector.actual_bytes > 0),
  );
  const fileInfo = status
    ? `<div class="native-model-assets">
        <div><span>Text model</span><strong>${escapeHtml(status.model.file_name)}</strong><small>${(status.model.expected_bytes / 1_000_000_000).toFixed(2)} GB · ${escapeHtml(integrityLabel(status.model.integrity))}</small></div>
        <div><span>Vision projector</span><strong>${escapeHtml(status.vision_projector.file_name)}</strong><small>${(status.vision_projector.expected_bytes / 1_000_000_000).toFixed(2)} GB · ${escapeHtml(integrityLabel(status.vision_projector.integrity))}</small></div>
      </div>`
    : '<p class="form-note">Reading local model configuration…</p>';
  const statusText = !desktopMode
    ? 'Desktop-only · browser mode does not start native inference.'
    : busy === 'downloading'
      ? 'Downloading pinned Gemma model assets…'
      : chatBusy
        ? 'Generating locally; responses stream into the conversation…'
        : busy === 'verifying'
        ? 'Verifying file sizes, GGUF headers and SHA-256 checksums…'
        : busy === 'loading'
          ? 'Starting local llama.cpp CPU inference…'
          : busy === 'unloading'
            ? 'Stopping the local inference process…'
            : status?.last_error ?? (loaded
              ? `Loaded · ${status.execution_device} · ${status.context_size.toLocaleString()} token context`
              : status && !enoughMemory
                ? status.available_memory_mb === null
                  ? 'Loading blocked · available memory could not be measured'
                  : `Loading blocked · needs ${(memoryMinimum / 1024).toFixed(1)} GB free RAM`
              : status?.runtime_available
                ? 'Ready to load · model process is stopped'
                : 'Local llama.cpp runtime is not installed with this build.');
  return `<section class="panel native-model-manager" aria-labelledby="native-model-heading">
    <div class="section-title compact">
      <div><p class="eyebrow">NATIVE · MULTIMODAL · LOCAL</p><h2 id="native-model-heading">Gemma 3 model manager</h2></div>
      <span class="status-pill ${loaded ? 'status-available' : ''}">${loaded ? 'LOADED · CPU' : status?.phase === 'error' ? 'NEEDS ATTENTION' : 'LOCAL MODEL'}</span>
    </div>
    <p class="native-ai-privacy">Inference runs through the app-managed llama.cpp CPU process, bound to loopback only. No separate third-party model service or cloud API is used. Image bytes stay local. Model files are separate from the installer.</p>
    <label class="native-model-path">Local model folder
      <input id="native-model-dir" value="${escapeHtml(modelDir)}" maxlength="1024" ${busy || loaded ? 'disabled' : ''} />
      <small>GGUF weights and the matching projector are downloaded only after you select Download. Leave the existing model folder intact until native inference is verified.</small>
    </label>
    ${fileInfo}
    <div class="native-model-metrics">
      <div><span>Runtime</span><strong>${status?.runtime_available ? 'llama.cpp CPU available' : 'Not found'}</strong></div>
      <div><span>Memory</span><strong>${status?.available_memory_mb !== null && status?.available_memory_mb !== undefined ? `${Math.round(status.available_memory_mb / 1024 * 10) / 10} GB available · ${((status.minimum_available_memory_mb ?? 6_144) / 1024).toFixed(1)} GB minimum` : 'Not reported · loading blocked'}</strong><small>Model process working set: ${status?.model_process_memory_mb !== null && status?.model_process_memory_mb !== undefined ? `${status.model_process_memory_mb} MiB` : 'not available'} · model + vision assets ${(status?.estimated_model_memory_mb ?? 0) > 0 ? `≈ ${((status?.estimated_model_memory_mb ?? 0) / 1024).toFixed(1)} GB on disk` : 'not yet scanned'}</small></div>
      <div><span>Execution</span><strong>CPU · 1 request at a time</strong><small>${status?.total_memory_mb ? `${Math.round(status.total_memory_mb / 1024 * 10) / 10} GB physical RAM · ` : ''}${status?.context_size ?? 4096} token context · CPU utilization not sampled</small></div>
    </div>
    <p class="native-model-status ${busy || status?.phase === 'error' ? 'error' : ''}" role="status" aria-live="polite">${escapeHtml(statusText)}</p>
    ${busy ? '<div class="native-model-progress" role="progressbar" aria-label="Local model operation in progress"><span></span></div>' : ''}
    <div class="native-model-actions">
      <button class="button button-secondary" type="button" data-action="native-model-refresh" ${busy || chatBusy || !desktopMode ? 'disabled' : ''}>Refresh</button>
      <button class="button button-secondary" type="button" data-action="native-model-download" ${busy || chatBusy || !desktopMode || !missingModel ? 'disabled' : ''}>${busy === 'downloading' ? 'Downloading…' : 'Download Gemma + vision'}</button>
      <button class="button button-secondary" type="button" data-action="native-model-verify" ${busy || chatBusy || !desktopMode || missingModel ? 'disabled' : ''}>${busy === 'verifying' ? 'Verifying…' : 'Verify assets'}</button>
      <button class="button button-danger" type="button" data-action="native-model-remove" ${busy || chatBusy || !desktopMode || !removable || loaded ? 'disabled' : ''}>Remove downloaded assets</button>
      ${loaded
        ? `<button class="button button-danger" type="button" data-action="native-model-unload" ${busy || chatBusy ? 'disabled' : ''}>${busy === 'unloading' ? 'Unloading…' : 'Unload'}</button>`
        : `<button class="button button-primary" type="button" data-action="native-model-load" ${busy || chatBusy || !desktopMode || missingModel || !status?.runtime_available || !enoughMemory ? 'disabled' : ''}>${busy === 'loading' ? 'Loading…' : 'Load local model'}</button>`}
    </div>
    <small class="native-license-note">Gemma 3 is subject to Google’s Gemma Terms of Use. Check those terms before use or redistribution. Model source: ggml-org/Gemma 3 4B IT GGUF, pinned revision. Remove deletes only the two Gemma GGUF files; runtime and other files are preserved.</small>
  </section>`;
}

export function renderNativeAiPanel(input: {
  status: NativeModelStatus | null;
  busy: string;
  modelDir: string;
  messages: readonly NativeChatMessage[];
  chatBusy: boolean;
  cancelRequested?: boolean;
  metrics: {
    promptTokens: number | null;
    completionTokens: number | null;
    tokensPerSecond: number | null;
  } | null;
  image: { name: string; url: string; width: number; height: number } | null;
  desktopMode: boolean;
  tools: readonly NativeToolDefinition[];
  lastToolResult: string | null;
}): string {
  const loaded = input.status?.loaded === true;
  const tools = input.tools.length
    ? input.tools.map((tool) => `<article class="native-tool-card">
        <div><strong>${escapeHtml(tool.name)}</strong><span>${escapeHtml(tool.category)} · v${tool.version}</span></div>
        <code>${escapeHtml(tool.id)}</code>
        <p>${escapeHtml(tool.description)}</p>
        <small>Permission: ${escapeHtml(tool.permission_scope)} · ${escapeHtml(tool.availability)} · timeout ${tool.timeout_ms} ms</small>
      </article>`).join('')
    : '<p class="ai-empty-state">Tool registry is not available in this runtime.</p>';
  return `<section class="native-ai-grid">
    ${modelManager(input.status, input.busy, input.modelDir, input.desktopMode, input.chatBusy)}
    <section class="panel native-chat-panel">
      <div class="section-title compact"><div><p class="eyebrow">OFFLINE CONVERSATION</p><h2>Local Gemma chat</h2></div><span class="status-pill">${loaded ? 'READY' : 'MODEL STOPPED'}</span></div>
      <div class="native-chat-history" id="native-chat-history" aria-live="polite" aria-label="Local Gemma conversation">${renderMessages(input.messages, input.chatBusy)}</div>
      ${input.image ? `<div class="ai-image-preview"><img src="${escapeHtml(input.image.url)}" alt="Selected image preview" /><div><strong>${escapeHtml(input.image.name)}</strong><small>${input.image.width} × ${input.image.height} pixels · local image input</small></div><button class="button button-secondary" type="button" data-action="clear-ai-image">Clear image</button></div>` : ''}
      <form id="native-chat-form" class="native-chat-form">
        <label for="native-prompt">Message Gemma 3 4B</label>
        <label class="native-image-input">Attach image<input id="native-image-file" type="file" accept="image/png,image/jpeg,image/webp" /></label>
        <textarea id="native-prompt" name="prompt" maxlength="4096" rows="3" placeholder="${loaded ? 'Ask in Hindi, Hinglish or English…' : 'Load the verified local model to start chatting.'}" ${loaded && !input.chatBusy ? '' : 'disabled'} required></textarea>
        <div class="native-chat-actions"><span>${input.chatBusy ? '<span class="ai-spinner" aria-hidden="true"></span> Generating locally…' : loaded ? 'Chat is kept only for this session.' : 'The model is not loaded.'}</span><button class="button button-primary" type="submit" ${!loaded || input.chatBusy ? 'disabled' : ''}>${input.chatBusy ? 'Generating…' : 'Send'}</button></div>
      </form>
      ${input.metrics ? `<small class="native-inference-metrics">Last turn · ${input.metrics.promptTokens ?? 'n/a'} prompt tokens · ${input.metrics.completionTokens ?? 'n/a'} output tokens · ${input.metrics.tokensPerSecond !== null ? `${input.metrics.tokensPerSecond.toFixed(1)} tokens/s` : 'speed unavailable'} (approximate)</small>` : ''}
      <div class="native-chat-footer"><span>AI text and image observations are unverified. Local calculation tools validate inputs and show their method, but are not independently cross-verified or confirmed financial results.</span><span>${input.chatBusy ? `<button class="button button-danger" type="button" data-action="cancel-native-chat" ${input.cancelRequested ? 'disabled' : ''}>${input.cancelRequested ? 'Stopping…' : 'Stop & unload model'}</button>` : ''}<button class="text-button" type="button" data-action="clear-native-chat" ${input.messages.length && !input.chatBusy ? '' : 'disabled'}>Clear chat</button></span></div>
    </section>
    <section class="panel native-tool-manager">
      <div class="panel-heading"><p class="eyebrow">MODEL-INDEPENDENT · VALIDATED</p><h2>Tool registry</h2><p>Only explicitly registered, schema-validated local calculation tools are exposed. Model output never runs arbitrary code.</p></div>
      <div class="native-tool-list">${tools}</div>
      <form id="native-tool-form" class="native-tool-form">
        <label>Run a registered calculation
          <select name="tool_id">${input.tools.map((tool) => `<option value="${escapeHtml(tool.id)}">${escapeHtml(tool.name)}</option>`).join('')}</select>
        </label>
        <label>JSON arguments<textarea name="arguments" rows="5" spellcheck="false" placeholder='{"kind":"call","spot":100,"strike":100,"maturity_years":1,"risk_free_rate":0.05,"volatility":0.2}' required></textarea></label>
        <button class="button button-secondary" type="submit" ${!input.tools.length || !input.desktopMode || input.chatBusy ? 'disabled' : ''}>Validate & run</button>
      </form>
      ${input.lastToolResult ? `<pre class="ai-state-output" role="status" aria-live="polite">${escapeHtml(input.lastToolResult)}</pre>` : ''}
    </section>
  </section>`;
}
