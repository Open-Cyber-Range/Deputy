import type { NextPage } from 'next';
import { Button, H3 } from '@blueprintjs/core';
import useTranslation from 'next-translate/useTranslation';
import { useRouter } from 'next/router';

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
      <main>
        <div className="flex flex-col items-center p-10 mt-6">
          <H3>{t('welcome')}</H3>
          <Button className="mt-6" intent="primary" large onClick={handleClick}>
            {t('documentationButton')}
          </Button>
        </div>
      </main>
    </div>
  );
};

export default Home;
