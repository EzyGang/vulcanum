import type { JSX } from 'preact';
import { Input } from '../../../shared/ui/Input.view';
import { Label } from '../../../shared/ui/Label.view';
import { Select } from '../../../shared/ui/Select.view';
import { TextArea } from '../../../shared/ui/TextArea.view';
import { useProjectFormFieldsContext } from '../../context/ProjectFormFieldsContext';
import { useProjectFormMetaContext } from '../../context/ProjectFormMetaContext';

export const ProjectFormTextFields = (): JSX.Element => {
  const m = useProjectFormMetaContext();
  const f = useProjectFormFieldsContext();

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
          value={f.promptTemplate.value}
          onInput={(e) => f.onPromptTemplateChange((e.target as HTMLTextAreaElement).value)}
          disabled={m.submitting.value}
          rows={4}
        />
      </div>

      <div class='flex flex-col gap-2'>
        <Label for='field-repo-url'>Repo URL</Label>
        {f.repos.length > 0 ? (
          <Select
            id='field-repo-url'
            value={f.repoUrl.value}
            onChange={(e) => f.onRepoUrlChange((e.target as HTMLSelectElement).value)}
            disabled={m.submitting.value}
          >
            <option value=''>Select a repository...</option>
            {f.repos.map((r) => (
              <option key={r} value={`https://github.com/${r}`}>
                {r}
              </option>
            ))}
          </Select>
        ) : (
          <Input
            id='field-repo-url'
            type='text'
            value={f.repoUrl.value}
            onInput={(e) => f.onRepoUrlChange((e.target as HTMLInputElement).value)}
            placeholder='https://github.com/org/repo'
            disabled={m.submitting.value || f.reposLoading}
          />
        )}
        {f.reposLoading && <span class='text-xs text-text-muted'>Loading repos...</span>}
      </div>

      <div class='flex flex-col gap-2'>
        <Label for='field-agents-md'>Agents.md</Label>
        <span class='text-text-muted text-xs'>
          Global agent guide for this project. Does not overwrite any per-repo AGENTS.md file.
        </span>
        <TextArea
          id='field-agents-md'
          value={f.agentsMd.value}
          onInput={(e) => f.onAgentsMdChange((e.target as HTMLTextAreaElement).value)}
          disabled={m.submitting.value}
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
          value={f.opencodeConfig.value}
          onInput={(e) => f.onOpencodeConfigChange((e.target as HTMLTextAreaElement).value)}
          disabled={m.submitting.value}
          rows={6}
        />
      </div>
    </>
  );
};
