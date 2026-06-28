import type { Signal } from '@preact/signals';
import { IconCheck, IconCopy, IconKey, IconTerminal2 } from '@tabler/icons-react';
import type { JSX } from 'preact';
import { Button } from '../../../shared/ui/Button.view';
import { ErrorBanner } from '../../../shared/ui/ErrorBanner.view';
import type { WorkerRegistrationCopyTarget } from '../../hooks/useWorkers.hook';

interface WorkersRegistrationCodesProps {
  maskedCode: string | null;
  setupCommandPreview: string | null;
  countdown: Signal<string>;
  generateLoading: boolean;
  copiedTarget: Signal<WorkerRegistrationCopyTarget | null>;
  copyError: Signal<string | null>;
  onGenerateCode: () => void;
  onCopyCode: () => void;
  onCopySetupCommand: () => void;
}

const CopyButtonLabel = ({
  copied,
  children
}: {
  copied: boolean;
  children: string;
}): JSX.Element => (
  <span class='inline-flex items-center gap-2'>
    {copied ? (
      <IconCheck size={15} stroke={1.75} aria-hidden='true' />
    ) : (
      <IconCopy size={15} stroke={1.75} aria-hidden='true' />
    )}
    {copied ? 'Copied' : children}
  </span>
);

export const WorkersRegistrationCodes = ({
  maskedCode,
  setupCommandPreview,
  countdown,
  generateLoading,
  copiedTarget,
  copyError,
  onGenerateCode,
  onCopyCode,
  onCopySetupCommand
}: WorkersRegistrationCodesProps): JSX.Element => (
  <section class='flex flex-col gap-4'>
    <div class='flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between'>
      <div class='flex flex-col gap-2'>
        <span class='text-xs font-medium uppercase tracking-wider text-accent'>
          Worker enrollment
        </span>
        <h2 class='text-lg font-semibold text-text-primary uppercase tracking-wide'>
          Registration code
        </h2>
        <p class='max-w-2xl text-sm leading-relaxed text-text-muted'>
          Generate a short-lived secret, then copy the setup command onto the machine that should
          run the worker daemon.
        </p>
      </div>
      <Button variant='primary' onClick={onGenerateCode} disabled={generateLoading}>
        {generateLoading ? 'Generating…' : 'Generate code'}
      </Button>
    </div>

    {copyError.value && <ErrorBanner message={copyError.value} />}

    {maskedCode && setupCommandPreview ? (
      <div class='border border-border-focus bg-bg-page'>
        <div class='flex items-center justify-between gap-3 border-b border-border-base bg-bg-card px-4 py-3'>
          <span class='inline-flex items-center gap-2 text-xs font-medium uppercase tracking-wider text-text-primary'>
            <IconKey size={15} stroke={1.75} aria-hidden='true' />
            One-time secret ready
          </span>
          {countdown.value && <span class='text-xs text-text-muted'>{countdown.value}</span>}
        </div>

        <div class='flex flex-col gap-4 p-4'>
          <div class='grid gap-3 lg:grid-cols-[minmax(0,1fr)_auto] lg:items-center'>
            <div class='flex min-w-0 flex-col gap-2'>
              <span class='text-xs uppercase tracking-wider text-text-muted'>
                Registration code
              </span>
              <code class='font-mono text-lg tracking-[0.28em] text-text-primary'>
                {maskedCode}
              </code>
              <span class='text-xs text-text-muted'>
                Hidden in the interface. Copy it directly when a manual setup step needs only the
                code.
              </span>
            </div>
            <Button variant='secondary' onClick={onCopyCode}>
              <CopyButtonLabel copied={copiedTarget.value === 'code'}>Copy code</CopyButtonLabel>
            </Button>
          </div>

          <div class='grid gap-3 border border-border-base bg-bg-card p-3 lg:grid-cols-[minmax(0,1fr)_auto] lg:items-center'>
            <div class='flex min-w-0 flex-col gap-2'>
              <span class='inline-flex items-center gap-2 text-xs uppercase tracking-wider text-text-muted'>
                <IconTerminal2 size={15} stroke={1.75} aria-hidden='true' />
                Recommended setup command
              </span>
              <code class='block overflow-x-auto whitespace-nowrap font-mono text-xs text-text-secondary'>
                {setupCommandPreview}
              </code>
            </div>
            <Button variant='primary' onClick={onCopySetupCommand}>
              <CopyButtonLabel copied={copiedTarget.value === 'setup-command'}>
                Copy command
              </CopyButtonLabel>
            </Button>
          </div>
        </div>
      </div>
    ) : (
      <div class='border border-border-base bg-bg-page p-5 text-sm leading-relaxed text-text-muted'>
        No active registration secret is displayed. Generate one when you are ready to enroll a
        worker host.
      </div>
    )}
  </section>
);
