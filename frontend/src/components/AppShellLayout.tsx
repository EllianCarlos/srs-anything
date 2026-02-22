import { AppShell, Badge, Burger, Button, Group, NavLink, Stack, Text } from '@mantine/core';
import { useDisclosure } from '@mantine/hooks';
import { useQueryClient } from '@tanstack/react-query';
import { Link, useLocation, useNavigate } from 'react-router-dom';
import { api } from '../api/client';

type LinkItem = { to: string; label: string };

const links: LinkItem[] = [
  { to: '/', label: 'Dashboard' },
  { to: '/review', label: 'Review' },
  { to: '/history', label: 'History' },
  { to: '/integrations', label: 'Integrations' },
  { to: '/settings', label: 'Settings' },
];

export const AppShellLayout = ({
  children,
  dueCount,
}: {
  children: React.ReactNode;
  dueCount: number;
}) => {
  const [opened, { toggle }] = useDisclosure();
  const location = useLocation();
  const navigate = useNavigate();
  const queryClient = useQueryClient();

  return (
    <AppShell
      header={{ height: 64 }}
      navbar={{ width: 280, breakpoint: 'sm', collapsed: { mobile: !opened } }}
      padding="md"
    >
      <AppShell.Header>
        <Group h="100%" px="lg" justify="space-between">
          <Group>
            <Burger opened={opened} onClick={toggle} hiddenFrom="sm" size="sm" />
            <Text fw={700}>SRS Anything</Text>
          </Group>
          <Badge color="blue" variant="filled">
            Due {dueCount}
          </Badge>
          <Button
            variant="subtle"
            color="red"
            size="xs"
            onClick={async () => {
              await api.logout().catch(() => undefined);
              queryClient.clear();
              navigate('/login', { replace: true });
            }}
          >
            Logout
          </Button>
        </Group>
      </AppShell.Header>
      <AppShell.Navbar p="sm">
        <Stack gap={4}>
          <Text size="xs" c="dimmed" tt="uppercase" fw={700} px="xs">
            Navigation
          </Text>
          {links.map((item) => (
            <NavLink
              key={item.to}
              component={Link}
              to={item.to}
              label={item.label}
              active={location.pathname === item.to}
              variant="light"
            />
          ))}
        </Stack>
      </AppShell.Navbar>
      <AppShell.Main>{children}</AppShell.Main>
    </AppShell>
  );
};
