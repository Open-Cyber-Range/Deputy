import useTranslation from 'next-translate/useTranslation';
import { useState, useEffect } from 'react';
import useSWR from 'swr';
import Link from 'next/link';
import { packageFetcher, packagesWithVersionsFetcher } from '../utils/api';
import { extractAndRemoveTypeAndCategory, getLatestVersion } from '../utils';
import styles from '../styles/MainNavbar.module.css';

const SearchBar = () => {
  const { t } = useTranslation('common');
  const [searchInput, setSearchInput] = useState('');
  const [debouncedSearchInput, setDebouncedSearchInput] = useState('');
  const { data: packageList } = useSWR(
    '/api/v1/package',
    packagesWithVersionsFetcher
  );

  const parsedSearchInput =
    extractAndRemoveTypeAndCategory(debouncedSearchInput);
  let searchUrl = debouncedSearchInput
    ? `/api/v1/search?search_term=${encodeURIComponent(
        parsedSearchInput.remainingString
      )}`
    : null;
  if (parsedSearchInput.type) {
    if (searchUrl) {
      searchUrl += `&type=${encodeURIComponent(parsedSearchInput.type)}`;
    }
  }
  if (parsedSearchInput.categories) {
    if (searchUrl) {
      searchUrl += `&categories=${encodeURIComponent(
        parsedSearchInput.categories
      )}`;
    }
  }
  const { data: searchResults } = useSWR(searchUrl, packageFetcher);

  useEffect(() => {
    const debounceTimer = setTimeout(() => {
      setDebouncedSearchInput(searchInput);
    }, 100);

    return () => {
      clearTimeout(debounceTimer);
    };
  }, [searchInput]);

  return (
    <>
      <input
        className={`bp4-input ${styles.searchbox}`}
        type="search"
        placeholder={t('searchbox')}
        dir="auto"
        value={searchInput}
        onChange={(e) => setSearchInput(e.target.value)}
      />
      {searchResults && debouncedSearchInput && (
        <div className={styles.searchResults}>
          <ul>
            {searchResults.map((result) => {
              const matchedPackage = packageList?.packages?.find(
                (pkg) => pkg.name === result.name
              );
              if (matchedPackage) {
                const latestVersion = getLatestVersion(matchedPackage);
                return (
                  <li key={result.id}>
                    <Link
                      href={`/packages/${result.name}/${latestVersion?.version}`}
                    >
                      {result.name}
                    </Link>
                  </li>
                );
              }
              return null;
            })}
          </ul>
        </div>
      )}
    </>
  );
};

export default SearchBar;
