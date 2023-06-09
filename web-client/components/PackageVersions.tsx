import { Card, Elevation } from '@blueprintjs/core';
import Link from 'next/link';
import useSWR from 'swr';
import useTranslation from 'next-translate/useTranslation';
import styles from '../styles/PackageList.module.css';
import { packageVersionsFethcer } from '../utils/api';

const PackageVersions = ({ packageName }: { packageName: string }) => {
  const { t } = useTranslation('common');
  const { data: packageVersions, error: allPackageVersions } = useSWR(
    () => `/api/v1/package/${packageName}`,
    packageVersionsFethcer
  );

  if (!packageVersions) {
    return null;
  }

  if (allPackageVersions) {
    return <div>{t('failedLoading')} </div>;
  }

  return (
    <div>
      <ul className={styles.noBullets}>
        {packageVersions.map((deputyPackage) => (
          <li key={deputyPackage.version}>
            <Card interactive={false} elevation={Elevation.ONE}>
              <span>
                <Link
                  href={`/packages/${packageName}/${deputyPackage.version}`}
                  className={styles.name}
                >
                  {packageName}
                </Link>
              </span>
              <span className={styles.version}>{deputyPackage.version}</span>
              {/* <div className={styles.description}>
                {deputyPackage.description}
              </div> */}
            </Card>
          </li>
        ))}
      </ul>
    </div>
  );
};

export default PackageVersions;
