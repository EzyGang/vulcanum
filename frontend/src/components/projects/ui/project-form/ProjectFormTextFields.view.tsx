import type { JSX } from 'preact';
import { Input } from '../../../shared/ui/Input.view';
import { Label } from '../../../shared/ui/Label.view';
import { Select } from '../../../shared/ui/Select.view';
import { TextArea } from '../../../shared/ui/TextArea.view';
import { useProjectFormContext } from '../../context/ProjectFormContext';

export const ProjectFormTextFields = (): JSX.Element => {
  const { data: d, status, actions: a } = useProjectFormContext();

  return (
    <>
      <div class='flex flex-col gap-2'>
        <div class='flex flex-col gap-1'>
          <Label for='field-prompt-template'>Prompt Template</Label>
          <span class='text-text-muted text-xs'>
            Supports {'{{task_title}}'}, {'{{task_body}}'}, and {'{{repo_url}}'} variables.
          </span>
        </div>
        <TextArea
          id='field-prompt-template'
          value={d.promptTemplate.value}
          onInput={(e) => a.onPromptTemplateChange((e.target as HTMLTextAreaElement).value)}
          disabled={status.submitting.value}
          rows={4}
        />
      </div>

      <div class='flex flex-col gap-2'>
        <Label for='field-repo-url'>Repo URL</Label>
        {d.repos.length > 0 ? (
          <Select
            id='field-repo-url'
            value={d.repoUrl.value}
            onChange={(e) => a.onRepoUrlChange((e.target as HTMLSelectElement).value)}
            disabled={status.submitting.value}
          >
            <option value=''>Select a repository...</option>
            {d.repos.map((r) => (
              <option key={r} value={`https://github.com/${r}`}>
                {r}
              </option>
            ))}
          </Select>
        ) : (
          <Input
            id='field-repo-url'
            type='text'
            value={d.repoUrl.value}
            onInput={(e) => a.onRepoUrlChange((e.target as HTMLInputElement).value)}
            placeholder='https://github.com/org/repo'
            disabled={status.submitting.value || d.reposLoading}
          />
        )}
        {d.reposLoading && <span class='text-xs text-text-muted'>Loading repos...</span>}
      </div>

      <div class='flex flex-col gap-2'>
        <Label for='field-agents-md'>Agents.md</Label>
        <span class='text-text-muted text-xs'>
          Project instructions written to AGENTS.md in the work directory.
        </span>
        <TextArea
          id='field-agents-md'
          value={d.agentsMd.value}
          onInput={(e) => a.onAgentsMdChange((e.target as HTMLTextAreaElement).value)}
          disabled={status.submitting.value}
          rows={6}
        />
      </div>

      <div class='flex flex-col gap-2'>
        <Label for='field-opencode-config'>OpenCode Config</Label>
        <span class='text-text-muted text-xs'>
          JSON configuration for opencode. Written to opencode.json. Supports {'{env:VAR}'} syntax
          for environment variable references.
        </span>
        <TextArea
          id='field-opencode-config'
          value={d.opencodeConfig.value}
          onInput={(e) => a.onOpencodeConfigChange((e.target as HTMLTextAreaElement).value)}
          disabled={status.submitting.value}
          rows={6}
        />
      </div>
    </>
  );
};
