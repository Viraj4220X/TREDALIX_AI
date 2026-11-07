import { Channel, invoke, isTauri } from '@tauri-apps/api/core';

export interface NativeModelFileStatus {
  file_name: string;
  expected_bytes: number;
  actual_bytes: number;
  present: boolean;
  integrity: string;
}

export interface NativeModelStatus {
  model_name: string;
  model_dir: string;
  engine: string;
  phase: 'unloaded' | 'loading' | 'ready' | 'error';
  model: NativeModelFileStatus;
  vision_projector: NativeModelFileStatus;
  runtime_available: boolean;
  loaded: boolean;
  execution_device: string;
  cpu_fallback: boolean;
  context_size: number;
  estimated_model_memory_mb: number;
  total_memory_mb: number | null;
  available_memory_mb: number | null;
  model_process_memory_mb: number | null;
  minimum_available_memory_mb: number;
  last_error: string | null;
}

export interface NativeChatMessage {
  role: 'user' | 'assistant';
  content: string;
}

export interface NativeChatReply extends NativeChatMessage {
  model: string;
  prompt_tokens: number | null;
  completion_tokens: number | null;
  tokens_per_second: number | null;
  tool_calls: string[];
  verified_financial_result: boolean;
}

export interface NativeChatEvent {
  event: 'delta';
  content: string;
}

export interface NativeToolDefinition {
  id: string;
  name: string;
  description: string;
  category: string;
  version: number;
  permission_scope: string;
  input_schema: Record<string, unknown>;
  output_schema: Record<string, unknown>;
  timeout_ms: number;
  availability: string;
}

export interface NativeToolExecution {
  tool_id: string;
  result: Record<string, unknown>;
  result_validated: boolean;
  independently_verified: boolean;
  method: string;
  elapsed_ms: number;
}

function requireDesktop(): void {
  if (!isTauri()) {
    throw new Error('The native Gemma model manager is available in the Windows desktop app.');
  }
}

export async function readNativeModelStatus(modelDir?: string): Promise<NativeModelStatus> {
  requireDesktop();
  return invoke<NativeModelStatus>('native_model_status', { modelDir: modelDir || null });
}

export async function downloadNativeModel(modelDir: string): Promise<NativeModelStatus> {
  requireDesktop();
  return invoke<NativeModelStatus>('download_native_model', { modelDir: modelDir || null });
}

export async function removeNativeModel(modelDir: string): Promise<NativeModelStatus> {
  requireDesktop();
  return invoke<NativeModelStatus>('remove_native_model', { modelDir: modelDir || null });
}

export async function verifyNativeModel(modelDir: string): Promise<NativeModelStatus> {
  requireDesktop();
  return invoke<NativeModelStatus>('verify_native_model', { modelDir: modelDir || null });
}

export async function loadNativeModel(modelDir: string): Promise<NativeModelStatus> {
  requireDesktop();
  return invoke<NativeModelStatus>('load_native_model', { modelDir: modelDir || null });
}

export async function unloadNativeModel(): Promise<void> {
  requireDesktop();
  await invoke<void>('unload_native_model');
}

export async function sendNativeChat(
  messages: readonly NativeChatMessage[],
  image?: Uint8Array,
  onDelta?: (content: string) => void,
): Promise<NativeChatReply> {
  requireDesktop();
  const onEvent = new Channel<NativeChatEvent>();
  onEvent.onmessage = (event) => {
    if (event.event === 'delta') onDelta?.(event.content);
  };
  return invoke<NativeChatReply>('native_model_chat', {
    messages,
    image: image ? Array.from(image) : null,
    onEvent,
  });
}

export async function readNativeToolRegistry(): Promise<NativeToolDefinition[]> {
  requireDesktop();
  return invoke<NativeToolDefinition[]>('list_ai_tools');
}

export async function executeNativeTool(
  id: string,
  argumentsValue: Record<string, unknown>,
): Promise<NativeToolExecution> {
  requireDesktop();
  return invoke<NativeToolExecution>('execute_ai_tool', {
    id,
    arguments: argumentsValue,
  });
}
