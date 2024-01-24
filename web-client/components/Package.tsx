import { Card, Elevation, Tag } from '@blueprintjs/core';
import Link from 'next/link';
import useTranslation from 'next-translate/useTranslation';
import { Version } from '../interfaces/Package';
import { formatBytes } from '../utils';

const Package = ({
  deputyPackage,
  version,
}: {
  deputyPackage: {
    name: string;
    description?: string;
  };
  version: Version;
}) => {
  const { t } = useTranslation('common');
  return (
    <Card
      key={`${deputyPackage.name}-${version.version}`}
      interactive={false}
      elevation={Elevation.TWO}
      className="mt-[2rem]"
    >
      <div className="flex flex-col gap-8">
        <div className="flex justify-between items-end">
          <span>
            <Link
              href={`/packages/${deputyPackage.name}/${version.version}`}
              className="decoration-0 font-bold text-xl text-nowrap text-[#0082be]"
            >
              {deputyPackage.name}
            </Link>
          </span>
          <div className="flex gap-4">
            {version.packageSize > 0 && (
              <Tag large minimal round icon="bring-data">
                {formatBytes(version.packageSize)}
              </Tag>
            )}
            <Tag large minimal round>
              v{version.version}
            </Tag>
            {version.isYanked && (
              <Tag large minimal round intent="danger">
                {t('yanked')}
              </Tag>
            )}
          </div>
        </div>
        <div>{deputyPackage.description}</div>
      </div>
    </Card>
  );
};

export default Package;
