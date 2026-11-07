declare module '*tredalix_wasm.js' {
  export default function initialize(): Promise<unknown>;
  export function inspect_csv(
    csv: string,
    datasetOptions: string,
    csvImportEnabled: boolean,
    dataValidationEnabled: boolean,
  ): string;
  export function run_backtest(
    csv: string,
    datasetOptions: string,
    runOptions: string,
    featureFlags: string,
  ): string;
}

declare module '*.css';

interface ImportMeta {
  readonly env: {
    readonly BASE_URL: string;
  };
}
