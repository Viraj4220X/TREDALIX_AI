import { escapeHtml } from '../components/layout.ts';
import type { LocalAnalysisReport } from '../types.ts';

function formatTimestamp(timestamp: number): string {
  const milliseconds = timestamp * 1_000;
  return Number.isSafeInteger(milliseconds)
    ? new Date(milliseconds).toISOString()
    : 'Invalid timestamp';
}

function renderReport(report: LocalAnalysisReport): string {
  return `<section class="panel analyst-result" aria-live="polite">
    <div class="section-title compact">
      <div><p class="eyebrow">LOCAL DETERMINISTIC ANALYSIS</p><h3>${escapeHtml(report.selected_instrument)} · ${escapeHtml(report.data_class)}</h3></div>
      <button class="button button-secondary" type="button" data-action="clear-analyst">Clear this analysis</button>
    </div>
    <p class="analyst-answer">${escapeHtml(report.answer_text)}</p>
    <p class="form-note">${escapeHtml(report.disclaimer)} No remote model was used.</p>
    ${report.claims.length ? `<h4>Supported claims</h4><ul class="limitations-list">${report.claims
      .map((claim) => `<li>${escapeHtml(claim.text)} <small>Evidence: ${claim.evidence_ids.map(escapeHtml).join(', ')}</small></li>`)
      .join('')}</ul>` : ''}
    <details class="analyst-evidence"><summary>Evidence and provenance (${report.evidence.length})</summary>
      <div class="table-wrap"><table><thead><tr><th>Evidence</th><th>Value</th><th>Unit / method</th><th>Source label · as of (UTC)</th></tr></thead><tbody>${report.evidence
        .map((item) => `<tr><td>${escapeHtml(item.label)}</td><td>${Number.isFinite(item.value) ? item.value.toFixed(4) : 'Unavailable'}</td><td>${escapeHtml(item.unit)}</td><td>${escapeHtml(item.source_name)} · ${escapeHtml(formatTimestamp(item.as_of_utc))}</td></tr>`)
        .join('')}</tbody></table></div>
      <p class="table-caption">Source label was supplied with the CSV and is not independently verified. Price currency and adjustment basis are not provided by this analysis.</p>
    </details>
    ${report.missing_data.length ? `<h4>Data gaps</h4><ul class="limitations-list">${report.missing_data.map((item) => `<li>${escapeHtml(item)}</li>`).join('')}</ul>` : ''}
    <details><summary>Assumptions, uncertainty, and audit fingerprint</summary>
      <ul class="limitations-list">${report.assumptions.map((item) => `<li>${escapeHtml(item)}</li>`).join('')}</ul>
      <p class="form-note">${escapeHtml(report.uncertainty)}</p>
      <p class="table-caption">Audit record is held in session memory only; no question or dataset is persisted. Fingerprints use non-cryptographic FNV-1a and are for correlation, not tamper detection.</p>
      <code>${escapeHtml(report.audit.context_fingerprint)}</code>
    </details>
  </section>`;
}

export function renderAnalystPanel(
  report: LocalAnalysisReport | null,
  question: string,
  busy: boolean,
  enabled: boolean,
  blockedReason: string | null,
): string {
  return `<section class="panel analyst-panel">
    <div class="panel-heading"><p class="eyebrow">OFFLINE AI MIND FOUNDATION</p><h2>Ask about this dataset</h2>
      <p>Structured candle perception and deterministic Rust analysis. No API key, remote LLM, provider data, or hidden memory is used.</p></div>
    <form id="analyst-form">
      <label for="analyst-question">Question about the validated CSV</label>
      <textarea id="analyst-question" name="question" maxlength="500" rows="3" required ${busy || !enabled ? 'disabled' : ''}>${escapeHtml(question)}</textarea>
      <button class="button button-primary" type="submit" ${busy || !enabled ? 'disabled' : ''}>${busy ? 'Analyzing locally…' : 'Analyze with local Rust engine'} <span aria-hidden="true">→</span></button>
      <p class="form-note">${blockedReason ? escapeHtml(blockedReason) : 'The local analyst can summarize the selected CSV, calculate its return/drawdown/indicators, and explain its data gaps. Forecasts, causal explanations, company research, and buy/sell instructions are not generated.'}</p>
    </form>
    ${report ? renderReport(report) : ''}
  </section>`;
}
