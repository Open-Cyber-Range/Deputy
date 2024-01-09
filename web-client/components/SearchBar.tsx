import useTranslation from 'next-translate/useTranslation';
import { useState } from 'react';
import { useRouter } from 'next/router';
import { Icon, InputGroup } from '@blueprintjs/core';
import { extractAndRemoveTypeAndCategory } from '../utils';

const SearchBar = () => {
  const { t } = useTranslation('common');
  const [searchInput, setSearchInput] = useState('');
  const router = useRouter();

  const parsedSearchInput = extractAndRemoveTypeAndCategory(searchInput);

  let searchUrl = searchInput
    ? `/search?q=${encodeURIComponent(parsedSearchInput.remainingString)}`
    : ``;
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

  const handleSearchSubmit = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    router.push(`${searchUrl}`);
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
