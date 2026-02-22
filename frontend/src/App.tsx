import { useQuery } from '@tanstack/react-query';
import { Navigate, Route, Routes } from 'react-router-dom';
import { AppShellLayout } from './components/AppShellLayout';
import { ProtectedRoute } from './components/ProtectedRoute';
import { api } from './api/client';
import { DashboardPage } from './pages/DashboardPage';
import { HistoryPage } from './pages/HistoryPage';
import { IntegrationsPage } from './pages/IntegrationsPage';
import { LoginPage } from './pages/LoginPage';
import { ReviewPage } from './pages/ReviewPage';
import { SettingsPage } from './pages/SettingsPage';
import { VerifyPage } from './pages/VerifyPage';

const ProtectedApp = ({ children }: { children: React.ReactNode }) => {
  const { data } = useQuery({
    queryKey: ['dashboard-shell'],
    queryFn: api.dashboard,
    retry: false,
  });
  return <AppShellLayout dueCount={data?.due_count ?? 0}>{children}</AppShellLayout>;
};

export const App = () => (
  <Routes>
    <Route path="/login" element={<LoginPage />} />
    <Route path="/verify" element={<VerifyPage />} />
    <Route
      path="/"
      element={
        <ProtectedRoute>
          <ProtectedApp>
            <DashboardPage />
          </ProtectedApp>
        </ProtectedRoute>
      }
    />
    <Route
      path="/review"
      element={
        <ProtectedRoute>
          <ProtectedApp>
            <ReviewPage />
          </ProtectedApp>
        </ProtectedRoute>
      }
    />
    <Route
      path="/history"
      element={
        <ProtectedRoute>
          <ProtectedApp>
            <HistoryPage />
          </ProtectedApp>
        </ProtectedRoute>
      }
    />
    <Route
      path="/settings"
      element={
        <ProtectedRoute>
          <ProtectedApp>
            <SettingsPage />
          </ProtectedApp>
        </ProtectedRoute>
      }
    />
    <Route
      path="/integrations"
      element={
        <ProtectedRoute>
          <ProtectedApp>
            <IntegrationsPage />
          </ProtectedApp>
        </ProtectedRoute>
      }
    />
    <Route path="*" element={<Navigate to="/" replace />} />
  </Routes>
);
