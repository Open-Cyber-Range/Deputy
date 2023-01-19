import type {AppProps} from 'next/app';
import '@blueprintjs/core/lib/css/blueprint.css';
import '../styles/global.css';
import MainNavbar from '../components/MainNavbar';
import Head from 'next/head';
import useTranslation from 'next-translate/useTranslation';

function MyApp({Component, pageProps}: AppProps) {
  const {t} = useTranslation('common');

  return (
    <>
      <Head>
        <title>{t('title')}</title>
        <meta name={t('metaName')} content={t('metaContent')} />
      </Head>
      <MainNavbar/>
      <Component {...pageProps} />
    </>
  );
}

export default MyApp;
