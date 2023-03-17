import type {Fetcher} from 'swr';
import useSWR from 'swr';
import styles from '../styles/PackageList.module.css';
import type {Package, PackageMetadata} from '../interfaces/PackageListInterface';
import {Card, Elevation} from '@blueprintjs/core';
import type {SWRResponse} from 'swr/dist/types';
import useTranslation from 'next-translate/useTranslation';
import {useRouter} from 'next/router';
import {Tab, TabList, TabPanel, Tabs} from 'react-tabs';
import 'react-tabs/style/react-tabs.css';
import parse from 'html-react-parser';
import FilePreview from './FilePreview';
import PackageVersions from "./PackageVersions";

const metadataFetcher: Fetcher<PackageMetadata, string> = async (...url) => fetch(...url).then(async res => res.json());
const packageFetcher: Fetcher<Package, string> = async (...url) => fetch(...url).then(async res => res.json());

const PackageDetailView = () => {
  const {t} = useTranslation('common');
  const {asPath} = useRouter();

  const nameAndVersion = asPath.split('/packages/')[1];
  const {data: packageMetadata, error: detailError}: SWRResponse<PackageMetadata, string> = useSWR('/api/v1/package/' + nameAndVersion + '/metadata', metadataFetcher);
  const {data: packageData, error: packageError}: SWRResponse<Package, string> = useSWR('/api/v1/package/' + nameAndVersion + '/toml', packageFetcher);
  if (!packageMetadata || !packageData) {
    return null;
  }

  if (detailError ?? packageError) {
    return <div>{t('failedLoading')} </div>;
  }

  return (
    <div className={styles.packageContainer}>
      <Card interactive={false} elevation={Elevation.ONE}>
        <div className={styles.nameContainer}>
          <span><a href='#' className={styles.name}>{packageData.package.name}</a></span>
          <span className={styles.version}>{packageData.package.version}</span>
          <span className={styles.version}>{packageData.package.license}</span>
          <span className={styles.created_at}>{t('created_at')}: {packageMetadata.created_at}</span>
          <p>{packageData.package.description}</p>
        </div>
        <Tabs>
          <TabList>
            <Tab>Readme</Tab>
            <Tab>{t('versions')}</Tab>
            <Tab disabled={!packageData.content.preview}>{t('preview')}</Tab>
          </TabList>

          <TabPanel>
            <div className={styles.readme}>{ parse(packageMetadata.readme_html) }</div>
          </TabPanel>
          <TabPanel>
            <PackageVersions packageName={packageMetadata.name}/>
          </TabPanel>
          <TabPanel>
            <FilePreview packageData={packageData} />
          </TabPanel>
        </Tabs>

      </Card>
    </div>
  );
};

export default PackageDetailView;
