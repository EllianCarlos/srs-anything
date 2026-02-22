import { getSessionToken } from '../auth';
import type { Dashboard, Integrations, ProblemCard, ReviewEvent, Settings, User } from './types';

const API_BASE = import.meta.env.VITE_API_BASE_URL ?? 'http://localhost:3000';

type JsonValue = Record<string, unknown>;

const headers = (): HeadersInit => {
  const token = getSessionToken();
  const base: Record<string, string> = {
    'Content-Type': 'application/json',
  };
  if (token) {
    base.Authorization = `Bearer ${token}`;
  }
  return base;
};

const request = async <T>(path: string, init?: RequestInit): Promise<T> => {
  const response = await fetch(`${API_BASE}${path}`, {
    ...init,
    headers: headers(),
  });
  if (!response.ok) {
    const text = await response.text();
    throw new Error(text || `HTTP ${response.status}`);
  }
  if (response.status === 204) {
    return undefined as T;
  }
  return (await response.json()) as T;
};

export const api = {
  requestMagicLink: (email: string) =>
    request<{ sent: boolean; dev_magic_token: string }>('/auth/request-magic-link', {
      method: 'POST',
      body: JSON.stringify({ email } as JsonValue),
    }),
  verifyMagicLink: (token: string) =>
    request<{ session_token: string; user: User }>('/auth/verify-magic-link', {
      method: 'POST',
      body: JSON.stringify({ token } as JsonValue),
    }),
  me: () => request<User>('/me'),
  logout: () => request<void>('/auth/logout', { method: 'POST' }),
  dashboard: () => request<Dashboard>('/dashboard'),
  dueReviews: () => request<ProblemCard[]>('/reviews/due'),
  gradeCard: (cardId: number, grade: 'again' | 'hard' | 'good' | 'easy') =>
    request<ReviewEvent>(`/reviews/${cardId}/grade`, {
      method: 'POST',
      body: JSON.stringify({ grade } as JsonValue),
    }),
  history: () => request<ReviewEvent[]>('/history'),
  settings: () => request<Settings>('/settings'),
  saveSettings: (payload: Pick<Settings, 'email_enabled' | 'digest_hour_utc'>) =>
    request<Settings>('/settings', {
      method: 'POST',
      body: JSON.stringify(payload),
    }),
  integrations: () => request<Integrations>('/integrations'),
};
