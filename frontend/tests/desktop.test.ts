import assert from 'node:assert/strict';
import test from 'node:test';
import { renderSettings } from '../src/pages/settings.ts';
import { openExternalHttpUrl, probeDesktopRuntime } from '../src/services/desktop-runtime.ts';
import { defaultSettings } from '../src/settings/storage.ts';

test('browser mode remains independent from the desktop IPC host', async () => {
  const runtime = await probeDesktopRuntime();
  assert.deepEqual(runtime, { mode: 'browser' });
  const html = renderSettings(defaultSettings(), runtime);
  assert.match(html, /Browser mode/);
  assert.match(html, /No desktop IPC service or localhost server is required/);
  assert.doesNotMatch(html, /Desktop service ready/);
});

test('external link helper rejects unsafe schemes before opening anything', async () => {
  await assert.rejects(
    openExternalHttpUrl('javascript:alert(1)'),
    /Only HTTP and HTTPS links/,
  );
  await assert.rejects(
    openExternalHttpUrl('file:///C:/Users/test/private.csv'),
    /Only HTTP and HTTPS links/,
  );
  await assert.rejects(
    openExternalHttpUrl('https://example.invalid'),
    /only in the desktop app/,
  );
});

test('desktop capability UI exposes no port, token or environment values', () => {
  const html = renderSettings(defaultSettings(), {
    mode: 'ready',
    diagnostics: {
      status: 'ready',
      app_version: '0.1.0',
      api_version: 1,
      transport: 'tauri_ipc',
      build_mode: 'release',
      capabilities: ['local_csv_validation', 'offline_deterministic_analyst'],
    },
  });
  assert.match(html, /Desktop service ready/);
  assert.match(html, /no HTTP listener is opened/);
  assert.doesNotMatch(html, /127\.0\.0\.1|auth.?token|secret|process\.env/i);
});
