import { describe, expect, it } from 'vitest';
import { applyHistoryFilters } from './history';

const entries = [
  {
    id: 1,
    card_id: 1,
    user_id: 1,
    grade: 'good' as const,
    reviewed_at: new Date().toISOString(),
    next_due_at: new Date().toISOString(),
  },
  {
    id: 2,
    card_id: 2,
    user_id: 1,
    grade: 'again' as const,
    reviewed_at: new Date().toISOString(),
    next_due_at: new Date().toISOString(),
  },
];

describe('applyHistoryFilters', () => {
  it('filters by grade', () => {
    expect(applyHistoryFilters(entries, { grade: 'good' })).toHaveLength(1);
    expect(applyHistoryFilters(entries, { grade: 'again' })[0].id).toBe(2);
  });
});
