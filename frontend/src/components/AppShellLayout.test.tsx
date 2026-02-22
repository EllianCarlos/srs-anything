import { fireEvent, render, screen } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { describe, expect, it, vi } from 'vitest';
import { MemoryRouter } from 'react-router-dom';
import { MantineProvider } from '@mantine/core';
import { AppShellLayout } from './AppShellLayout';
import { appTheme } from '../theme';

describe('AppShellLayout', () => {
  it('renders header logout action', () => {
    render(
      <MantineProvider defaultColorScheme="light" theme={appTheme}>
        <QueryClientProvider client={new QueryClient()}>
          <MemoryRouter>
            <AppShellLayout dueCount={3}>
              <div>content</div>
            </AppShellLayout>
          </MemoryRouter>
        </QueryClientProvider>
      </MantineProvider>,
    );
    expect(screen.getByRole('button', { name: /logout/i })).toBeInTheDocument();
  });

  it('calls logout endpoint when logout is clicked', async () => {
    const fetchMock = vi.spyOn(globalThis, 'fetch').mockResolvedValue(
      new Response(null, { status: 204 }),
    );

    render(
      <MantineProvider defaultColorScheme="light" theme={appTheme}>
        <QueryClientProvider client={new QueryClient()}>
          <MemoryRouter>
            <AppShellLayout dueCount={1}>
              <div>content</div>
            </AppShellLayout>
          </MemoryRouter>
        </QueryClientProvider>
      </MantineProvider>,
    );

    fireEvent.click(screen.getByRole('button', { name: /logout/i }));

    expect(fetchMock).toHaveBeenCalledWith(
      expect.stringMatching(/\/auth\/logout$/),
      expect.objectContaining({
        credentials: 'include',
        method: 'POST',
      }),
    );
    fetchMock.mockRestore();
  });
});
