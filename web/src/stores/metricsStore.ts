// Placeholder metrics store
import type { MetricSnapshot, MetricAlert } from '@/types/generated';

export const useMetricsStore = () => {
  return {
    metrics: [] as MetricSnapshot[],
    history: [] as MetricSnapshot[],
    alerts: [] as MetricAlert[],
    hasAlerts: false,
    subscribe: (callback: Function) => {
      // Placeholder
    }
  };
};

export type { MetricSnapshot, MetricAlert };
