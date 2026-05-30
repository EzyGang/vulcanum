import { signal } from '@preact/signals';
import { fireEvent, render } from '@testing-library/preact';
import { describe, expect, it, vi } from 'vitest';
import { ProjectsView } from '../components/projects/ui/Projects.view';

const makeProject = (overrides = {}) => ({
  id: '1',
  kaneoProjectId: 'test-project-1',
  kaneoWorkspaceId: '',
  enabled: true,
  pickupColumn: 'to-do',
  targetColumn: 'in-review',
  progressColumn: 'in-progress',
  promptTemplate: 'Review {{task_title}}',
  repoUrl: 'https://github.com/test/repo',
  agentsMd: '',
  createdAt: '2026-01-01T00:00:00Z',
  ...overrides
});

describe('Projects.view', () => {
  const deleteConfirmId = signal<string | null>(null);
  const deleteError = signal<string | null>(null);
  const onEditClick = vi.fn();
  const onConnectProject = vi.fn();
  const onConfirmDelete = vi.fn();
  const onCancelDelete = vi.fn();
  const onDelete = vi.fn();

  it('renders the connect project button', () => {
    const { getByText } = render(
      <ProjectsView
        data={{ projects: [], deleteConfirmId, deleteError }}
        status={{ loading: false, error: null }}
        extra={{ hasProviders: true }}
        actions={{ onEditClick, onConnectProject, onConfirmDelete, onCancelDelete, onDelete }}
      />
    );

    expect(getByText('Connect Project')).toBeDefined();
  });

  it('shows empty state when no projects', () => {
    const { getByText } = render(
      <ProjectsView
        data={{ projects: [], deleteConfirmId, deleteError }}
        status={{ loading: false, error: null }}
        extra={{ hasProviders: true }}
        actions={{ onEditClick, onConnectProject, onConfirmDelete, onCancelDelete, onDelete }}
      />
    );

    expect(getByText('No project configs configured yet.')).toBeDefined();
  });

  it('shows loading text when loading', () => {
    const { getByText } = render(
      <ProjectsView
        data={{ projects: [], deleteConfirmId, deleteError }}
        status={{ loading: true, error: null }}
        extra={{ hasProviders: true }}
        actions={{ onEditClick, onConnectProject, onConfirmDelete, onCancelDelete, onDelete }}
      />
    );

    expect(getByText('Loading projects...')).toBeDefined();
  });

  it('shows error message when error is set', () => {
    const error = {
      name: 'ApiError',
      message: 'Server error',
      status: 500,
      serverError: 'Server error'
    };

    const { getByText } = render(
      <ProjectsView
        data={{ projects: [], deleteConfirmId, deleteError }}
        status={{ loading: false, error }}
        extra={{ hasProviders: true }}
        actions={{ onEditClick, onConnectProject, onConfirmDelete, onCancelDelete, onDelete }}
      />
    );

    expect(getByText('Server error')).toBeDefined();
  });

  it('renders projects in the table', () => {
    const projects = [
      makeProject({ id: '1', kaneoProjectId: 'proj-a' }),
      makeProject({ id: '2', kaneoProjectId: 'proj-b', enabled: false })
    ];

    const { getByText } = render(
      <ProjectsView
        data={{ projects, deleteConfirmId, deleteError }}
        status={{ loading: false, error: null }}
        extra={{ hasProviders: true }}
        actions={{ onEditClick, onConnectProject, onConfirmDelete, onCancelDelete, onDelete }}
      />
    );

    expect(getByText('proj-a')).toBeDefined();
    expect(getByText('proj-b')).toBeDefined();
  });

  it('shows column triad', () => {
    const projects = [makeProject()];

    const { getByText } = render(
      <ProjectsView
        data={{ projects, deleteConfirmId, deleteError }}
        status={{ loading: false, error: null }}
        extra={{ hasProviders: true }}
        actions={{ onEditClick, onConnectProject, onConfirmDelete, onCancelDelete, onDelete }}
      />
    );

    expect(getByText('to-do → in-progress → in-review')).toBeDefined();
  });

  it('shows delete confirmation when confirming', () => {
    deleteConfirmId.value = '1';
    const projects = [makeProject({ id: '1' })];

    const { getByText } = render(
      <ProjectsView
        data={{ projects, deleteConfirmId, deleteError }}
        status={{ loading: false, error: null }}
        extra={{ hasProviders: true }}
        actions={{ onEditClick, onConnectProject, onConfirmDelete, onCancelDelete, onDelete }}
      />
    );

    expect(getByText('Confirm?')).toBeDefined();
  });

  it('calls onDelete when confirm delete is clicked', () => {
    deleteConfirmId.value = '1';
    const projects = [makeProject({ id: '1' })];

    const { getByText } = render(
      <ProjectsView
        data={{ projects, deleteConfirmId, deleteError }}
        status={{ loading: false, error: null }}
        extra={{ hasProviders: true }}
        actions={{ onEditClick, onConnectProject, onConfirmDelete, onCancelDelete, onDelete }}
      />
    );

    expect(getByText('Confirm?')).toBeDefined();
    fireEvent.click(getByText('Delete'));
    expect(onDelete).toHaveBeenCalledWith('1');
  });
});
