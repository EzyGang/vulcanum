import { useSignal } from '@preact/signals';
import { useCallback } from 'preact/hooks';
import type { TaskBoardHelpCard } from '../types';
import { readDismissedHelpCards, writeDismissedHelpCards } from './taskBoard.helpers';

export const useTaskBoardHelpCards = () => {
  const dismissedHelpCards = useSignal<TaskBoardHelpCard[]>(readDismissedHelpCards());

  const dismissHelpCard = useCallback(
    (card: TaskBoardHelpCard) => {
      if (dismissedHelpCards.value.includes(card)) {
        return;
      }

      const nextCards = [...dismissedHelpCards.value, card];
      dismissedHelpCards.value = nextCards;
      writeDismissedHelpCards(nextCards);
    },
    [dismissedHelpCards]
  );

  return {
    dismissedHelpCards: dismissedHelpCards.value,
    onDismissHelpCard: dismissHelpCard
  };
};
