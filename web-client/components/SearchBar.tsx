import useTranslation from 'next-translate/useTranslation';
import { useState } from 'react';
import { useRouter } from 'next/router';
import styles from '../styles/MainNavbar.module.css';

const SearchBar = () => {
  const { t } = useTranslation('common');
  const [searchInput, setSearchInput] = useState('');
  const router = useRouter();

  const handleSearchSubmit = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    router.push(`/search?q=${encodeURIComponent(searchInput)}`);
  };

  return (
    <form onSubmit={handleSearchSubmit}>
      <input
        className={`bp4-input ${styles.searchbox}`}
        type="search"
        placeholder={t('searchbox')}
        dir="auto"
        value={searchInput}
        onChange={(event) => setSearchInput(event.target.value)}
      />
    </form>
  );
};

export default SearchBar;
