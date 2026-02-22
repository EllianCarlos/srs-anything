import { fireEvent, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { LoginPage } from './LoginPage';
import { renderWithProviders } from '../test/renderWithProviders';

describe('LoginPage', () => {
  it('renders form', () => {
    renderWithProviders(<LoginPage />);
    expect(screen.getByRole('heading', { name: 'Login' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /send magic link/i })).toBeInTheDocument();
  });

  it('submits email', async () => {
    const fetchMock = vi.spyOn(globalThis, 'fetch').mockResolvedValue(
      new Response(JSON.stringify({ sent: true, dev_magic_token: 'token123' }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }),
    );
    renderWithProviders(<LoginPage />);
    fireEvent.change(screen.getByPlaceholderText('you@example.com'), {
      target: { value: 'a@b.com' },
    });
    fireEvent.click(screen.getByRole('button', { name: /send magic link/i }));
    expect(await screen.findByText(/Token generated/i)).toBeInTheDocument();
    fetchMock.mockRestore();
  });
});
