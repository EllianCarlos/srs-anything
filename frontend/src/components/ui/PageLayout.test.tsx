import { screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { EmptyState, ErrorState, LoadingState } from './AsyncState';
import { PageLayout, SectionCard } from './PageLayout';
import { renderWithProviders } from '../../test/renderWithProviders';

describe('PageLayout', () => {
  it('renders title, description, actions, and content', () => {
    renderWithProviders(
      <PageLayout title="Dashboard" description="Summary" actions={<button type="button">Action</button>}>
        <div>Body content</div>
      </PageLayout>,
    );

    expect(screen.getByText('Dashboard')).toBeInTheDocument();
    expect(screen.getByText('Summary')).toBeInTheDocument();
    expect(screen.getByText('Action')).toBeInTheDocument();
    expect(screen.getByText('Body content')).toBeInTheDocument();
  });
});

describe('SectionCard', () => {
  it('renders section metadata and child content', () => {
    renderWithProviders(
      <SectionCard title="Section title" description="Section description">
        <div>Section body</div>
      </SectionCard>,
    );

    expect(screen.getByText('Section title')).toBeInTheDocument();
    expect(screen.getByText('Section description')).toBeInTheDocument();
    expect(screen.getByText('Section body')).toBeInTheDocument();
  });
});

describe('AsyncState helpers', () => {
  it('renders loading message', () => {
    renderWithProviders(<LoadingState message="Loading data..." />);
    expect(screen.getByText('Loading data...')).toBeInTheDocument();
  });

  it('renders error and empty messages', () => {
    renderWithProviders(
      <>
        <ErrorState message="Error happened" />
        <EmptyState message="Nothing to show" />
      </>,
    );

    expect(screen.getByText('Error happened')).toBeInTheDocument();
    expect(screen.getByText('Nothing to show')).toBeInTheDocument();
  });
});
