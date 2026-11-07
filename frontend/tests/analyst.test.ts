import assert from 'node:assert/strict';
import test from 'node:test';
import { renderAnalystPanel } from '../src/pages/analyst.ts';
import type { LocalAnalysisReport } from '../src/types.ts';

const report: LocalAnalysisReport = {
  schema_version: 1,
  mode: 'local_deterministic',
  intent: 'dataset_summary',
  selected_instrument: '<script>alert(1)</script>',
  data_class: 'SYNTHETIC / TEST',
  answer_text: '<img src=x onerror=alert(1)>',
  claims: [{
    id: 'claim-1',
    text: 'A calculated claim.',
    kind: 'calculation',
    evidence_ids: ['period_return_pct'],
    counter_evidence_ids: [],
    assumptions: [],
    uncertainty: 'Historical only.',
  }],
  evidence: [{
    id: 'period_return_pct',
    kind: 'calculation',
    label: 'Return',
    value: 1.25,
    unit: 'percent',
    period_start_utc: 1_800_000_000,
    period_end_utc: 1_800_000_060,
    source_id: 'user_provided_csv',
    source_name: 'SYNTHETIC / TEST',
    as_of_utc: 1_800_000_060,
  }],
  citations: [],
  assumptions: ['Only local CSV data was used.'],
  uncertainty: 'No future inference.',
  missing_data: [],
  validation: { valid: true, warnings: [] },
  audit: {
    analyst_version: 'local-analyst-v1',
    context_fingerprint: 'fnv1a64:1234',
    output_fingerprint: 'fnv1a64:5678',
    calculation_versions: [],
    persisted: false,
  },
  disclaimer: 'Not financial advice.',
};

test('analyst panel escapes untrusted question, data labels, and answer text', () => {
  const html = renderAnalystPanel(report, '<svg onload=alert(1)>', false, true, null);
  assert.ok(html.includes('&lt;script&gt;alert(1)&lt;/script&gt;'));
  assert.ok(html.includes('&lt;img src=x onerror=alert(1)&gt;'));
  assert.ok(html.includes('&lt;svg onload=alert(1)&gt;'));
  assert.ok(!html.includes('<svg onload='));
  assert.ok(html.includes('No remote model was used.'));
});

test('analyst panel explains disabled capability and keeps no-report state empty', () => {
  const html = renderAnalystPanel(null, '', false, false, 'Enable CSV import first.');
  assert.ok(html.includes('Enable CSV import first.'));
  assert.ok(!html.includes('Evidence and provenance'));
});
