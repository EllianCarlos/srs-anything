import { useQuery } from '@tanstack/react-query';
import { api } from '../api/client';

export const useCurrentUser = () =>
  useQuery({
    queryKey: ['me'],
    queryFn: api.me,
    retry: false,
  });
