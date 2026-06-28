import type { JSX } from 'preact';
import { useTaskBoard } from '../hooks/useTaskBoard.hook';
import { TaskBoardView } from '../ui/TaskBoard.view';

export const TaskBoardContainer = (): JSX.Element => {
  const { data, form, status, actions } = useTaskBoard();

  return <TaskBoardView data={data} form={form} status={status} actions={actions} />;
};
