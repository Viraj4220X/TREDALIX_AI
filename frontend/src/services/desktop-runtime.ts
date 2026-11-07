import { invoke, isTauri } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import type { DesktopDiagnostics } from '../types.ts';

export type DesktopRuntimeState =
  | { mode: 'browser' }
  | { mode: 'starting' }
  | { mode: 'ready'; diagnostics: DesktopDiagnostics }
  | { mode: 'unavailable'; message: string };

const RETRY_DELAYS_MS = [0, 250, 750];

export function isDesktopApp(): boolean {
  return isTauri();
}

export async function readDesktopDiagnostics(): Promise<DesktopDiagnostics> {
  if (!isTauri()) {
    throw new Error('Desktop diagnostics are available only in the TREDALIX desktop app.');
  }
  return invoke<DesktopDiagnostics>('get_app_status');
}

export async function probeDesktopRuntime(): Promise<DesktopRuntimeState> {
  if (!isTauri()) return { mode: 'browser' };
  for (const delay of RETRY_DELAYS_MS) {
    if (delay > 0) await new Promise((resolve) => window.setTimeout(resolve, delay));
    if (!isTauri()) return { mode: 'browser' };
    try {
      const diagnostics = await readDesktopDiagnostics();
      if (diagnostics.status !== 'ready' || diagnostics.transport !== 'tauri_ipc') {
        throw new Error('The local desktop service returned an incompatible status.');
      }
      return { mode: 'ready', diagnostics };
    } catch (error) {
      if (delay === RETRY_DELAYS_MS.at(-1)) {
        return {
          mode: 'unavailable',
          message: error instanceof Error ? error.message : 'The local desktop service is unavailable.',
        };
      }
    }
  }
  return { mode: 'unavailable', message: 'The local desktop service is unavailable.' };
}

export async function openExternalHttpUrl(rawUrl: string): Promise<void> {
  let url: URL;
  try {
    url = new URL(rawUrl);
  } catch {
    throw new Error('External link is not a valid URL.');
  }
  if (url.protocol !== 'https:' && url.protocol !== 'http:') {
    throw new Error('Only HTTP and HTTPS links can be opened outside the app.');
  }
  if (!isTauri()) {
    throw new Error('System-browser links are available only in the desktop app.');
  }
  await openUrl(url.toString());
}
