import type { AppProps } from 'next/app';
import '@blueprintjs/core/lib/css/blueprint.css';
import '../styles/global.css';
import Head from 'next/head';
import useTranslation from 'next-translate/useTranslation';
import MainNavbar from '../components/MainNavbar';

const Deputy = ({ Component, pageProps }: AppProps) => {
  const { t } = useTranslation('common');

  return (
    <>
      <Head>
        <title>{t('title')}</title>
        <meta name={t('metaName')} content={t('metaContent')} />
      </Head>
      <div className="flex flex-col min-h-screen">
        <MainNavbar />
        <main className="grow">
          {/* eslint-disable-next-line react/jsx-props-no-spreading */}
          <Component {...pageProps} />
        </main>
      </div>
    </>
  );
};

export default Deputy;
