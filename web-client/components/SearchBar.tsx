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
      <InputGroup
        className="m-[1rem]"
        leftIcon={<Icon icon="search" />}
        type="search"
        placeholder={t('searchbox')}
        onChange={(event) => setSearchInput(event.target.value)}
      />
    </form>
  );
};

export default SearchBar;
