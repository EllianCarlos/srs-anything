import { Loader, Center } from '@mantine/core';
import { Navigate } from 'react-router-dom';
import { useCurrentUser } from '../hooks/useCurrentUser';

export const ProtectedRoute = ({ children }: { children: React.ReactNode }) => {
  const { isLoading, isError } = useCurrentUser();

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
