import { Select, Table } from '@mantine/core';
import { useMemo, useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { api } from '../api/client';
import { EmptyState, ErrorState, LoadingState } from '../components/ui/AsyncState';
import { PageLayout, SectionCard } from '../components/ui/PageLayout';
import { applyHistoryFilters } from '../domain/history';

export const HistoryPage = () => {
  const [grade, setGrade] = useState<string | null>(null);
  const { data, isLoading, isError } = useQuery({
    queryKey: ['history'],
    queryFn: api.history,
  });

  const filtered = useMemo(
    () => applyHistoryFilters(data ?? [], { grade: grade as never }),
    [data, grade],
  );

  if (isLoading) return <LoadingState message="Loading history..." />;
  if (isError || !data) return <ErrorState message="Could not load history." />;

  return (
    <PageLayout title="History" description="Review logs for graded cards and upcoming due dates.">
      <SectionCard title="Filters">
        <Select
          label="Grade filter"
          value={grade}
          onChange={setGrade}
          data={[
            { value: 'again', label: 'Again' },
            { value: 'hard', label: 'Hard' },
            { value: 'good', label: 'Good' },
            { value: 'easy', label: 'Easy' },
          ]}
          clearable
        />
      </SectionCard>
      {filtered.length === 0 ? (
        <EmptyState message="No reviews found with the selected filter." />
      ) : (
        <Table.ScrollContainer minWidth={760}>
          <Table>
            <Table.Thead>
              <Table.Tr>
                <Table.Th>Card</Table.Th>
                <Table.Th>Grade</Table.Th>
                <Table.Th>Reviewed at</Table.Th>
                <Table.Th>Next due</Table.Th>
              </Table.Tr>
            </Table.Thead>
            <Table.Tbody>
              {filtered.map((entry) => (
                <Table.Tr key={entry.id}>
                  <Table.Td>{entry.card_id}</Table.Td>
                  <Table.Td>{entry.grade}</Table.Td>
                  <Table.Td>{new Date(entry.reviewed_at).toLocaleString()}</Table.Td>
                  <Table.Td>{new Date(entry.next_due_at).toLocaleString()}</Table.Td>
                </Table.Tr>
              ))}
            </Table.Tbody>
          </Table>
        </Table.ScrollContainer>
      )}
    </PageLayout>
  );
};
