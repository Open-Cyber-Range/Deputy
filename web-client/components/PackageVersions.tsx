import styles from '../styles/PackageList.module.css';
import type {PackageMetadata} from '../interfaces/PackageListInterface';
import {Card, Elevation} from '@blueprintjs/core';
import Link from 'next/link';
import type {Fetcher} from 'swr';
import useSWR from 'swr';
import type {SWRResponse} from 'swr/dist/types';
import useTranslation from 'next-translate/useTranslation';

const versionFetcher: Fetcher<PackageMetadata[], string> = async (...url) => fetch(...url).then(async res => res.json());

const PackageVersions = ({packageName}: {packageName: string}) => {
  const {t} = useTranslation('common');
  const {data: packageVersions, error: versionError}: SWRResponse<PackageMetadata[], string> = useSWR(() => '/api/v1/package/' + packageName + '/all_versions', versionFetcher);

  if (!packageVersions) {
    return null;
  }

  if (versionError) {
    return <div>{t('failedLoading')} </div>;
  }

  return (
    <div>
      <ul className={styles.noBullets}>
        {packageVersions.map((deputyPackage: PackageMetadata) =>
          <li key={deputyPackage.version}>
            <Card interactive={false} elevation={Elevation.ONE}>
              <span><Link href={'/packages/' + deputyPackage.name + '/' + deputyPackage.version}
                className={styles.name}>{deputyPackage.name}</Link></span>
              <span className={styles.version}>{deputyPackage.version}</span>
              <div className={styles.description}>{deputyPackage.description}</div>
            </Card>
          </li>)}
      </ul>
    </div>
  );
};

export default PackageVersions;
