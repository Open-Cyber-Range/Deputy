import { useRouter } from 'next/router';
import useSWR from 'swr';
import useTranslation from 'next-translate/useTranslation';
import { Card, Elevation, H4 } from '@blueprintjs/core';
import Link from 'next/link';
import { packagesWithVersionsFetcher } from '../../utils/api';
import styles from '../../styles/PackageList.module.css';
import { formatBytes, getLatestVersion } from '../../utils';

const SearchPage = () => {
  const { t } = useTranslation('common');
  const { query } = useRouter();
  const { q } = query as { q: string };

  const { data: searchResults, error } = useSWR(
    `/api/v1/package?search_term=${q}`,
    packagesWithVersionsFetcher
  );
  if (error) {
    return (
      <div className={styles.packageContainer}>
        <H4>{t('searchResultsFor', { q })}</H4>
        {t('failedLoading')}
      </div>
    );
  }

  if (!searchResults || searchResults.packages.length === 0) {
    return (
      <div className={styles.packageContainer}>
        <H4>{t('searchResultsFor', { q })}</H4>
        {t('noResults')}
      </div>
    );
  }

  return (
    <div className={styles.packageContainer}>
      <H4>{t('searchResultsFor', { q })}</H4>
      <ul className={styles.noBullets}>
        {searchResults.packages.map((deputyPackage) => {
          const latestVersion = getLatestVersion(deputyPackage);
          return (
            latestVersion && (
              <li
                className="mt-[2rem]"
                key={`${deputyPackage.name}-${latestVersion.version}`}
              >
                <Card interactive={false} elevation={Elevation.ONE}>
                  <span>
                    <Link
                      href={`/packages/${deputyPackage.name}/${latestVersion.version}`}
                      className={styles.name}
                    >
                      {deputyPackage.name}
                    </Link>
                  </span>
                  <span className={styles.version}>
                    {latestVersion.version}
                  </span>
                  <span className={styles.packageSize}>
                    {formatBytes(latestVersion.packageSize)}
                  </span>
                  <div className={styles.description}>
                    {deputyPackage.description}
                  </div>
                </Card>
              </li>
            )
          );
        })}
      </ul>
    </div>
  );
};

export default SearchPage;
