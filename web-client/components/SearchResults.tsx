import { ChangeEvent, useState } from 'react';
import { useRouter } from 'next/router';
import useSWR from 'swr';
import useTranslation from 'next-translate/useTranslation';
import { H4 } from '@blueprintjs/core';
import { packagesWithVersionsFetcher } from '../utils/api';
import styles from '../styles/PackageList.module.css';
import { calculateStartEndIndex, getSearchUrlAndInput } from '../utils';
import PageLimitSelect from './PageLimitSelect';
import Pagination from './Pagination';
import PackageList from './PackageList';

const SearchResults = () => {
  const { t } = useTranslation('common');
  const { query } = useRouter();
  const { q, type, categories } = query as {
    q: string;
    type: string;
    categories: string;
  };
  const [currentPage, setCurrentPage] = useState(1);
  const [selectedLimit, setSelectedLimit] = useState(20);

  const { apiSearchUrl, searchInput } = getSearchUrlAndInput(
    q,
    currentPage,
    selectedLimit,
    type,
    categories
  );

  const { data: searchResults, error } = useSWR(
    `${apiSearchUrl}`,
    packagesWithVersionsFetcher
  );

  if (error) {
    return (
      <div className={styles.packageContainer}>
        <H4>{t('searchResultsFor', { searchInput })}</H4>
        <div>{t('failedLoading')}</div>
      </div>
    );
  }

  if (!searchResults) {
    return null;
  }

  const { startIndex, endIndex } = calculateStartEndIndex(
    currentPage,
    selectedLimit,
    searchResults.totalPackages
  );

  const handleLimitChange = (event: ChangeEvent<HTMLSelectElement>) => {
    setSelectedLimit(parseInt(event.target.value, 10));
    setCurrentPage(1);
  };

  const handlePageChange = (newPage: number) => {
    setCurrentPage(newPage);
  };

  return (
    <div className={styles.packageContainer}>
      <H4>{t('searchResultsFor', { searchInput })}</H4>
      {searchResults.packages.length === 0 && <div>{t('noResults')}</div>}
      {searchResults.packages.length > 0 && (
        <div>
          <div className={styles.resultsCountContainer}>
            <PageLimitSelect
              selectedLimit={selectedLimit}
              onChange={handleLimitChange}
            />
            <span className={styles.resultsCount}>
              {t('resultsCount', {
                startIndex,
                endIndex,
                count: searchResults.totalPackages,
              })}
            </span>
          </div>

          <PackageList packages={searchResults.packages} />

          <Pagination
            currentPage={currentPage}
            totalPages={searchResults.totalPages}
            onPageChange={handlePageChange}
          />
        </div>
      )}
    </div>
  );
};

export default SearchResults;
