import { Group, SimpleGrid, Text } from '@mantine/core';
import { useQuery } from '@tanstack/react-query';
import { api } from '../api/client';
import { EmptyState, ErrorState, LoadingState } from '../components/ui/AsyncState';
import { PageLayout, SectionCard } from '../components/ui/PageLayout';
import { hasDueReviews, sourceBreakdown } from '../domain/dashboard';

export const DashboardPage = () => {
  const { data, isLoading, isError } = useQuery({
    queryKey: ['dashboard'],
    queryFn: api.dashboard,
  });

  if (isLoading) return <LoadingState message="Loading dashboard..." />;
  if (isError || !data) return <ErrorState message="Could not load dashboard." />;

  return (
    <PageLayout title="Dashboard" description="Your daily spaced-repetition snapshot.">
      <SimpleGrid cols={{ base: 1, md: 3 }}>
        <SectionCard title="Due today">
          <Text size="xl" fw={700}>
            {data.due_count}
          </Text>
        </SectionCard>
        <SectionCard title="Upcoming">
          <Text size="xl" fw={700}>
            {data.upcoming_count}
          </Text>
        </SectionCard>
        <SectionCard title="Streak">
          <Text size="xl" fw={700}>
            0
          </Text>
        </SectionCard>
      </SimpleGrid>

      <SectionCard title="Source split">
        <Group gap="md">
          {sourceBreakdown(data).map((item) => (
            <Text key={item.source} size="sm">
              {item.source}: {item.total}
            </Text>
          ))}
        </Group>
      </SectionCard>

      {!hasDueReviews(data) ? <EmptyState message="No reviews due right now. Check integrations or upcoming cards." /> : null}

      <SectionCard title="Latest ingestion">
        <Text c="dimmed">
          {data.latest_ingestion
            ? `${data.latest_ingestion.source} - ${data.latest_ingestion.title}`
            : 'No events yet'}
        </Text>
      </SectionCard>
    </PageLayout>
  );
};
