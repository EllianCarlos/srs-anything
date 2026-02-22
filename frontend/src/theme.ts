import { Alert, Button, Card, NavLink, Table, Title, createTheme } from '@mantine/core';

export const appTheme = createTheme({
  primaryColor: 'blue',
  primaryShade: { light: 6, dark: 8 },
  defaultRadius: 'md',
  fontFamily:
    'Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
  headings: {
    fontFamily:
      'Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
    fontWeight: '700',
  },
  colors: {
    blue: [
      '#eef4ff',
      '#dce8ff',
      '#c3d7ff',
      '#9fc0ff',
      '#78a6ff',
      '#5c92ff',
      '#3f7fff',
      '#2f69e6',
      '#2553b8',
      '#1f468f',
    ],
    gray: [
      '#f8f9fb',
      '#f1f3f7',
      '#e7eaf0',
      '#d7dde7',
      '#c5cdd9',
      '#aeb8c7',
      '#8d99ad',
      '#6b778d',
      '#4f5a70',
      '#3d4659',
    ],
  },
  components: {
    Card: Card.extend({
      defaultProps: {
        withBorder: true,
        padding: 'lg',
      },
    }),
    Button: Button.extend({
      defaultProps: {
        radius: 'md',
      },
    }),
    Title: Title.extend({
      defaultProps: {
        fw: 700,
      },
    }),
    Alert: Alert.extend({
      defaultProps: {
        radius: 'md',
        variant: 'light',
      },
    }),
    NavLink: NavLink.extend({
      defaultProps: {
        variant: 'subtle',
      },
    }),
    Table: Table.extend({
      defaultProps: {
        striped: true,
        highlightOnHover: true,
        withTableBorder: true,
        verticalSpacing: 'sm',
      },
    }),
  },
});
