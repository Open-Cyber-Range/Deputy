import { Card, Elevation } from '@blueprintjs/core';
import Link from 'next/link';
import styles from '../styles/PackageList.module.css';
import { getLatestVersion, formatBytes } from '../utils';
import { PackageWithVersions } from '../interfaces/Package';

const PackageList = ({ packages }: { packages: PackageWithVersions[] }) => {
  return (
    <ul className={styles.noBullets}>
      {packages.map((deputyPackage) => {
        const latestVersion = getLatestVersion(deputyPackage);
        return (
          latestVersion && (
            <li
              className={styles.packageCard}
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
                <span className={styles.version}>{latestVersion.version}</span>
                <span>{formatBytes(latestVersion.packageSize)}</span>
                <div>{deputyPackage.description}</div>
              </Card>
            </li>
          )
        );
      })}
    </ul>
  );
};

export default PackageList;
