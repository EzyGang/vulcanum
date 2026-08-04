import type { JSX } from 'preact';

interface RunBlockedReasonProps {
  reason: string | null;
}

export const RunBlockedReason = ({ reason }: RunBlockedReasonProps): JSX.Element | null =>
  reason ? (
    <p class='max-w-sm break-words text-left text-text-secondary text-xs leading-relaxed'>
      <span class='font-semibold text-text-primary'>Blocked reason:</span> {reason}
    </p>
  ) : null;
