import type { ComponentChildren, JSX } from 'preact';
import { Button } from '../../shared/ui/Button.view';
import type { CliLoginViewProps } from '../hooks/useCliLogin.hook';

export const CliLoginCodeView = ({ data, status, actions }: CliLoginViewProps): JSX.Element => (
  <CliLoginFrame>
    <div class='flex flex-col gap-5'>
      <p class='text-pretty text-sm leading-6 text-text-secondary'>
        Copy this one-time code into the terminal where vulcanum login is waiting.
      </p>
      <code class='select-all overflow-x-auto border border-border-base bg-bg-input px-4 py-4 font-mono text-sm text-text-primary'>
        {data.code}
      </code>
      <div class='flex flex-col gap-3'>
        <Button type='button' variant='primary' onClick={actions.onCopy}>
          Copy code
        </Button>
        <p class='min-h-5 text-sm text-text-secondary' aria-live='polite'>
          {status.copyMessage}
        </p>
      </div>
    </div>
  </CliLoginFrame>
);

export const CliLoginMissingView = (): JSX.Element => (
  <CliLoginFrame>
    <p class='text-pretty text-sm leading-6 text-error'>
      Authorization code missing. Restart vulcanum login and complete GitHub sign-in.
    </p>
  </CliLoginFrame>
);

const CliLoginFrame = ({ children }: { children: ComponentChildren }): JSX.Element => (
  <main class='flex min-h-screen items-center justify-center bg-bg-page px-5'>
    <section class='flex w-full max-w-md flex-col gap-6 border border-border-base bg-bg-card p-8'>
      <div class='flex flex-col gap-3'>
        <p class='font-mono text-xs uppercase tracking-[0.18em] text-text-muted'>
          Terminal handoff
        </p>
        <h1 class='text-balance text-2xl font-semibold tracking-wide text-text-primary'>
          CLI login
        </h1>
      </div>
      {children}
    </section>
  </main>
);
