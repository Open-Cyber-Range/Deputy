import useSWR from 'swr';
import useTranslation from 'next-translate/useTranslation';
import { ChangeEvent, useState } from 'react';
import Link from 'next/link';
import { packagesWithVersionsFetcher } from '../utils/api';
import PageLimitSelect from './PageLimitSelect';
import Pagination from './Pagination';
import PackageList from './PackageList';

const PackageListView = () => {
  const { t } = useTranslation('common');
  const [currentPage, setCurrentPage] = useState(1);
  const [selectedLimit, setSelectedLimit] = useState(20);

  const { data: packageList, error } = useSWR(
    `/api/v1/package?page=${currentPage}&limit=${selectedLimit}`,
    packagesWithVersionsFetcher
  );
  if (!packageList) {
    return <div>{t('loading')} </div>;
  }

  if (error) {
    return <div>{t('failedLoadingPackages')} </div>;
  }
  const handleLimitChange = (event: ChangeEvent<HTMLSelectElement>) => {
    setSelectedLimit(parseInt(event.target.value, 10));
    setCurrentPage(1);
  };

  const handlePageChange = (newPage: number) => {
    setCurrentPage(newPage);
  };

  return (
    <div className="p-[2rem] w-[60rem] max-w-[90%]">
      <div className="flex justify-between">
        <PageLimitSelect
          selectedLimit={selectedLimit}
          onChange={handleLimitChange}
        />
        <Link
          className="bp5-button bp5-small w-56 h-8 bg-cr14-dark-blue rounded-md"
          href="/categories"
        >
          <span className="text-white">{t('browseAllCategories')}</span>
        </Link>
      </div>

      <PackageList packages={packageList.packages} />

      <Pagination
        currentPage={currentPage}
        totalPages={packageList.totalPages}
        onPageChange={handlePageChange}
      />
    </div>
  );
};

export default PackageListView;
