import { useSignal, useSignalEffect } from '@preact/signals';
import { useQuery } from '@tanstack/react-query';
import { useCallback, useRef } from 'preact/hooks';
import { useLocation } from 'wouter-preact';
import {
  createProject,
  getProject,
  listColumnsByKaneoId,
  updateProject
} from '../../../services/projects/projects.service';
import type { ColumnInfo, ProjectConfig } from '../../../types/projects';
import { invalidate } from '../../../utils/api/query/client';
import { useApiMutation } from '../../../utils/api/query/hooks';

const COLUMN_DEBOUNCE_MS = 400;

export const useProjectForm = (projectId: string | null) => {
  const [_, setLocation] = useLocation();

  const { data: existingProject, isLoading: projectLoading } = useQuery<ProjectConfig>({
    queryKey: ['project', projectId ?? ''],
    queryFn: () => getProject(projectId ?? ''),
    enabled: !!projectId
  });

  const kaneoProjectId = useSignal(projectId ? '' : '');
  const enabled = useSignal(true);
  const pickupColumn = useSignal('');
  const progressColumn = useSignal('');
  const targetColumn = useSignal('');
  const promptTemplate = useSignal('');
  const repoUrl = useSignal('');
  const agentsMd = useSignal('');
  const submitting = useSignal(false);
  const formError = useSignal<string | null>(null);
  const columns = useSignal<ColumnInfo[]>([]);
  const columnsLoading = useSignal(false);
  const columnsFetched = useSignal(false);

  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const columnKaneoId = useSignal('');

  const fetchColumns = useCallback(async (kaneoId: string) => {
    if (!kaneoId) return;
    columnsLoading.value = true;
    try {
      const result = await listColumnsByKaneoId(kaneoId);
      columns.value = result;
      columnsFetched.value = true;
    } catch {
      columns.value = [];
      columnsFetched.value = true;
    } finally {
      columnsLoading.value = false;
    }
  }, []);

  const scheduleColumnFetch = useCallback(
    (value: string) => {
      kaneoProjectId.value = value;
      columnKaneoId.value = value;
      if (debounceRef.current) {
        clearTimeout(debounceRef.current);
      }
      if (!value) {
        columns.value = [];
        columnsFetched.value = false;
        return;
      }
      columnsLoading.value = true;
      columnsFetched.value = false;
      debounceRef.current = setTimeout(() => {
        fetchColumns(value);
      }, COLUMN_DEBOUNCE_MS);
    },
    [fetchColumns]
  );

  useSignalEffect(() => {
    if (projectId && existingProject) {
      const p = existingProject;
      kaneoProjectId.value = p.kaneoProjectId;
      enabled.value = p.enabled;
      pickupColumn.value = p.pickupColumn;
      progressColumn.value = p.progressColumn;
      targetColumn.value = p.targetColumn;
      promptTemplate.value = p.promptTemplate;
      repoUrl.value = p.repoUrl;
      agentsMd.value = p.agentsMd;
    }
  });

  useSignalEffect(() => {
    if (projectId && existingProject) {
      columnKaneoId.value = existingProject.kaneoProjectId;
      fetchColumns(existingProject.kaneoProjectId);
    }
  });

  const createMutation = useApiMutation(
    (input: Parameters<typeof createProject>[0]) => createProject(input),
    {
      onSuccess: () => {
        invalidate('projects');
        setLocation('/projects');
      }
    }
  );

  const updateMutation = useApiMutation(
    ({ id, input }: { id: string; input: Parameters<typeof updateProject>[1] }) =>
      updateProject(id, input),
    {
      onSuccess: () => {
        invalidate('projects');
        setLocation('/projects');
      }
    }
  );

  const handleSubmit = useCallback(
    async (e: Event) => {
      e.preventDefault();
      formError.value = null;

      if (!kaneoProjectId.value && !projectId) {
        formError.value = 'Kaneo project ID is required';
        return;
      }

      if (!promptTemplate.value) {
        formError.value = 'Prompt template is required';
        return;
      }

      submitting.value = true;

      try {
        if (projectId) {
          await updateMutation.mutateAsync({
            id: projectId,
            input: {
              enabled: enabled.value,
              pickupColumn: pickupColumn.value || undefined,
              progressColumn: progressColumn.value || undefined,
              targetColumn: targetColumn.value || undefined,
              promptTemplate: promptTemplate.value || undefined,
              repoUrl: repoUrl.value || undefined,
              agentsMd: agentsMd.value || undefined
            }
          });
        } else {
          await createMutation.mutateAsync({
            kaneoProjectId: kaneoProjectId.value,
            enabled: enabled.value,
            pickupColumn: pickupColumn.value || undefined,
            progressColumn: progressColumn.value || undefined,
            targetColumn: targetColumn.value || undefined,
            promptTemplate: promptTemplate.value,
            repoUrl: repoUrl.value || undefined,
            agentsMd: agentsMd.value || undefined
          });
        }
      } catch (err) {
        formError.value = err instanceof Error ? err.message : 'Failed to save project config';
      } finally {
        submitting.value = false;
      }
    },
    [projectId, createMutation, updateMutation]
  );

  return {
    isEdit: !!projectId,
    projectLoading: projectId ? projectLoading : false,
    kaneoProjectId,
    enabled,
    pickupColumn,
    progressColumn,
    targetColumn,
    promptTemplate,
    repoUrl,
    agentsMd,
    submitting,
    formError,
    columns,
    columnsLoading,
    columnsFetched,
    columnKaneoId,
    handleKaneoIdChange: scheduleColumnFetch,
    handleSubmit
  };
};
