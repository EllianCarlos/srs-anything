import { describe, expect, it } from 'vitest';
import { currentCard, reviewReducer } from './review';

const sampleQueue = [
  {
    id: 1,
    user_id: 1,
    source: 'leetcode',
    problem_slug: 'two-sum',
    title: 'Two Sum',
    url: 'https://leetcode.com/problems/two-sum',
    interval_index: 0,
    next_due_at: new Date().toISOString(),
  },
];

describe('reviewReducer', () => {
  it('loads queue and resets index', () => {
    const state = reviewReducer({ queue: [], index: 4 }, { type: 'load', payload: sampleQueue });
    expect(state.index).toBe(0);
    expect(state.queue).toHaveLength(1);
  });

  it('moves next without overflow', () => {
    const loaded = reviewReducer({ queue: [], index: 0 }, { type: 'load', payload: sampleQueue });
    const next = reviewReducer(loaded, { type: 'next' });
    expect(next.index).toBe(1);
  });

  it('returns current card', () => {
    const card = currentCard({ queue: sampleQueue, index: 0 });
    expect(card?.problem_slug).toBe('two-sum');
  });
});
