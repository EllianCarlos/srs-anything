import { List, Text } from '@mantine/core';
import { useQuery } from '@tanstack/react-query';
import { api } from '../api/client';
import { ErrorState, LoadingState } from '../components/ui/AsyncState';
import { PageLayout, SectionCard } from '../components/ui/PageLayout';

export const IntegrationsPage = () => {
  const { data, isLoading, isError } = useQuery({
    queryKey: ['integrations'],
    queryFn: api.integrations,
  });

  if (isLoading) return <LoadingState message="Loading integrations..." />;
  if (isError || !data) return <ErrorState message="Could not load integrations." />;
  const sessionTokenSetup = data.session_token_setup ?? [
    'Use the real app session token (not a placeholder)',
    'After login + verify, copy localStorage.srs_session_token from the SRS app tab',
    'Set the same value in LeetCode/NeetCode localStorage.srs_session_token',
  ];

  return (
    <PageLayout
      title="Integrations"
      description="Connect external sources and verify ingestion details."
    >
      <SectionCard title="Tampermonkey setup">
        <List spacing="xs" type="ordered">
          {data.checklist.map((line) => (
            <List.Item key={line}>{line}</List.Item>
          ))}
        </List>
      </SectionCard>
      <SectionCard title="Session token setup">
        <List spacing="xs" type="ordered">
          {sessionTokenSetup.map((line) => (
            <List.Item key={line}>{line}</List.Item>
          ))}
        </List>
      </SectionCard>
      <SectionCard title="Latest received event">
        <Text c="dimmed">
          {data.latest_event
            ? `${data.latest_event.source}: ${data.latest_event.title}`
            : 'No events yet'}
        </Text>
      </SectionCard>
    </PageLayout>
  );
};
