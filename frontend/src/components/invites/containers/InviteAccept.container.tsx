import type { JSX } from 'preact';
import { useInviteAccept } from '../hooks/useInviteAccept.hook';
import { InviteAcceptView } from '../ui/InviteAccept.view';

interface InviteAcceptContainerProps {
  token: string;
}

export const InviteAcceptContainer = ({ token }: InviteAcceptContainerProps): JSX.Element => {
  const invite = useInviteAccept({ token });

  return <InviteAcceptView {...invite} />;
};
