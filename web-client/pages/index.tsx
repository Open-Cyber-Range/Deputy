import type {NextPage} from 'next';
import Head from 'next/head';
import useTranslation from 'next-translate/useTranslation';
import styles from '../styles/Home.module.css';
import MainNavbar from '../components/MainNavbar';
import Dashboard from './dashboard';

const Home: NextPage = () => {
  const {t} = useTranslation('common');
  return (
    <div>
      <Head>
        <title>{t('title')}</title>
        <meta name={t('metaName')} content={t('metaContent')} />
      </Head>

      <header className={styles.header}>
        <span>{t('searchbox')}</span>
      </header>

      <main className={styles.main}>
        <Dashboard />
      </main>

      <footer className={styles.footer}>
        <span>{t('footer')}</span>
      </footer>
    </div>
  );
};

export default Home;
