import type {NextPage} from 'next';
import {Button} from '@blueprintjs/core';
import useTranslation from 'next-translate/useTranslation';
import {useRouter} from 'next/router';
import styles from '../styles/Index.module.css';

const Home: NextPage = () => {
  const {t} = useTranslation('common');
  const router = useRouter();
  const handleClick = (event: React.MouseEvent<HTMLElement>) => {
    event.preventDefault();

    if (process.env.DOCUMENTATION_URL) {
      void router.push(process.env.DOCUMENTATION_URL);
    }
  };

  return (
    <div>
      <main className={styles.main}>
        <div className = {styles.dashboard}>
          <h1>{t('welcome')}</h1>
          <Button intent='primary' large onClick={handleClick}>{t('documentationButton')}</Button>
        </div>
      </main>

      <footer className={styles.footer}>
        <span>{t('footer')}</span>
      </footer>
    </div>
  );
};

export default Home;
