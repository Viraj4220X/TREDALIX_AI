import { featureRegistry, featureBlockedReason, getFeature } from './features.ts';
import type { AppSettings, Theme } from '../types.ts';

export const SETTINGS_KEY = 'tredalix.settings.v1';
export const REPORTS_KEY = 'tredalix.reports.v1';
export const SETTINGS_SCHEMA_VERSION = 1;
export const BACKTEST_REQUIRED_FEATURES = [
  'csv_import',
  'data_validation',
  'technical_indicators',
  'signal_generation',
  'backtesting',
] as const;

export interface SettingsLoadResult {
  settings: AppSettings;
  warning: string | null;
}

export function defaultSettings(): AppSettings {
  return {
    schemaVersion: SETTINGS_SCHEMA_VERSION,
    theme: 'dark',
    features: Object.fromEntries(
      featureRegistry.map((feature) => [
        feature.id,
        { enabled: feature.defaultEnabled },
      ]),
    ),
  };
}

export function decodeSettings(raw: string | null): SettingsLoadResult {
  if (raw === null) return { settings: defaultSettings(), warning: null };
  try {
    const parsed = JSON.parse(raw) as Partial<AppSettings>;
    if (parsed.schemaVersion !== SETTINGS_SCHEMA_VERSION) {
      return {
        settings: defaultSettings(),
        warning: 'Saved settings use an unsupported version; safe defaults are active.',
      };
    }
    if (parsed.theme !== 'dark' && parsed.theme !== 'light') {
      throw new Error('Invalid saved theme');
    }
    if (!parsed.features || typeof parsed.features !== 'object') {
      throw new Error('Invalid saved feature settings');
    }
    const defaults = defaultSettings();
    const features = { ...defaults.features };
    for (const feature of featureRegistry) {
      const saved = parsed.features[feature.id];
      if (saved && typeof saved.enabled === 'boolean') {
        features[feature.id] = {
          enabled:
            feature.status === 'available' || feature.status === 'experimental'
              ? saved.enabled
              : false,
        };
      }
    }
    return {
      settings: { schemaVersion: SETTINGS_SCHEMA_VERSION, theme: parsed.theme, features },
      warning: null,
    };
  } catch {
    return {
      settings: defaultSettings(),
      warning: 'Saved settings could not be read; safe defaults are active. The stored value was not changed.',
    };
  }
}

export function setTheme(settings: AppSettings, theme: Theme): AppSettings {
  return { ...settings, theme };
}

export function setFeature(
  settings: AppSettings,
  featureId: string,
  enabled: boolean,
): AppSettings {
  const feature = getFeature(featureId);
  if (enabled) {
    const blockedReason = featureBlockedReason(featureId, settings.features);
    if (blockedReason) throw new Error(blockedReason);
  }
  return {
    ...settings,
    features: {
      ...settings.features,
      [featureId]: { enabled },
    },
  };
}

export function requireFeatures(
  settings: AppSettings,
  featureIds: readonly string[],
): void {
  for (const featureId of featureIds) {
    const feature = getFeature(featureId);
    if (!settings.features[featureId]?.enabled) {
      throw new Error(`${feature.name} is disabled in Feature Manager.`);
    }
    const blockedReason = featureBlockedReason(featureId, settings.features);
    if (blockedReason) throw new Error(blockedReason);
  }
}

export function canRunBacktest(
  settings: AppSettings,
  hasValidatedDataset: boolean,
): boolean {
  return (
    hasValidatedDataset &&
    BACKTEST_REQUIRED_FEATURES.every(
      (featureId) =>
        settings.features[featureId]?.enabled &&
        featureBlockedReason(featureId, settings.features) === null,
    )
  );
}

export function saveSettings(settings: AppSettings, storage: Storage): void {
  storage.setItem(SETTINGS_KEY, JSON.stringify(settings));
}

export function loadSettings(storage: Storage): SettingsLoadResult {
  return decodeSettings(storage.getItem(SETTINGS_KEY));
}

export function resetSettings(storage: Storage): AppSettings {
  const settings = defaultSettings();
  saveSettings(settings, storage);
  return settings;
}
