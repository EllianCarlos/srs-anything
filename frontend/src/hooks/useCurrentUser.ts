import { useQuery } from '@tanstack/react-query';
import { api } from '../api/client';
import { getSessionToken } from '../auth';

export const useCurrentUser = () =>
  useQuery({
    queryKey: ['me'],
    queryFn: api.me,
    enabled: Boolean(getSessionToken()),
    retry: false,
  });
