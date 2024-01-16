import useSWR from 'swr';
import useTranslation from 'next-translate/useTranslation';
import { Card, Elevation, H4 } from '@blueprintjs/core';
import { categoryFetcher } from '../utils/api';
import styles from '../styles/PackageList.module.css';

const CategoryListView = () => {
  const { t } = useTranslation('common');
  const { data: categories, error } = useSWR(
    `/api/v1/category`,
    categoryFetcher
  );

  if (error) {
    return <div>{t('failedLoading')}</div>;
  }

  if (!categories) {
    return null;
  }

  return (
    <div className={styles.packageContainer}>
      <ul className={styles.noBullets}>
        {categories.map((category) => {
          return (
            <li className="mt-[2rem]" key={`${category.name}-${category.name}`}>
              <Card interactive={false} elevation={Elevation.ONE}>
                <span>
                  <H4 className={styles.name}>{category.name}</H4>
                </span>
                <div className={styles.createdAt}>{category.createdAt}</div>
              </Card>
            </li>
          );
        })}
      </ul>
    </div>
  );
};

export default CategoryListView;
