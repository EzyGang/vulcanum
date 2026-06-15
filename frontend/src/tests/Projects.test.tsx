import { signal } from '@preact/signals';
import { fireEvent, render } from '@testing-library/preact';
import { describe, expect, it, vi } from 'vitest';
import { ProjectsView } from '../components/projects/ui/Projects.view';

const makeProject = (overrides = {}) => ({
  id: '1',
  externalProjectId: 'test-project-1',
  name: '',
  externalWorkspaceId: '',
  enabled: true,
  pickupColumn: 'to-do',
  targetColumn: 'in-review',
  progressColumn: 'in-progress',
  promptTemplate: 'Review {{task_title}}',
  repoUrl: 'https://github.com/test/repo',
  agentsMd: '',
  opencodeConfig: '',
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
        extra={{ canCreateProject: true, projectSetupWarning: '' }}
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
        extra={{ canCreateProject: true, projectSetupWarning: '' }}
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
        extra={{ canCreateProject: true, projectSetupWarning: '' }}
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
        extra={{ canCreateProject: true, projectSetupWarning: '' }}
        actions={{ onEditClick, onConnectProject, onConfirmDelete, onCancelDelete, onDelete }}
      />
    );

    expect(getByText('Server error')).toBeDefined();
  });

  it('renders projects in the table', () => {
    const projects = [
      makeProject({ id: '1', externalProjectId: 'proj-a' }),
      makeProject({ id: '2', externalProjectId: 'proj-b', enabled: false })
    ];

    const { getByText } = render(
      <ProjectsView
        data={{ projects, deleteConfirmId, deleteError }}
        status={{ loading: false, error: null }}
        extra={{ canCreateProject: true, projectSetupWarning: '' }}
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
        extra={{ canCreateProject: true, projectSetupWarning: '' }}
        actions={{ onEditClick, onConnectProject, onConfirmDelete, onCancelDelete, onDelete }}
      />
    );

    expect(getByText('to-do → in-progress → in-review')).toBeDefined();
  });

  it('shows delete confirmation when confirming', () => {
    deleteConfirmId.value = '1';
    const projects = [makeProject({ id: '1' })];

    const { getByLabelText } = render(
      <ProjectsView
        data={{ projects, deleteConfirmId, deleteError }}
        status={{ loading: false, error: null }}
        extra={{ canCreateProject: true, projectSetupWarning: '' }}
        actions={{ onEditClick, onConnectProject, onConfirmDelete, onCancelDelete, onDelete }}
      />
    );

    expect(getByLabelText('Confirm delete')).toBeDefined();
  });

  it('calls onDelete when confirm delete is clicked', () => {
    deleteConfirmId.value = '1';
    const projects = [makeProject({ id: '1' })];

    const { getByLabelText } = render(
      <ProjectsView
        data={{ projects, deleteConfirmId, deleteError }}
        status={{ loading: false, error: null }}
        extra={{ canCreateProject: true, projectSetupWarning: '' }}
        actions={{ onEditClick, onConnectProject, onConfirmDelete, onCancelDelete, onDelete }}
      />
    );

    expect(getByLabelText('Confirm delete')).toBeDefined();
    fireEvent.click(getByLabelText('Confirm delete'));
    expect(onDelete).toHaveBeenCalledWith('1');
  });

  it('disables connect project and shows setup warning when requirements are missing', () => {
    const warning =
      'Create at least one task tracker provider. Connect at least one model provider.';
    const { getByText } = render(
      <ProjectsView
        data={{ projects: [], deleteConfirmId, deleteError }}
        status={{ loading: false, error: null }}
        extra={{ canCreateProject: false, projectSetupWarning: warning }}
        actions={{ onEditClick, onConnectProject, onConfirmDelete, onCancelDelete, onDelete }}
      />
    );

    expect((getByText('Connect Project') as HTMLButtonElement).disabled).toBe(true);
    expect(getByText(warning)).toBeDefined();
  });
});
