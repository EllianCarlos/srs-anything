import { screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { IntegrationsPage } from './IntegrationsPage';
import { renderWithProviders } from '../test/renderWithProviders';

describe('IntegrationsPage', () => {
  it('shows session token setup guidance and no regeneration placeholder', async () => {
    const fetchMock = vi.spyOn(globalThis, 'fetch').mockResolvedValue(
      new Response(
        JSON.stringify({
          session_token_setup: [
            'Use the real app session token (not a placeholder)',
            'Set the same value in LeetCode/NeetCode localStorage.srs_session_token',
          ],
          latest_event: null,
          checklist: ['Install Tampermonkey extension'],
        }),
        {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        },
      ),
    );

    renderWithProviders(<IntegrationsPage />);

    expect(await screen.findByRole('heading', { name: 'Integrations' })).toBeInTheDocument();
    expect(await screen.findByText('Session token setup')).toBeInTheDocument();
    expect(
      await screen.findByText(/Use the real app session token \(not a placeholder\)/i),
    ).toBeInTheDocument();
    expect(screen.queryByText(/Regenerate token/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/Ingestion token hint/i)).not.toBeInTheDocument();

    fetchMock.mockRestore();
  });
});
