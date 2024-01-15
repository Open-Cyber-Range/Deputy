import useTranslation from 'next-translate/useTranslation';
import { FormEvent, useState } from 'react';
import { useRouter } from 'next/router';
import { Icon, InputGroup } from '@blueprintjs/core';
import { getEncodedSearchUrl } from '../utils';
import styles from '../styles/MainNavbar.module.css';

const SearchBar = () => {
  const { t } = useTranslation('common');
  const [searchInput, setSearchInput] = useState('');
  const router = useRouter();

  const handleSearchSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    router.push(`${getEncodedSearchUrl(searchInput.trim())}`);
  };

  const handleBlur = () => {
    setSearchInput('');
  };

  return (
    <form onSubmit={handleSearchSubmit}>
      <InputGroup
        className={styles.searchbox}
        leftIcon={<Icon icon="search" />}
        type="search"
        placeholder={t('searchbox')}
        value={searchInput}
        onChange={(event) => {
          setSearchInput(event.target.value);
        }}
        onBlur={handleBlur}
      />
    </form>
  );
};

export default SearchBar;
