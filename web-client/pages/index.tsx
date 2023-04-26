import type { NextPage } from 'next';
import { Button, H3 } from '@blueprintjs/core';
import useTranslation from 'next-translate/useTranslation';
import { useRouter } from 'next/router';
import styles from '../styles/Index.module.css';

const Home: NextPage = () => {
  const { t } = useTranslation('common');
  const router = useRouter();
  const handleClick = (event: React.MouseEvent<HTMLElement>) => {
    event.preventDefault();

    if (process.env.DOCUMENTATION_URL) {
      router.push(process.env.DOCUMENTATION_URL);
    }
  };

  return (
    <div>
      <div className={styles.dashboard}>
        <H3>{t('welcome')}</H3>
        <Button intent="primary" large onClick={handleClick}>
          {t('documentationButton')}
        </Button>
      </div>
    </div>
  );
};

export default Home;
