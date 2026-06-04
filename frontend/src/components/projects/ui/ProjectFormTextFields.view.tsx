import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';
import { Input } from '../../shared/ui/Input.view';
import { Label } from '../../shared/ui/Label.view';

interface ProjectFormTextFieldsProps {
  promptTemplate: Signal<string>;
  repoUrl: Signal<string>;
  agentsMd: Signal<string>;
  opencodeConfig: Signal<string>;
  githubToken: Signal<string>;
  submitting: Signal<boolean>;
  onPromptTemplateChange: (value: string) => void;
  onRepoUrlChange: (value: string) => void;
  onAgentsMdChange: (value: string) => void;
  onOpencodeConfigChange: (value: string) => void;
  onGithubTokenChange: (value: string) => void;
  onGithubTokenFocus: () => void;
}

const inputStyles =
  'bg-bg-input border border-border-base text-text-primary px-4 py-3 text-sm w-full';
const textareaStyles = `${inputStyles} font-mono`;

export const ProjectFormTextFields = ({
  promptTemplate,
  repoUrl,
  agentsMd,
  opencodeConfig,
  githubToken,
  submitting,
  onPromptTemplateChange,
  onRepoUrlChange,
  onAgentsMdChange,
  onOpencodeConfigChange,
  onGithubTokenChange,
  onGithubTokenFocus
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
      <span class='text-text-muted text-xs'>
        Git repository URL. Use https://{'{token}'}@github.com/org/repo for private repos.
      </span>
      <Input
        id='field-repo-url'
        type='text'
        value={repoUrl.value}
        onInput={(e) => onRepoUrlChange((e.target as HTMLInputElement).value)}
        placeholder='https://github.com/org/repo'
        disabled={submitting.value}
      />
    </div>

    <div class='flex flex-col gap-2'>
      <Label for='field-agents-md'>Agents.md</Label>
      <span class='text-text-muted text-xs'>
        Global agent guide injected into sessions. Does not overwrite per-repo AGENTS.md files.
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

    <div class='flex flex-col gap-2'>
      <Label for='field-github-token'>GitHub Token</Label>
      <span class='text-text-muted text-xs'>
        Personal access token for PR creation. Leave empty to skip PRs.
      </span>
      <Input
        id='field-github-token'
        type='text'
        value={githubToken.value}
        onInput={(e) => onGithubTokenChange((e.target as HTMLInputElement).value)}
        onFocus={onGithubTokenFocus}
        placeholder='ghp_...'
        disabled={submitting.value}
      />
    </div>
  </>
);
