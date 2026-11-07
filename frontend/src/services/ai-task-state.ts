import type { DatasetInspection, DatasetOptions } from '../types.ts';
import {
  completeAiTask as completeTask,
  createAiTask,
  retainAiTask,
  type AiTask,
  type AiTaskKind,
} from './ai-control.ts';

export interface AiTaskState {
  aiTasks: AiTask[];
  activeAiTask: AiTask | null;
  inspection: DatasetInspection | null;
  datasetOptions: DatasetOptions;
}

export function startAiTask(
  state: AiTaskState,
  kind: AiTaskKind,
  stage: string,
  message: string,
  source?: string,
  selectedDataset?: string,
): void {
  const task = {
    ...createAiTask(kind, stage, message, source, selectedDataset ?? state.inspection?.instrument),
    ...(state.inspection ? { dataTimestamp: state.inspection.last_timestamp } : {}),
  };
  state.activeAiTask = task;
  state.aiTasks = retainAiTask(state.aiTasks, task);
}

export function finishAiTask(
  state: AiTaskState,
  status: 'succeeded' | 'failed' | 'blocked',
  message: string,
  errorCategory?: AiTask['errorCategory'],
  resultReference?: string,
): void {
  if (!state.activeAiTask) return;
  const completed = {
    ...completeTask(state.activeAiTask, status, message, errorCategory),
    ...(resultReference ? { resultReference } : {}),
  };
  state.aiTasks = retainAiTask(state.aiTasks, completed);
  state.activeAiTask = null;
}
