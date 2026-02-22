import { Alert, Button, Card, Stack, TextInput, Title } from '@mantine/core';
import { useMutation } from '@tanstack/react-query';
import { useState } from 'react';
import type { FormEvent } from 'react';
import { useNavigate } from 'react-router-dom';
import { api } from '../api/client';

export const LoginPage = () => {
  const [email, setEmail] = useState('');
  const [devToken, setDevToken] = useState('');
  const navigate = useNavigate();
  const mutation = useMutation({
    mutationFn: api.requestMagicLink,
    onSuccess: (response) => {
      setDevToken(response.dev_magic_token);
    },
  });

  const onSubmit = (event: FormEvent) => {
    event.preventDefault();
    mutation.mutate(email);
  };

  return (
    <Card maw={520} mx="auto" mt="xl" withBorder>
      <form onSubmit={onSubmit}>
        <Stack>
          <Title order={2}>Login</Title>
          <TextInput
            label="Email"
            value={email}
            onChange={(event) => setEmail(event.currentTarget.value)}
            placeholder="you@example.com"
            required
          />
          <Button type="submit" loading={mutation.isPending}>
            Send magic link
          </Button>
          {mutation.isError ? <Alert color="red">Could not send link.</Alert> : null}
          {devToken ? (
            <Alert color="blue" title="MVP token">
              Token generated. Use it on verify page for local flow.
              <br />
              <code>{devToken}</code>
              <br />
              <Button mt="sm" size="xs" onClick={() => navigate(`/verify?token=${devToken}`)}>
                Continue to verify
              </Button>
            </Alert>
          ) : null}
        </Stack>
      </form>
    </Card>
  );
};
