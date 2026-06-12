import type { JSX } from 'preact';
import { InviteAcceptContainer } from '../components/invites/containers/InviteAccept.container';

interface InviteAcceptProps {
  token: string;
}

export const InviteAccept = ({ token }: InviteAcceptProps): JSX.Element => (
  <InviteAcceptContainer token={token} />
);
