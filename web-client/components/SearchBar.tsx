import useTranslation from 'next-translate/useTranslation';
import { useState, useEffect } from 'react';
import useSWR from 'swr';
import Link from 'next/link';
import { packagesWithVersionsFetcher } from '../utils/api';
import { extractAndRemoveTypeAndCategory, getLatestVersion } from '../utils';
import styles from '../styles/MainNavbar.module.css';

const SearchBar = () => {
  const { t } = useTranslation('common');
  const [searchInput, setSearchInput] = useState('');
  const [debouncedSearchInput, setDebouncedSearchInput] = useState('');

  const parsedSearchInput =
    extractAndRemoveTypeAndCategory(debouncedSearchInput);
  let searchUrl = debouncedSearchInput
    ? `/api/v1/package?search_term=${encodeURIComponent(
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
  const { data: searchResults } = useSWR(
    searchUrl,
    packagesWithVersionsFetcher
  );

  useEffect(() => {
    const debounceTimer = setTimeout(() => {
      setDebouncedSearchInput(searchInput);
    }, 200);

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
            {searchResults.packages.map((result) => {
              const latestVersion = getLatestVersion(result);
              return (
                <li key={result.id}>
                  <Link
                    href={`/packages/${result.name}/${latestVersion?.version}`}
                  >
                    {result.name}
                  </Link>
                </li>
              );
              return null;
            })}
          </ul>
        </div>
      )}
    </>
  );
};

export default SearchBar;
