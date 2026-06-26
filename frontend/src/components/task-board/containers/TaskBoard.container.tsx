import type { JSX } from 'preact';
import { useTaskBoard } from '../hooks/useTaskBoard.hook';
import { TaskBoardView } from '../ui/TaskBoard.view';

export const TaskBoardContainer = (): JSX.Element => {
  const taskBoard = useTaskBoard();

  return <TaskBoardView {...taskBoard} />;
};
