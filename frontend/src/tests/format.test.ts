import { beforeEach, describe, expect, it, vi } from 'vitest';
import { formatDateTime, formatDuration, formatRelativeTime } from '../utils/format';

describe('formatRelativeTime', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2026-05-23T12:00:00Z'));
  });

  it('returns "Just now" for less than 60 seconds ago', () => {
    const date = new Date('2026-05-23T11:59:30Z').toISOString();
    expect(formatRelativeTime(date)).toBe('Just now');
  });

  it('returns minutes for less than 60 minutes', () => {
    const date = new Date('2026-05-23T11:55:00Z').toISOString();
    expect(formatRelativeTime(date)).toMatch(/minutes ago/);
  });

  it('returns hours for less than 24 hours', () => {
    const date = new Date('2026-05-23T10:00:00Z').toISOString();
    expect(formatRelativeTime(date)).toMatch(/hours ago/);
  });

  it('returns days for more than 24 hours', () => {
    const date = new Date('2026-05-22T12:00:00Z').toISOString();
    expect(formatRelativeTime(date)).toMatch(/day ago/);
  });
});

describe('formatDuration', () => {
  it('formats milliseconds', () => {
    expect(formatDuration(500)).toBe('500ms');
  });

  it('formats seconds with one decimal', () => {
    expect(formatDuration(1500)).toBe('1.5s');
  });

  it('formats minutes and seconds', () => {
    expect(formatDuration(125_000)).toBe('2m 5s');
  });

  it('formats hours and minutes', () => {
    expect(formatDuration(7_260_000)).toBe('2h 1m');
  });

  it('formats zero', () => {
    expect(formatDuration(0)).toBe('0ms');
  });
});

describe('formatDateTime', () => {
  it('formats timestamps for human reading', () => {
    expect(formatDateTime('2026-05-23T12:30:00Z')).toMatch(/May 23, 2026/);
  });

  it('returns a dash for invalid timestamps', () => {
    expect(formatDateTime('not-a-date')).toBe('—');
  });
});
