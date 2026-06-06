import type { Signal } from '@preact/signals';
import type { JSX } from 'preact';
import { Button } from '../../../shared/ui/Button.view';
import { Card } from '../../../shared/ui/Card.view';

interface WorkersRegistrationCodesProps {
  code: string | null;
  countdown: Signal<string>;
  generateLoading: boolean;
  onGenerateCode: () => void;
}

export const WorkersRegistrationCodes = ({
  code,
  countdown,
  generateLoading,
  onGenerateCode
}: WorkersRegistrationCodesProps): JSX.Element => (
  <section class='flex flex-col gap-4'>
    <div class='flex items-center justify-between'>
      <h2 class='text-lg font-semibold text-text-primary uppercase tracking-wide'>
        Registration Codes
      </h2>
      <Button variant='primary' onClick={onGenerateCode} disabled={generateLoading}>
        {generateLoading ? 'Generating...' : 'Generate Code'}
      </Button>
    </div>

    {code && (
      <Card class='flex flex-col gap-2'>
        <div class='flex items-center gap-4'>
          <span class='text-text-muted text-sm uppercase tracking-wider'>Code:</span>
          <code class='text-accent font-mono text-lg tracking-widest'>{code}</code>
        </div>
        {countdown.value && <span class='text-text-muted text-sm'>{countdown.value}</span>}
      </Card>
    )}
  </section>
);
