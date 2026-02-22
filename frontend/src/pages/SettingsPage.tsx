import { Alert, Button, NumberInput, Stack, Switch } from '@mantine/core';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { api } from '../api/client';
import { ErrorState, LoadingState } from '../components/ui/AsyncState';
import { PageLayout, SectionCard } from '../components/ui/PageLayout';

export const SettingsPage = () => {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const { data, isLoading, isError } = useQuery({
    queryKey: ['settings'],
    queryFn: api.settings,
  });
  const [draft, setDraft] = useState<{
    email_enabled: boolean;
    digest_hour_utc: number;
  } | null>(null);
  const mutation = useMutation({
    mutationFn: api.saveSettings,
  });

  if (isLoading) return <LoadingState message="Loading settings..." />;
  if (isError || !data) return <ErrorState message="Could not load settings." />;
  const emailEnabled = draft?.email_enabled ?? data?.email_enabled ?? true;
  const hour = draft?.digest_hour_utc ?? data?.digest_hour_utc ?? 12;

  return (
    <PageLayout title="Settings" description="Manage reminders and account session controls.">
      {mutation.isError ? <Alert color="red">Failed to save settings.</Alert> : null}
      {mutation.isSuccess ? <Alert color="green">Settings saved.</Alert> : null}
      <SectionCard title="Reminder settings">
        <Stack>
          <Switch
            label="Enable email reminders"
            checked={emailEnabled}
            onChange={(event) =>
              setDraft({
                email_enabled: event.currentTarget.checked,
                digest_hour_utc: hour,
              })
            }
          />
          <NumberInput
            label="Digest hour UTC"
            value={hour}
            onChange={(value) =>
              setDraft({
                email_enabled: emailEnabled,
                digest_hour_utc: Number(value ?? 12),
              })
            }
            min={0}
            max={23}
          />
          <Button
            onClick={() =>
              mutation.mutate({ email_enabled: emailEnabled, digest_hour_utc: hour })
            }
          >
            Save
          </Button>
        </Stack>
      </SectionCard>
      <Button
        variant="outline"
        color="red"
        onClick={async () => {
          await api.logout().catch(() => undefined);
          queryClient.clear();
          navigate('/login', { replace: true });
        }}
      >
        Logout
      </Button>
    </PageLayout>
  );
};
