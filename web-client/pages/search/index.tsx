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
  const { q, type, categories } = query as {
    q: string;
    type: string;
    categories: string;
  };

  let apiSearchUrl = `/api/v1/package?search_term=${q}`;
  let searchInput = `"${q}"`;
  if (type) {
    apiSearchUrl += `&type=${type}`;
    searchInput += ` (type: ${type})`;
  }
  if (categories) {
    apiSearchUrl += `&categories=${categories}`;
    searchInput += ` (categories: ${categories})`;
  }

  const { data: searchResults, error } = useSWR(
    `${apiSearchUrl}`,
    packagesWithVersionsFetcher
  );

  return (
    <div className={styles.packageContainer}>
      <H4>{t('searchResultsFor', { searchInput })}</H4>
      {error && <div>{t('failedLoading')}</div>}
      {(!searchResults || searchResults.packages.length === 0) && (
        <div>{t('noResults')}</div>
      )}
      {searchResults && searchResults.packages.length > 0 && (
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
      )}
    </div>
  );
};

export default SearchPage;
