import { describe, expect, it } from 'vitest';
import { hasDueReviews, sourceBreakdown } from './dashboard';

const dashboard = {
  due_count: 2,
  upcoming_count: 4,
  leetcode_count: 3,
  neetcode_count: 1,
  latest_ingestion: null,
};

describe('dashboard selectors', () => {
  it('reports due state', () => {
    expect(hasDueReviews(dashboard)).toBe(true);
    expect(hasDueReviews({ ...dashboard, due_count: 0 })).toBe(false);
  });

  it('returns source breakdown', () => {
    const rows = sourceBreakdown(dashboard);
    expect(rows).toEqual([
      { source: 'leetcode', total: 3 },
      { source: 'neetcode', total: 1 },
    ]);
  });
});
