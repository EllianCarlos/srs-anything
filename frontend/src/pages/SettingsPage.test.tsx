import { fireEvent, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { SettingsPage } from './SettingsPage';
import { renderWithProviders } from '../test/renderWithProviders';

describe('SettingsPage', () => {
  it('logs out through backend endpoint', async () => {
    const fetchMock = vi
      .spyOn(globalThis, 'fetch')
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            user_id: 1,
            email_enabled: true,
            digest_hour_utc: 12,
          }),
          { status: 200, headers: { 'Content-Type': 'application/json' } },
        ),
      )
      .mockResolvedValueOnce(new Response(null, { status: 204 }));

    renderWithProviders(<SettingsPage />);

    fireEvent.click(await screen.findByRole('button', { name: /logout/i }));

    expect(fetchMock).toHaveBeenLastCalledWith(
      expect.stringMatching(/\/auth\/logout$/),
      expect.objectContaining({
        credentials: 'include',
        method: 'POST',
      }),
    );
    fetchMock.mockRestore();
  });
});
