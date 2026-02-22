import { fireEvent, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { IntegrationsPage } from './IntegrationsPage';
import { renderWithProviders } from '../test/renderWithProviders';

describe('IntegrationsPage', () => {
  it('shows api token setup guidance and supports tampermonkey bridge message', async () => {
    const fetchMock = vi.spyOn(globalThis, 'fetch').mockResolvedValue(
      new Response(
        JSON.stringify({
          api_token_setup: [
            'Create an API token from this page.',
            'Use Send to Tampermonkey to hand off token to userscript storage.',
          ],
          latest_event: null,
          checklist: ['Install Tampermonkey extension'],
          tokens: [],
        }),
        {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        },
      ),
    );

    renderWithProviders(<IntegrationsPage />);

    expect(await screen.findByRole('heading', { name: 'Integrations' })).toBeInTheDocument();
    expect(await screen.findByText('API token setup')).toBeInTheDocument();
    expect(
      await screen.findByText(/Create an API token from this page/i),
    ).toBeInTheDocument();

    fetchMock.mockRestore();
  });

  it('emits bridge payload after token creation', async () => {
    const postMessageSpy = vi.spyOn(window, 'postMessage').mockImplementation(() => undefined);
    const fetchMock = vi
      .spyOn(globalThis, 'fetch')
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            api_token_setup: ['Create token'],
            latest_event: null,
            checklist: ['Install Tampermonkey extension'],
            tokens: [],
          }),
          { status: 200, headers: { 'Content-Type': 'application/json' } },
        ),
      )
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            token: 'srs_it_abc123abc123abc123abc123abc123abc123abc123abc',
            token_summary: {
              id: 1,
              label: 'Default token',
              scopes: ['events:write'],
              created_at: '2026-01-01T00:00:00Z',
              expires_at: null,
              revoked_at: null,
              last_used_at: null,
            },
          }),
          { status: 200, headers: { 'Content-Type': 'application/json' } },
        ),
      )
      .mockResolvedValue(
        new Response(
          JSON.stringify({
            api_token_setup: ['Create token'],
            latest_event: null,
            checklist: ['Install Tampermonkey extension'],
            tokens: [],
          }),
          { status: 200, headers: { 'Content-Type': 'application/json' } },
        ),
      );

    renderWithProviders(<IntegrationsPage />);
    fireEvent.click(await screen.findByRole('button', { name: /create token/i }));
    await waitFor(() => {
      expect(postMessageSpy).toHaveBeenCalledWith(
        expect.objectContaining({
          source: 'srs-anything',
          type: 'SRS_API_TOKEN_CREATED',
        }),
        window.location.origin,
      );
    });

    fetchMock.mockRestore();
    postMessageSpy.mockRestore();
  });
});
