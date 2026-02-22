export type User = {
  id: number;
  email: string;
  created_at: string;
};

export type ProblemEvent = {
  id: number;
  user_id: number;
  source: string;
  problem_slug: string;
  title: string;
  url: string;
  status: 'solved' | 'unsolved';
  occurred_at: string;
};

export type ProblemCard = {
  id: number;
  user_id: number;
  source: string;
  problem_slug: string;
  title: string;
  url: string;
  interval_index: number;
  next_due_at: string;
};

export type ReviewEvent = {
  id: number;
  card_id: number;
  user_id: number;
  grade: 'again' | 'hard' | 'good' | 'easy';
  reviewed_at: string;
  next_due_at: string;
};

export type Dashboard = {
  due_count: number;
  upcoming_count: number;
  leetcode_count: number;
  neetcode_count: number;
  latest_ingestion: ProblemEvent | null;
};

export type Settings = {
  user_id: number;
  email_enabled: boolean;
  digest_hour_utc: number;
};

export type Integrations = {
  session_token_setup: string[];
  latest_event: ProblemEvent | null;
  checklist: string[];
};
