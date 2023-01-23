import type {Fetcher} from 'swr';
import useSWR from 'swr';
import styles from '../styles/PackageList.module.css';
import type {Package} from '../interfaces/PackageListInterface';
import {Card, Elevation} from '@blueprintjs/core';
import type {SWRResponse} from 'swr/dist/types';
import useTranslation from 'next-translate/useTranslation';
import {useRouter} from 'next/router';
import parse from 'html-react-parser';
import Link from 'next/link';

const detailFetcher: Fetcher<Package, string> = async (...url) => fetch(...url).then(async res => res.json());
const versionFetcher: Fetcher<Package[], string> = async (...url) => fetch(...url).then(async res => res.json());

const PackageDetailView = () => {
  const {t} = useTranslation('common');
  const {asPath} = useRouter();

  const {data: packageDetail, error: detailError}: SWRResponse<Package, string> = useSWR('/api/v1/package/' + asPath.split('/packages/')[1] + '/metadata', detailFetcher);
  // @ts-expect-error packageDetail is possibly undefined
  const {data: packageVersions, error: versionError}: SWRResponse<Package[], string> = useSWR(() => '/api/v1/package/' + packageDetail.name + '/all_versions', versionFetcher);
  if (!packageDetail || !packageVersions) {
    return null;
  }

  if (detailError ?? versionError) {
    return <div>{t('failedLoading')} </div>;
  }

  return (
    <div className={styles.packageContainer}>
      <Card interactive={false} elevation={Elevation.ONE}>
        <span><a href='#' className={styles.name}>{packageDetail.name}</a></span>
        <span className={styles.version}>{packageDetail.version}</span>
        <span className={styles.version}>{packageDetail.license}</span>
        <span className={styles.created_at}>Created at: {packageDetail.created_at}</span>
        <div className={styles.readme}>{ parse(packageDetail.readme_html) }</div>
        <div className={styles.versionContainer}>
          <ul className={styles.noBullets}>
            {packageVersions.map((deputyPackage: Package) =>
              <li key={deputyPackage.version}>
                <Card interactive={false} elevation={Elevation.ONE}>
                  <span><Link href={'/packages/' + deputyPackage.name + '/' + deputyPackage.version} className={styles.name}>{deputyPackage.name}</Link></span>
                  <span className={styles.version}>{deputyPackage.version}</span>
                  <div className={styles.description}>{deputyPackage.description}</div>
                </Card>
              </li>)}
          </ul>
        </div>
      </Card>
    </div>
  );
};

export default PackageDetailView;
