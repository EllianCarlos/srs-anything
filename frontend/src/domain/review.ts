import type { ProblemCard } from '../api/types';

export type ReviewState = {
  queue: ProblemCard[];
  index: number;
};

export type ReviewAction =
  | { type: 'load'; payload: ProblemCard[] }
  | { type: 'next' };

export const initialReviewState: ReviewState = {
  queue: [],
  index: 0,
};

export const reviewReducer = (state: ReviewState, action: ReviewAction): ReviewState => {
  switch (action.type) {
    case 'load':
      return {
        queue: action.payload,
        index: 0,
      };
    case 'next':
      return {
        ...state,
        index: Math.min(state.index + 1, state.queue.length),
      };
    default:
      return state;
  }
};

export const currentCard = (state: ReviewState): ProblemCard | null =>
  state.queue[state.index] ?? null;
