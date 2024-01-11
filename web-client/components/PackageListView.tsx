import useSWR from 'swr';
import useTranslation from 'next-translate/useTranslation';
import { ChangeEvent, useState } from 'react';
import { packagesWithVersionsFetcher } from '../utils/api';
import styles from '../styles/PackageList.module.css';
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
  if (error) {
    return <div>{t('failedLoading')} </div>;
  }

  if (!packageList) {
    return null;
  }

  const handleLimitChange = (event: ChangeEvent<HTMLSelectElement>) => {
    setSelectedLimit(parseInt(event.target.value, 10));
    setCurrentPage(1);
  };

  const handlePageChange = (newPage: number) => {
    setCurrentPage(newPage);
  };

  return (
    <div className={styles.packageContainer}>
      <div>
        <PageLimitSelect
          selectedLimit={selectedLimit}
          onChange={handleLimitChange}
        />
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
