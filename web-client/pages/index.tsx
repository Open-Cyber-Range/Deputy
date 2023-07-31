import type { NextPage } from 'next';
import { Button, H3 } from '@blueprintjs/core';
import useTranslation from 'next-translate/useTranslation';
import { useRouter } from 'next/router';
import { useSession, signIn, signOut } from 'next-auth/react';
import styles from '../styles/Index.module.css';

const Home: NextPage = () => {
  const { t } = useTranslation('common');
  const { data: session } = useSession();
  const router = useRouter();
  const handleClick = (event: React.MouseEvent<HTMLElement>) => {
    event.preventDefault();

    if (process.env.DOCUMENTATION_URL) {
      router.push(process.env.DOCUMENTATION_URL);
    }
  };
  if (session) {
    console.log(session.user?.email);
    return (
      <div>
        <main className={styles.main}>
          <div className={styles.dashboard}>
            <h1>{t('welcome')}</h1>
            <Button intent="primary" large onClick={handleClick}>
              {t('documentationButton')}
            </Button>
            <button onClick={() => signOut()}>Sign out</button>
          </div>
        </main>

        <footer className={styles.footer}>
          <span>{t('footer')}</span>
        </footer>
      </div>
    );
  }
  return (
    <>
      Not signed in <br />
      <button onClick={() => signIn()}>Sign in</button>
    </>
  );
};

export default Home;
