import NextAuth, { AuthOptions, Session } from 'next-auth';
import { JWT } from 'next-auth/jwt';
import KeycloakProvider from 'next-auth/providers/keycloak';

export const refreshToken = async (token: JWT): Promise<JWT> => {
  const details = {
    client_id: process.env.KEYCLOAK_CLIENT_ID,
    client_secret: process.env.KEYCLOAK_CLIENT_SECRET,
    grant_type: ['refresh_token'],
    refresh_token: token.refreshToken!,
  };
  const formBody: string[] = [];
  Object.entries(details).forEach(([key, value]: [string, any]) => {
    const encodedKey = encodeURIComponent(key);
    const encodedValue = encodeURIComponent(value);
    formBody.push(`${encodedKey}=${encodedValue}`);
  });
  const formData = formBody.join('&');
  const url = `${process.env.KEYCLOAK_ISSUER}/protocol/openid-connect/token`;
  const response = await fetch(url, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/x-www-form-urlencoded;charset=UTF-8',
    },
    body: formData,
  });
  const refreshedTokens = await response.json();

  return {
    ...token,
    idToken: refreshedTokens.id_token,
    refreshToken: refreshedTokens.refresh_token,
  };
};

export const authOptions: AuthOptions = {
  providers: [
    KeycloakProvider({
      clientId: process.env.KEYCLOAK_CLIENT_ID ?? '',
      clientSecret: process.env.KEYCLOAK_CLIENT_SECRET ?? '',
      issuer: process.env.KEYCLOAK_ISSUER,
    }),
  ],
  session: {
    strategy: 'jwt',
  },
  callbacks: {
    async jwt({ token, account }) {
      if (account) {
        // eslint-disable-next-line no-param-reassign
        token.idToken = account.id_token;
        // eslint-disable-next-line no-param-reassign
        token.refreshToken = account.refresh_token;
        return token;
      }
      return refreshToken(token);
    },
    async session({ session, token }) {
      const mergedSession: Session & {
        idToken?: string;
        refreshToken?: string;
      } = {
        ...session,
      };

      if (token.idToken) {
        // eslint-disable-next-line no-param-reassign
        mergedSession.idToken = token.idToken as string;
        mergedSession.refreshToken = token.refreshToken as string;
      }
      return mergedSession;
    },
  },
};

export default NextAuth(authOptions);
