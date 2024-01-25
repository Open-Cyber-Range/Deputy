import useSWR from 'swr';
import useTranslation from 'next-translate/useTranslation';
import { Card, Elevation, H4 } from '@blueprintjs/core';
import Link from 'next/link';
import { categoryFetcher } from '../utils/api';
import styles from '../styles/PackageList.module.css';

const CategoryListView = () => {
  const { t } = useTranslation('common');
  const { data: categories, error } = useSWR(
    `/api/v1/category`,
    categoryFetcher
  );

  if (error || !categories || categories.length === 0) {
    return <div>{t('failedLoadingCategories')}</div>;
  }

  return (
    <div className={styles.packageContainer}>
      <ul className={styles.noBullets}>
        {categories.map((category) => {
          return (
            <li className="mt-[2rem]" key={`${category.name}-${category.name}`}>
              <Card interactive={false} elevation={Elevation.ONE}>
                <Link href={`/search?q=&categories=${category.name}`}>
                  <H4 className={styles.name}>{category.name}</H4>
                </Link>
              </Card>
            </li>
          );
        })}
      </ul>
    </div>
  );
};

export default CategoryListView;
