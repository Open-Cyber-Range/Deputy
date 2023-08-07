import type { AppProps } from 'next/app';
import '@blueprintjs/core/lib/css/blueprint.css';
import '../styles/global.css';
import Head from 'next/head';
import useTranslation from 'next-translate/useTranslation';
import { SessionProvider } from 'next-auth/react';
import MainNavbar from '../components/MainNavbar';

const Deputy = ({
  Component,
  pageProps: { session, ...pageProps },
}: AppProps) => {
  const { t } = useTranslation('common');

  return (
    <SessionProvider session={session}>
      <Head>
        <title>{t('title')}</title>
        <meta name={t('metaName')} content={t('metaContent')} />
      </Head>
      <MainNavbar />
      <Component {...pageProps} />
    </SessionProvider>
  );
};

export default Deputy;
