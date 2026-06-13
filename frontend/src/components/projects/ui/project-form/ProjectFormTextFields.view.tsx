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
  const connectedProviderItems = f.modelProviders.map((provider) => ({
    value: provider.providerKey,
    label: provider.displayName || provider.providerKey
  }));
  const primaryCatalogProvider = f.catalogProviders.find(
    (provider) => provider.id === f.primaryModelProviderKey.value
  );
  const smallCatalogProvider = f.catalogProviders.find(
    (provider) => provider.id === f.smallModelProviderKey.value
  );

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
            onValueChange={f.onRepoUrlChange}
            disabled={m.submitting.value}
            placeholder='Select a repository...'
            items={f.repos.map((r) => ({
              value: `https://github.com/${r}`,
              label: r
            }))}
          />
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
        <Label for='field-primary-model-provider'>Primary Model Provider</Label>
        <Select
          id='field-primary-model-provider'
          value={f.primaryModelProviderKey.value}
          onValueChange={f.onPrimaryModelProviderChange}
          disabled={m.submitting.value}
          placeholder='Select a connected model provider...'
          items={connectedProviderItems}
        />
      </div>

      <div class='flex flex-col gap-2'>
        <Label for='field-primary-model'>Primary Model</Label>
        <Select
          id='field-primary-model'
          value={f.primaryModelId.value}
          onValueChange={f.onPrimaryModelChange}
          disabled={m.submitting.value || !primaryCatalogProvider}
          placeholder='Select a model...'
          items={(primaryCatalogProvider?.models ?? []).map((model) => ({
            value: model.id,
            label: model.name
          }))}
        />
      </div>

      <div class='grid grid-cols-1 md:grid-cols-2 gap-4'>
        <div class='flex flex-col gap-2'>
          <Label for='field-small-model-provider'>Small Model Provider</Label>
          <Select
            id='field-small-model-provider'
            value={f.smallModelProviderKey.value}
            onValueChange={f.onSmallModelProviderChange}
            disabled={m.submitting.value}
            placeholder='Optional provider...'
            items={connectedProviderItems}
          />
        </div>
        <div class='flex flex-col gap-2'>
          <Label for='field-small-model'>Small Model</Label>
          <Select
            id='field-small-model'
            value={f.smallModelId.value}
            onValueChange={f.onSmallModelChange}
            disabled={m.submitting.value || !smallCatalogProvider}
            placeholder='Optional model...'
            items={(smallCatalogProvider?.models ?? []).map((model) => ({
              value: model.id,
              label: model.name
            }))}
          />
        </div>
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
        <Label for='field-opencode-config'>Advanced OpenCode Config</Label>
        <span class='text-text-muted text-xs'>
          Raw user-managed OpenCode JSON. It is merged after app-managed provider/model config and
          can override generated values.
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
