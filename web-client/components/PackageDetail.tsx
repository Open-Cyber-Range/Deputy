import { ParsedUrlQuery } from 'querystring';
import useSWR from 'swr';
import { Card, Elevation } from '@blueprintjs/core';
import useTranslation from 'next-translate/useTranslation';
import { useRouter } from 'next/router';
import parse from 'html-react-parser';
import { TabList, TabPanel, Tab, Tabs } from 'react-tabs';
import 'react-tabs/style/react-tabs.css';
import styles from '../styles/PackageList.module.css';
import { packageTOMLFetcher, packageVersionFetcher } from '../utils/api';
import PackageVersions from './PackageVersions';
import FilePreview from './FilePreview';
import { displayLocalTime, formatBytes } from '../utils';
import PackageCategories from './PackageCategories';

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
    packageVersionFetcher
  );

  const { data: packageToml, error: packageTOMLError } = useSWR(
    `/api/v1/package/${name}/${version}/path/package.toml`,
    packageTOMLFetcher
  );

  if (!latestVersion || !packageToml) {
    return null;
  }

  if (latestVersionError || packageTOMLError) {
    return <div>{t('failedLoadingPackages')} </div>;
  }

  return (
    <div className={styles.packageContainer}>
      <Card interactive={false} elevation={Elevation.ONE}>
        <span className={styles.name}>{name}</span>
        <span className={styles.version}>{latestVersion.version}</span>
        <span className={styles.version}>
          {packageToml.content.type.toUpperCase()}
        </span>
        <span className={styles.version}>{latestVersion.license}</span>
        <span>{formatBytes(latestVersion.packageSize)}</span>
        <span className={styles.createdAt}>
          {t('createdAt')}: {displayLocalTime(latestVersion.createdAt)}
        </span>

        <Tabs className="pt-[2rem] pb-[2rem]">
          <TabList>
            <Tab>Readme</Tab>
            <Tab>{t('versions')}</Tab>
            <Tab
              disabled={
                !packageToml.package.categories ||
                packageToml.package.categories.length === 0
              }
            >
              {t('categories')}
            </Tab>
            <Tab disabled={!packageToml.content.preview}>{t('preview')}</Tab>
          </TabList>

          <TabPanel>
            <div className={styles.readme}>
              {parse(latestVersion.readmeHtml || '')}
            </div>
          </TabPanel>
          <TabPanel>
            <PackageVersions packageName={name} />
          </TabPanel>
          <TabPanel>
            <PackageCategories
              packageCategories={packageToml.package.categories}
            />
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
