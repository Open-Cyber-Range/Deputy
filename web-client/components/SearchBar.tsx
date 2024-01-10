import useTranslation from 'next-translate/useTranslation';
import { useEffect, useState } from 'react';
import { useRouter } from 'next/router';
import {
  Icon,
  InputGroup,
  OverlayToaster,
  Position,
  Toast,
} from '@blueprintjs/core';
import { extractAndRemoveTypeAndCategory } from '../utils';

const SearchBar = () => {
  const { t } = useTranslation('common');
  const [searchInput, setSearchInput] = useState('');
  const router = useRouter();
  const [isEmptySearch, setIsEmptySearch] = useState(false);

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
    if (!searchInput.trim()) {
      setIsEmptySearch(true);
      return;
    }
    setIsEmptySearch(false);
    router.push(`${searchUrl}`);
  };

  const handleBlur = () => {
    setSearchInput('');
    setIsEmptySearch(false);
  };

  useEffect(() => {
    let timer: NodeJS.Timeout;
    if (isEmptySearch) {
      timer = setTimeout(() => setIsEmptySearch(false), 5000);
    }
    return () => {
      if (timer) {
        clearTimeout(timer);
      }
    };
  }, [isEmptySearch]);

  return (
    <form onSubmit={handleSearchSubmit}>
      <OverlayToaster position={Position.TOP}>
        {isEmptySearch && (
          <Toast
            icon="warning-sign"
            intent="warning"
            message={t('emptySearchInput')}
          />
        )}
      </OverlayToaster>
      <InputGroup
        className="m-[1rem]"
        leftIcon={<Icon icon="search" />}
        type="search"
        placeholder={t('searchbox')}
        value={searchInput}
        onChange={(event) => {
          setSearchInput(event.target.value);
          setIsEmptySearch(false);
        }}
        onBlur={handleBlur}
      />
    </form>
  );
};

export default SearchBar;
