const SESSION_STORAGE_KEY = 'srs_session_token';

export const getSessionToken = (): string => localStorage.getItem(SESSION_STORAGE_KEY) ?? '';

export const setSessionToken = (value: string): void => {
  localStorage.setItem(SESSION_STORAGE_KEY, value);
};

export const clearSessionToken = (): void => {
  localStorage.removeItem(SESSION_STORAGE_KEY);
};
