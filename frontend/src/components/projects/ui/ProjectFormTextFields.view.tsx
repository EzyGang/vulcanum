import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';
import { Input } from '../../shared/ui/Input.view';
import { Label } from '../../shared/ui/Label.view';

interface ProjectFormTextFieldsProps {
  promptTemplate: Signal<string>;
  repoUrl: Signal<string>;
  agentsMd: Signal<string>;
  opencodeConfig: Signal<string>;
  repos: Signal<string[]>;
  reposLoading: Signal<boolean>;
  submitting: Signal<boolean>;
  onPromptTemplateChange: (value: string) => void;
  onRepoUrlChange: (value: string) => void;
  onAgentsMdChange: (value: string) => void;
  onOpencodeConfigChange: (value: string) => void;
}

const inputStyles =
  'bg-bg-input border border-border-base text-text-primary px-4 py-3 text-sm w-full';
const textareaStyles = `${inputStyles} font-mono`;

export const ProjectFormTextFields = ({
  promptTemplate,
  repoUrl,
  agentsMd,
  opencodeConfig,
  repos,
  reposLoading,
  submitting,
  onPromptTemplateChange,
  onRepoUrlChange,
  onAgentsMdChange,
  onOpencodeConfigChange
}: ProjectFormTextFieldsProps): JSX.Element => (
  <>
    <div class='flex flex-col gap-2'>
      <div class='flex flex-col gap-1'>
        <Label for='field-prompt-template'>Prompt Template</Label>
        <span class='text-text-muted text-xs'>
          Supports {'{{task_title}}'}, {'{{task_body}}'}, and {'{{repo_url}}'} variables.
        </span>
      </div>
      <textarea
        id='field-prompt-template'
        value={promptTemplate.value}
        onInput={(e) => onPromptTemplateChange((e.target as HTMLTextAreaElement).value)}
        disabled={submitting.value}
        rows={4}
        class={textareaStyles}
      />
    </div>

    <div class='flex flex-col gap-2'>
      <Label for='field-repo-url'>Repo URL</Label>
      {repos.value.length > 0 ? (
        <select
          id='field-repo-url'
          value={repoUrl.value}
          onChange={(e) => onRepoUrlChange((e.target as HTMLSelectElement).value)}
          disabled={submitting.value}
          class={`${inputStyles} cursor-pointer`}
        >
          <option value=''>Select a repository...</option>
          {repos.value.map((r) => (
            <option key={r} value={`https://github.com/${r}`}>
              {r}
            </option>
          ))}
        </select>
      ) : (
        <Input
          id='field-repo-url'
          type='text'
          value={repoUrl.value}
          onInput={(e) => onRepoUrlChange((e.target as HTMLInputElement).value)}
          placeholder='https://github.com/org/repo'
          disabled={submitting.value || reposLoading.value}
        />
      )}
      {reposLoading.value && <span class='text-xs text-text-muted'>Loading repos...</span>}
    </div>

    <div class='flex flex-col gap-2'>
      <Label for='field-agents-md'>Agents.md</Label>
      <span class='text-text-muted text-xs'>
        Project instructions written to AGENTS.md in the work directory.
      </span>
      <textarea
        id='field-agents-md'
        value={agentsMd.value}
        onInput={(e) => onAgentsMdChange((e.target as HTMLTextAreaElement).value)}
        disabled={submitting.value}
        rows={6}
        class={textareaStyles}
      />
    </div>

    <div class='flex flex-col gap-2'>
      <Label for='field-opencode-config'>OpenCode Config</Label>
      <span class='text-text-muted text-xs'>
        JSON configuration for opencode. Written to opencode.json. Supports {'{env:VAR}'} syntax for
        environment variable references.
      </span>
      <textarea
        id='field-opencode-config'
        value={opencodeConfig.value}
        onInput={(e) => onOpencodeConfigChange((e.target as HTMLTextAreaElement).value)}
        disabled={submitting.value}
        rows={6}
        class={textareaStyles}
      />
    </div>
  </>
);
