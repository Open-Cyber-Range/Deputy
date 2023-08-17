import NextAuth, { AuthOptions, Session } from 'next-auth';
import KeycloakProvider from 'next-auth/providers/keycloak';

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
      }
      return token;
    },
    async session({ session, token }) {
      const sessionParamter: Session & { idToken?: string } = {
        ...session,
      };
      if (token.idToken) {
        // eslint-disable-next-line no-param-reassign
        sessionParamter.idToken = token.idToken as string;
      }
      return sessionParamter;
    },
  },
};

export default NextAuth(authOptions);
