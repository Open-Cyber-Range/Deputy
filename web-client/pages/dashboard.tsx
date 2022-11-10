import {Button} from '@blueprintjs/core';
import useTranslation from 'next-translate/useTranslation';
import {useRouter} from 'next/router';
import styles from '../styles/Dashboard.module.css';

const Dashboard = () => {
  const {t} = useTranslation('common');
  const router = useRouter();
  const handleClick = (event: React.MouseEvent<HTMLElement>) => {
    event.preventDefault();

    if (process.env.DOCUMENTATION_URL) {
      void router.push(process.env.DOCUMENTATION_URL);
    } else {
      console.warn(t('docuUrlWarning'));
    }
  };

  return (
    <div>
      <main className={styles.dashboard}>
        <h2>{t('welcome')}</h2>
        <Button large onClick={handleClick}>{t('documentationButton')}</Button> <br/>
        <input className='bp4-input' type='search' placeholder={t('searchbox')} dir='auto'/>
      </main>
    </div>
  );
};

export default Dashboard;
