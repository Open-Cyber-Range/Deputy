import { Session } from 'next-auth';

export interface Token {
  id: string;
  name: string;
  token: string;
  userId: string;
  createdAt: string;
  updatedAt: string;
}

export interface PostToken {
  name: string;
}

export type ModifiedSession = Session & { idToken?: string };
