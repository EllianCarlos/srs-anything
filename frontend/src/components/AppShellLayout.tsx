import { AppShell, Badge, Burger, Group, NavLink, Stack, Text } from '@mantine/core';
import { useDisclosure } from '@mantine/hooks';
import { Link, useLocation } from 'react-router-dom';

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
