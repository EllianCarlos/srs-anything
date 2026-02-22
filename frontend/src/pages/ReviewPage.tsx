import { Button, Group, Text } from '@mantine/core';
import { useMutation, useQuery } from '@tanstack/react-query';
import { useEffect, useMemo, useReducer } from 'react';
import { api } from '../api/client';
import { EmptyState, ErrorState, LoadingState } from '../components/ui/AsyncState';
import { PageLayout, SectionCard } from '../components/ui/PageLayout';
import { currentCard, initialReviewState, reviewReducer } from '../domain/review';

const grades: Array<'again' | 'hard' | 'good' | 'easy'> = ['again', 'hard', 'good', 'easy'];

export const ReviewPage = () => {
  const [state, dispatch] = useReducer(reviewReducer, initialReviewState);
  const dueQuery = useQuery({
    queryKey: ['due'],
    queryFn: api.dueReviews,
  });
  const gradeMutation = useMutation({
    mutationFn: ({ cardId, grade }: { cardId: number; grade: (typeof grades)[number] }) =>
      api.gradeCard(cardId, grade),
    onSuccess: () => dispatch({ type: 'next' }),
  });

  const queue = useMemo(
    () => (state.queue.length > 0 ? state.queue : dueQuery.data ?? []),
    [state.queue, dueQuery.data],
  );
  useEffect(() => {
    if (state.queue.length === 0 && queue.length > 0) {
      dispatch({ type: 'load', payload: queue });
    }
  }, [queue, state.queue.length]);

  const card = currentCard({ ...state, queue });
  if (dueQuery.isLoading) return <LoadingState message="Loading review queue..." />;
  if (dueQuery.isError) return <ErrorState message="Failed to load due reviews." />;
  if (!card) return <EmptyState message="No cards due. You are done for now." />;

  return (
    <PageLayout title="Review" description="Work through your due cards and grade each response.">
      <SectionCard title={card.title} description={card.source}>
        <Text size="sm">
          Progress: {Math.min(state.index + 1, queue.length)}/{queue.length}
        </Text>
        <Button component="a" href={card.url} target="_blank" mt="sm">
          Open problem
        </Button>
      </SectionCard>
      <Group wrap="wrap">
        {grades.map((grade) => (
          <Button
            key={grade}
            variant={grade === 'again' ? 'outline' : 'filled'}
            onClick={() => gradeMutation.mutate({ cardId: card.id, grade })}
            loading={gradeMutation.isPending}
          >
            {grade}
          </Button>
        ))}
      </Group>
    </PageLayout>
  );
};
