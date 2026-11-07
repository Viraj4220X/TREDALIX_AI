export type FeatureStatus =
  | 'available'
  | 'experimental'
  | 'planned'
  | 'unavailable';

export interface FeatureDefinition {
  id: string;
  name: string;
  description: string;
  status: FeatureStatus;
  platforms: readonly string[];
  dependencies: readonly string[];
  blockedReason?: string;
  defaultEnabled: boolean;
}

export const featureRegistry: readonly FeatureDefinition[] = [
  {
    id: 'csv_import',
    name: 'CSV import',
    description: 'Read a user-selected local CSV file for this session.',
    status: 'available',
    platforms: ['Web'],
    dependencies: [],
    defaultEnabled: true,
  },
  {
    id: 'data_validation',
    name: 'Data validation',
    description: 'Validate OHLCV rows and timestamps with the Rust core.',
    status: 'available',
    platforms: ['Web'],
    dependencies: ['csv_import'],
    defaultEnabled: true,
  },
  {
    id: 'technical_indicators',
    name: 'Technical indicators',
    description: 'Rust indicator overlays for local charts and the existing strategy.',
    status: 'experimental',
    platforms: ['Web'],
    dependencies: [],
    defaultEnabled: true,
  },
  {
    id: 'signal_generation',
    name: 'SMA/RSI signal strategy',
    description: 'The existing SMA-9/SMA-21/RSI-14 long-only strategy.',
    status: 'experimental',
    platforms: ['Web'],
    dependencies: ['technical_indicators'],
    defaultEnabled: true,
  },
  {
    id: 'backtesting',
    name: 'Portfolio backtesting',
    description: 'Run the existing offline Rust portfolio simulator.',
    status: 'available',
    platforms: ['Web'],
    dependencies: ['csv_import', 'data_validation', 'signal_generation'],
    defaultEnabled: true,
  },
  {
    id: 'portfolio_analysis',
    name: 'Portfolio analysis',
    description: 'View portfolio values calculated by the backtest engine.',
    status: 'available',
    platforms: ['Web'],
    dependencies: ['backtesting'],
    defaultEnabled: true,
  },
  {
    id: 'risk_metrics',
    name: 'Risk metrics',
    description: 'View drawdown and exposure values from the engine.',
    status: 'available',
    platforms: ['Web'],
    dependencies: ['backtesting'],
    defaultEnabled: true,
  },
  {
    id: 'report_export',
    name: 'Report export',
    description: 'Download the current real run as JSON and CSV reports.',
    status: 'available',
    platforms: ['Web'],
    dependencies: ['backtesting'],
    defaultEnabled: true,
  },
  {
    id: 'walk_forward',
    name: 'Repeated walk-forward',
    description: 'Expanding chronological baseline evaluation across chosen bar horizons.',
    status: 'experimental',
    platforms: ['Web'],
    dependencies: ['csv_import', 'data_validation'],
    defaultEnabled: true,
  },
  {
    id: 'naive_forecast',
    name: 'Naive random-walk baseline',
    description: 'Repeat the last validated close as an explicitly labelled baseline estimate.',
    status: 'experimental',
    platforms: ['Web'],
    dependencies: ['csv_import', 'data_validation'],
    defaultEnabled: true,
  },
  {
    id: 'prediction_models',
    name: 'Prediction models',
    description: 'Trained prediction models and evaluation workflows.',
    status: 'planned',
    platforms: [],
    dependencies: [],
    blockedReason: 'No trained model or validated prediction workflow exists.',
    defaultEnabled: false,
  },
  {
    id: 'live_market_data',
    name: 'Live market data',
    description: 'Market data from an authorized provider.',
    status: 'unavailable',
    platforms: [],
    dependencies: [],
    blockedReason: 'No provider adapter or connection lifecycle is implemented.',
    defaultEnabled: false,
  },
  {
    id: 'paper_trading',
    name: 'Paper trading',
    description: 'A separate simulated real-time order lifecycle.',
    status: 'unavailable',
    platforms: [],
    dependencies: [],
    blockedReason: 'Not implemented.',
    defaultEnabled: false,
  },
  {
    id: 'notifications',
    name: 'Notifications',
    description: 'Desktop, browser, or Android alerts.',
    status: 'planned',
    platforms: [],
    dependencies: [],
    blockedReason: 'Platform-specific notification behavior is not implemented.',
    defaultEnabled: false,
  },
  {
    id: 'local_analyst',
    name: 'Offline deterministic analyst',
    description: 'Answer evidence-backed questions about the validated local CSV using Rust calculations; no LLM or network.',
    status: 'available',
    platforms: ['Web'],
    dependencies: ['csv_import', 'data_validation'],
    defaultEnabled: true,
  },
  {
    id: 'local_ai',
    name: 'Optional local AI assistant',
    description: 'An optional local language model integration.',
    status: 'planned',
    platforms: [],
    dependencies: [],
    blockedReason: 'No local model integration is implemented.',
    defaultEnabled: false,
  },
  {
    id: 'cloud_sync',
    name: 'Cloud synchronization',
    description: 'Optional synchronization of settings or local data.',
    status: 'planned',
    platforms: [],
    dependencies: [],
    blockedReason: 'Not implemented; local operation remains the default.',
    defaultEnabled: false,
  },
] as const;

const byId = new Map(featureRegistry.map((feature) => [feature.id, feature]));

export function getFeature(featureId: string): FeatureDefinition {
  const feature = byId.get(featureId);
  if (!feature) throw new Error(`Unknown feature: ${featureId}`);
  return feature;
}

export function featureBlockedReason(
  featureId: string,
  enabled: Readonly<Record<string, { enabled: boolean }>>,
): string | null {
  const feature = getFeature(featureId);
  if (feature.status === 'planned' || feature.status === 'unavailable') {
    return feature.blockedReason ?? 'This feature is not available.';
  }
  const missing = feature.dependencies.filter(
    (dependency) => !enabled[dependency]?.enabled,
  );
  return missing.length === 0
    ? null
    : `Enable ${missing.map((dependency) => getFeature(dependency).name).join(', ')} first.`;
}
