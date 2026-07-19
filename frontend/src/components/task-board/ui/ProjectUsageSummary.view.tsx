import type { JSX } from 'preact';
import type { TaskBoardProjectUsageSummaryViewProps } from '../types';
import { ProjectUsagePeriodView } from './ProjectUsagePeriod.view';

export const ProjectUsageSummaryView = ({
  data
}: TaskBoardProjectUsageSummaryViewProps): JSX.Element => (
  <section aria-label='Project usage' class='border border-border-base bg-bg-panel'>
    <div class='flex flex-col gap-1 border-b border-border-base px-4 py-3'>
      <h3 class='text-sm font-medium text-text-primary'>Project usage</h3>
      <p class='text-xs text-text-muted'>Token usage</p>
    </div>
    {data.emptyMessage ? (
      <p class='p-4 text-sm text-text-muted'>{data.emptyMessage}</p>
    ) : (
      <div class='grid md:grid-cols-2'>
        <ProjectUsagePeriodView data={data.total} />
        <div class='border-t border-border-base md:border-t-0 md:border-l'>
          <ProjectUsagePeriodView data={data.thisWeek} />
        </div>
      </div>
    )}
  </section>
);
