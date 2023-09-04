import useSWR from 'swr';
import { Button, Card, Elevation, HTMLSelect } from '@blueprintjs/core';
import Link from 'next/link';
import useTranslation from 'next-translate/useTranslation';
import { useState } from 'react';
import { packagesWithVersionsFetcher } from '../utils/api';
import { formatBytes, getLatestVersion } from '../utils';
import styles from '../styles/PackageList.module.css';

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

  return (
    <div className={styles.packageContainer}>
      <div>
        <HTMLSelect
          id="limit"
          value={selectedLimit}
          iconName="caret-down"
          onChange={(e) => {
            setSelectedLimit(parseInt(e.target.value, 10));
            setCurrentPage(1);
          }}
        >
          <option value="5">5</option>
          <option value="10">10</option>
          <option value="20">20</option>
          <option value="50">50</option>
        </HTMLSelect>
      </div>

      <ul className={styles.noBullets}>
        {packageList.packages.map((deputyPackage) => {
          const latestVersion = getLatestVersion(deputyPackage);
          return (
            latestVersion && (
              <li
                className="mt-[2rem]"
                key={`${deputyPackage.name}-${latestVersion.version}`}
              >
                <Card interactive={false} elevation={Elevation.ONE}>
                  <span>
                    <Link
                      href={`/packages/${deputyPackage.name}/${latestVersion.version}`}
                      className={styles.name}
                    >
                      {deputyPackage.name}
                    </Link>
                  </span>
                  <span className={styles.version}>
                    {latestVersion.version}
                  </span>
                  <span className={styles.packageSize}>
                    {formatBytes(latestVersion.package_size)}
                  </span>
                  <div className={styles.description}>
                    {deputyPackage.description}
                  </div>
                </Card>
              </li>
            )
          );
        })}
      </ul>

      <div className="flex flex-row justify-center gap-[1rem] mt-6">
        <Button
          onClick={() => setCurrentPage((prev) => Math.max(prev - 1, 1))}
          disabled={currentPage === 1}
          icon="chevron-left"
        />

        <span>
          {currentPage} / {packageList.total_pages}
        </span>

        <Button
          onClick={() =>
            setCurrentPage((prev) =>
              Math.min(prev + 1, packageList.total_pages)
            )
          }
          disabled={currentPage === packageList.total_pages}
          icon="chevron-right"
        />
      </div>
    </div>
  );
};

export default PackageListView;
