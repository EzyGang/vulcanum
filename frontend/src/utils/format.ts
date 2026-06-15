const COMPACT_NUMBER_FORMATTER = new Intl.NumberFormat('en', {
  maximumFractionDigits: 1,
  notation: 'compact'
});

const DATE_TIME_FORMATTER = new Intl.DateTimeFormat('en', {
  dateStyle: 'medium',
  timeStyle: 'short'
});

export const formatRelativeTime = (dateStr: string | null): string => {
  if (!dateStr) return '—';
  const diff = Date.now() - new Date(dateStr).getTime();
  const seconds = Math.floor(diff / 1000);

  if (seconds < 60) return 'Just now';

  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) {
    return new Intl.RelativeTimeFormat('en', { style: 'long' }).format(-minutes, 'minute');
  }

  const hours = Math.floor(minutes / 60);
  if (hours < 24) {
    return new Intl.RelativeTimeFormat('en', { style: 'long' }).format(-hours, 'hour');
  }

  const days = Math.floor(hours / 24);
  return new Intl.RelativeTimeFormat('en', { style: 'long' }).format(-days, 'day');
};

export const formatDuration = (ms: number): string => {
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60_000) return `${(ms / 1000).toFixed(1)}s`;
  if (ms < 3_600_000) {
    const m = Math.floor(ms / 60_000);
    const s = Math.floor((ms % 60_000) / 1000);
    return `${m}m ${s}s`;
  }
  const h = Math.floor(ms / 3_600_000);
  const m = Math.floor((ms % 3_600_000) / 60_000);
  return `${h}h ${m}m`;
};

export const formatTokenCount = (tokens: number | null | undefined): string => {
  if (tokens === null || tokens === undefined) return '—';

  return COMPACT_NUMBER_FORMATTER.format(tokens);
};

export const formatDateTime = (dateStr: string | null | undefined): string => {
  if (!dateStr) return '—';

  const date = new Date(dateStr);
  if (Number.isNaN(date.getTime())) return '—';

  return DATE_TIME_FORMATTER.format(date);
};
