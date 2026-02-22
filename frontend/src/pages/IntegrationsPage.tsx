import { Alert, Button, Group, List, Stack, Table, Text, TextInput } from '@mantine/core';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { api } from '../api/client';
import { ErrorState, LoadingState } from '../components/ui/AsyncState';
import { PageLayout, SectionCard } from '../components/ui/PageLayout';

const isLocalDev = (): boolean =>
  window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1';

const postTokenToTampermonkey = (token: string): void => {
  const payload = {
    source: 'srs-anything',
    type: 'SRS_API_TOKEN_CREATED',
    token,
  };
  if (isLocalDev()) {
    console.info('[srs-anything] sending token to tampermonkey bridge', {
      origin: window.location.origin,
      tokenPreview: `${token.slice(0, 8)}...`,
      tokenLength: token.length,
    });
  }
  window.postMessage(payload, window.location.origin);
};

export const IntegrationsPage = () => {
  const [label, setLabel] = useState('Default token');
  const queryClient = useQueryClient();
  const { data, isLoading, isError } = useQuery({
    queryKey: ['integrations'],
    queryFn: api.integrations,
  });
  const createTokenMutation = useMutation({
    mutationFn: api.createIntegrationToken,
    onSuccess: (payload) => {
      postTokenToTampermonkey(payload.token);
      queryClient.invalidateQueries({ queryKey: ['integrations'] }).catch(() => undefined);
    },
  });
  const revokeTokenMutation = useMutation({
    mutationFn: api.revokeIntegrationToken,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['integrations'] }).catch(() => undefined);
    },
  });

  if (isLoading) return <LoadingState message="Loading integrations..." />;
  if (isError || !data) return <ErrorState message="Could not load integrations." />;
  const apiTokenSetup = data.api_token_setup ?? [
    'Create an API token from this page.',
    'Use Send to Tampermonkey to hand off token to userscript storage.',
    'Tampermonkey will use X-API-Key for ingestion.',
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
      <SectionCard title="API token setup">
        <List spacing="xs" type="ordered">
          {apiTokenSetup.map((line) => (
            <List.Item key={line}>{line}</List.Item>
          ))}
        </List>
      </SectionCard>
      <SectionCard title="API token manager">
        <Stack>
          <TextInput label="Token label" value={label} onChange={(event) => setLabel(event.currentTarget.value)} />
          <Group>
            <Button
              onClick={() => createTokenMutation.mutate({ label, expires_in_days: 365 })}
              loading={createTokenMutation.isPending}
            >
              Create token
            </Button>
            <Button
              variant="light"
              onClick={() => {
                const token = createTokenMutation.data?.token;
                if (!token) return;
                postTokenToTampermonkey(token);
              }}
              disabled={!createTokenMutation.data?.token}
            >
              Send to Tampermonkey
            </Button>
          </Group>
          {createTokenMutation.isError ? <Alert color="red">Failed to create API token.</Alert> : null}
          {createTokenMutation.isSuccess ? (
            <Alert color="green">API token created and sent to Tampermonkey bridge.</Alert>
          ) : null}
          <Table>
            <Table.Thead>
              <Table.Tr>
                <Table.Th>Label</Table.Th>
                <Table.Th>Scope</Table.Th>
                <Table.Th>Status</Table.Th>
                <Table.Th />
              </Table.Tr>
            </Table.Thead>
            <Table.Tbody>
              {data.tokens.map((token) => (
                <Table.Tr key={token.id}>
                  <Table.Td>{token.label}</Table.Td>
                  <Table.Td>{token.scopes.join(', ')}</Table.Td>
                  <Table.Td>{token.revoked_at ? 'revoked' : 'active'}</Table.Td>
                  <Table.Td>
                    <Button
                      size="xs"
                      color="red"
                      variant="subtle"
                      disabled={Boolean(token.revoked_at)}
                      loading={revokeTokenMutation.isPending}
                      onClick={() => revokeTokenMutation.mutate(token.id)}
                    >
                      Revoke
                    </Button>
                  </Table.Td>
                </Table.Tr>
              ))}
            </Table.Tbody>
          </Table>
        </Stack>
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
