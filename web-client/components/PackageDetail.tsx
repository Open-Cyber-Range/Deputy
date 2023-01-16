import type {Fetcher} from 'swr';
import useSWR from 'swr';
import styles from '../styles/PackageList.module.css';
import type {Package} from '../interfaces/PackageListInterface';
import {Card, Elevation} from '@blueprintjs/core';
import type {SWRResponse} from 'swr/dist/types';
import useTranslation from 'next-translate/useTranslation';
import {useRouter} from 'next/router';
// @ts-expect-error The library itself has an ambiguous type, not strict enough
import ReactHtmlParser from 'react-html-parser';

const fetcher: Fetcher<Package, string> = async (...url) => fetch(...url).then(async res => res.json());

const PackageDetailView = () => {
  const {t} = useTranslation('common');
  const {asPath} = useRouter();

  const {data: packageDetail, error}: SWRResponse<Package, string> = useSWR('/api/v1/package/' + asPath.split('/packages/')[1] + '/metadata', fetcher);
  if (error) {
    return <div>{t('failedLoading')} </div>;
  }

  if (!packageDetail) {
    return null;
  }

  return (
    <div className={styles.packageContainer}>
      <Card interactive={false} elevation={Elevation.ONE}>
        <span><a href='#' className={styles.name}>{packageDetail.name}</a></span>
        <span className={styles.version}>{packageDetail.version}</span>
        <span className={styles.version}>{packageDetail.license}</span>
        <span className={styles.created_at}>Created at: {packageDetail.created_at}</span>
        <div className={styles.readme}>{ ReactHtmlParser(packageDetail.readme_html) }</div>
      </Card>
    </div>
  );
};

export default PackageDetailView;
