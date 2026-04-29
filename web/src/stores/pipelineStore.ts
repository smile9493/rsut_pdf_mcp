// Placeholder stores
import type { PipelineStage } from '@/types/generated';

export const usePipelineStore = () => {
  return {
    stages: [] as PipelineStage[],
    totalDuration: 0,
    confidence: 0,
    intercepted: false,
    circuitState: 'closed',
    blockingQueueDepth: 0,
    subscribe: (callback: Function) => {
      // Placeholder
    }
  };
};

export type { PipelineStage };

export const useMetricsStore = () => {
  return {
    state: {} as any,
    actions: {} as any
  };
};
