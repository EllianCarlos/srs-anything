import type { Dashboard } from '../api/types';

export const sourceBreakdown = (dashboard: Dashboard): Array<{ source: string; total: number }> => [
  { source: 'leetcode', total: dashboard.leetcode_count },
  { source: 'neetcode', total: dashboard.neetcode_count },
];

export const hasDueReviews = (dashboard: Dashboard): boolean => dashboard.due_count > 0;
