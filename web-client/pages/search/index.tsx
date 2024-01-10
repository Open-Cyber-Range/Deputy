import { useRouter } from 'next/router';
import useSWR from 'swr';
import useTranslation from 'next-translate/useTranslation';
import { Button, Card, Elevation, H4, HTMLSelect } from '@blueprintjs/core';
import Link from 'next/link';
import { useState } from 'react';
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
  const [currentPage, setCurrentPage] = useState(1);
  const [selectedLimit, setSelectedLimit] = useState(20);

  let apiSearchUrl = `/api/v1/package?search_term=${q}&page=${currentPage}&limit=${selectedLimit}`;
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
        <div>
          <HTMLSelect
            id="limit"
            value={selectedLimit}
            iconName="caret-down"
            onChange={(event) => {
              setSelectedLimit(parseInt(event.target.value, 10));
              setCurrentPage(1);
            }}
          >
            <option value="5">5</option>
            <option value="10">10</option>
            <option value="20">20</option>
            <option value="50">50</option>
          </HTMLSelect>

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
          <div className="flex flex-row justify-center gap-[1rem] mt-6">
            <Button
              onClick={() => setCurrentPage((prev) => Math.max(prev - 1, 1))}
              disabled={currentPage === 1}
              icon="chevron-left"
            />

            <span>
              {currentPage} / {searchResults.totalPages}
            </span>

            <Button
              onClick={() =>
                setCurrentPage((prev) =>
                  Math.min(prev + 1, searchResults.totalPages)
                )
              }
              disabled={currentPage === searchResults.totalPages}
              icon="chevron-right"
            />
          </div>
        </div>
      )}
    </div>
  );
};

export default SearchPage;
