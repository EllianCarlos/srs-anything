import { Alert, Card, Loader, Stack, Text, Title } from '@mantine/core';
import { useMutation } from '@tanstack/react-query';
import { useEffect } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { api } from '../api/client';

export const VerifyPage = () => {
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();
  const token = searchParams.get('token') ?? '';
  const mutation = useMutation({
    mutationFn: api.verifyMagicLink,
    onSuccess: () => {
      navigate('/', { replace: true });
    },
  });

  useEffect(() => {
    if (token) {
      mutation.mutate(token);
    }
  }, [token, mutation]);

  if (!token) {
    return (
      <Card maw={520} mx="auto" mt="xl" withBorder>
        <Alert color="red">Missing token. Request a new magic link.</Alert>
      </Card>
    );
  }

  return (
    <Card maw={520} mx="auto" mt="xl" withBorder>
      <Stack>
        <Title order={2}>Verifying</Title>
        {mutation.isPending ? (
          <>
            <Loader />
            <Text>Checking your magic link…</Text>
          </>
        ) : null}
        {mutation.isError ? <Alert color="red">Magic link invalid or expired.</Alert> : null}
      </Stack>
    </Card>
  );
};
