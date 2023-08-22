import useSWR from 'swr';
import { Card, Elevation } from '@blueprintjs/core';
import useTranslation from 'next-translate/useTranslation';
import { useRouter } from 'next/router';
import parse from 'html-react-parser';
import { TabList, TabPanel, Tab, Tabs } from 'react-tabs';
import 'react-tabs/style/react-tabs.css';
import { ParsedUrlQuery } from 'querystring';
import styles from '../styles/PackageList.module.css';
import { packageTOMLFetcher, packageVersionFethcer } from '../utils/api';
import PackageVersions from './PackageVersions';
import FilePreview from './FilePreview';
import { displayLocalTime, formatBytes } from '../utils';

interface DetailParams extends ParsedUrlQuery {
  name: string;
  version: string;
}

const PackageDetailView = () => {
  const { t } = useTranslation('common');
  const { query } = useRouter();
  const { name, version } = query as DetailParams;

  const { data: latestVersion, error: latestVersionError } = useSWR(
    `/api/v1/package/${name}/${version}`,
    packageVersionFethcer
  );

  const { data: packageToml, error: packageTOMLError } = useSWR(
    `/api/v1/package/${name}/${version}/path/package.toml`,
    packageTOMLFetcher
  );

  if (!latestVersion || !packageToml) {
    return null;
  }

  if (latestVersionError || packageTOMLError) {
    return <div>{t('failedLoading')} </div>;
  }

  return (
    <div className={styles.packageContainer}>
      <Card interactive={false} elevation={Elevation.ONE}>
        <span className={styles.name}>{name}</span>
        <span className={styles.version}>{latestVersion.version}</span>
        <span className={styles.version}>{latestVersion.license}</span>
        <span className={styles.created_at}>
          Created at: {displayLocalTime(latestVersion.created_at)}
        </span>
        <span className={styles.packageSize}>
          {formatBytes(latestVersion.package_size)}
        </span>

        <Tabs className="pt-[2rem] pb-[2rem]">
          <TabList>
            <Tab>Readme</Tab>
            <Tab>{t('versions')}</Tab>
            <Tab disabled={!packageToml.content.preview}>{t('preview')}</Tab>
          </TabList>

          <TabPanel>
            <div className={styles.readme}>
              {parse(latestVersion.readme_html)}
            </div>
          </TabPanel>
          <TabPanel>
            <PackageVersions packageName={name} />
          </TabPanel>
          <TabPanel>
            <FilePreview packageData={packageToml} />
          </TabPanel>
        </Tabs>
      </Card>
    </div>
  );
};

export default PackageDetailView;
