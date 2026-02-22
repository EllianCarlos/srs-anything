import type { ReviewEvent } from '../api/types';

type HistoryFilters = {
  grade?: ReviewEvent['grade'];
};

export const applyHistoryFilters = (
  events: ReviewEvent[],
  filters: HistoryFilters,
): ReviewEvent[] =>
  events.filter((event) => (filters.grade ? event.grade === filters.grade : true));
