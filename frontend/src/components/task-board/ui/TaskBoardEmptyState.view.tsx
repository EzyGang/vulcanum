import { IconBrandGithub, IconCpu, IconServer, IconTicket } from '@tabler/icons-react';
import type { JSX } from 'preact';

interface SetupStep {
  title: string;
  description: string;
  href: string;
  action: string;
  icon: typeof IconTicket;
}

const SETUP_STEPS: SetupStep[] = [
  {
    title: 'Task tracker',
    description:
      'Connect a task tracker first so Vulcanum can discover projects and create a board.',
    href: '/settings?tab=providers',
    action: 'Connect tracker',
    icon: IconTicket
  },
  {
    title: 'Model provider',
    description: 'Connect the models that workers will use for implementation and review runs.',
    href: '/settings?tab=model-providers',
    action: 'Connect models',
    icon: IconCpu
  },
  {
    title: 'GitHub app',
    description:
      'Install the GitHub app to give projects access to repositories and pull requests.',
    href: '/settings?tab=github',
    action: 'Connect GitHub',
    icon: IconBrandGithub
  },
  {
    title: 'Worker',
    description: 'Register a worker machine so configured projects can run agent jobs.',
    href: '/workers',
    action: 'Register worker',
    icon: IconServer
  }
];

export const TaskBoardEmptyState = (): JSX.Element => (
  <section class='flex flex-col gap-6 animate-fade-in' aria-labelledby='getting-started-title'>
    <div class='flex max-w-3xl flex-col gap-2'>
      <span class='text-xs font-medium uppercase tracking-wider text-accent'>Getting started</span>
      <h2 id='getting-started-title' class='text-3xl font-semibold text-text-primary text-balance'>
        Configure Vulcanum before adding a board
      </h2>
      <p class='text-sm leading-relaxed text-text-muted text-pretty'>
        Boards come from a connected task tracker. Complete the setup steps below, then use the
        board picker in the navigation to add a provider project.
      </p>
    </div>

    <div class='grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-4'>
      {SETUP_STEPS.map((step, index) => {
        const Icon = step.icon;
        return (
          <article
            key={step.title}
            class='flex min-h-64 flex-col border border-border-base bg-bg-card'
          >
            <div class='flex items-center justify-between gap-3 border-b border-border-base px-4 py-3'>
              <Icon size={18} stroke={1.75} aria-hidden='true' />
              <span class='font-mono text-xs text-text-muted'>0{index + 1}</span>
            </div>
            <div class='flex flex-1 flex-col gap-3 p-4'>
              <h3 class='text-sm font-semibold uppercase tracking-wider text-text-primary'>
                {step.title}
              </h3>
              <p class='text-sm leading-relaxed text-text-muted text-pretty'>{step.description}</p>
              <a
                href={step.href}
                class='mt-auto inline-flex min-h-10 items-center justify-center border border-border-base px-3 py-2 text-xs font-medium uppercase tracking-wider text-text-primary transition-colors duration-fast hover:border-border-focus hover:bg-bg-hover focus:outline-none focus-visible:ring-2 focus-visible:ring-border-focus active:scale-[0.96]'
              >
                {step.action}
              </a>
            </div>
          </article>
        );
      })}
    </div>
  </section>
);
