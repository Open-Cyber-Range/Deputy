import useSWR from 'swr';
import { Card, Elevation } from '@blueprintjs/core';
import Link from 'next/link';
import useTranslation from 'next-translate/useTranslation';
import { packagesWithVersionsFetcher } from '../utils/api';
import { formatBytes, getLatestVersion } from '../utils';
import styles from '../styles/PackageList.module.css';

const PackageListView = () => {
  const { t } = useTranslation('common');

  const { data: packageList, error } = useSWR(
    '/api/v1/package',
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
      <ul className={styles.noBullets}>
        {packageList.map((deputyPackage) => {
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
                  <span
                    className={styles.yanked}
                    hidden={!latestVersion.is_yanked}
                  >
                    {t('yanked')}
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
    </div>
  );
};

export default PackageListView;
