import { Loader, Center } from '@mantine/core';
import { Navigate } from 'react-router-dom';
import { getSessionToken } from '../auth';
import { useCurrentUser } from '../hooks/useCurrentUser';

export const ProtectedRoute = ({ children }: { children: React.ReactNode }) => {
  const token = getSessionToken();
  const { isLoading, isError } = useCurrentUser();

  if (!token) {
    return <Navigate to="/login" replace />;
  }
  if (isLoading) {
    return (
      <Center py="xl">
        <Loader />
      </Center>
    );
  }
  if (isError) {
    return <Navigate to="/login" replace />;
  }
  return <>{children}</>;
};
