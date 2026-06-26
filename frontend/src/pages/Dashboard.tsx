import type { JSX } from 'preact';
import { PageLayout } from '../components/shared/ui/PageLayout.view';
import { TaskBoardContainer } from '../components/task-board/containers/TaskBoard.container';

export const Dashboard = (): JSX.Element => (
  <PageLayout maxWidth='6xl' gap={8}>
    <TaskBoardContainer />
  </PageLayout>
);
