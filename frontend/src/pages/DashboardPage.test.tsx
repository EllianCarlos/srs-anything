import { screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { DashboardPage } from './DashboardPage';
import { renderWithProviders } from '../test/renderWithProviders';

describe('DashboardPage', () => {
  it('renders loaded state', async () => {
    const fetchMock = vi.spyOn(globalThis, 'fetch').mockResolvedValue(
      new Response(
        JSON.stringify({
          due_count: 0,
          upcoming_count: 2,
          leetcode_count: 1,
          neetcode_count: 1,
          latest_ingestion: null,
        }),
        {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        },
      ),
    );
    renderWithProviders(<DashboardPage />);
    expect(await screen.findByText('Dashboard')).toBeInTheDocument();
    expect(await screen.findByText(/No reviews due/i)).toBeInTheDocument();
    fetchMock.mockRestore();
  });
});
