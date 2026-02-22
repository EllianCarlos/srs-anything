import { Card, Container, Group, Stack, Text, Title } from '@mantine/core';

type PageLayoutProps = {
  title: string;
  description?: string;
  actions?: React.ReactNode;
  children: React.ReactNode;
};

type SectionCardProps = {
  title?: string;
  description?: string;
  children: React.ReactNode;
};

export const PageLayout = ({ title, description, actions, children }: PageLayoutProps) => (
  <Container size="lg" px={0}>
    <Stack gap="lg">
      <Group justify="space-between" align="end" wrap="wrap">
        <Stack gap={4}>
          <Title order={2}>{title}</Title>
          {description ? (
            <Text c="dimmed" size="sm">
              {description}
            </Text>
          ) : null}
        </Stack>
        {actions}
      </Group>
      {children}
    </Stack>
  </Container>
);

export const SectionCard = ({ title, description, children }: SectionCardProps) => (
  <Card>
    <Stack gap="sm">
      {title ? <Text fw={600}>{title}</Text> : null}
      {description ? (
        <Text c="dimmed" size="sm">
          {description}
        </Text>
      ) : null}
      {children}
    </Stack>
  </Card>
);
