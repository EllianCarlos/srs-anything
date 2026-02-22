import { Alert, Center, Loader, Stack, Text } from '@mantine/core';

export const LoadingState = ({ message }: { message: string }) => (
  <Center py="xl">
    <Stack gap="sm" align="center">
      <Loader size="sm" />
      <Text c="dimmed" size="sm">
        {message}
      </Text>
    </Stack>
  </Center>
);

export const ErrorState = ({ message }: { message: string }) => <Alert color="red">{message}</Alert>;

export const EmptyState = ({ message }: { message: string }) => <Alert color="blue">{message}</Alert>;
