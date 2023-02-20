import {useRouter} from 'next/router';
import type {Package} from '../interfaces/PackageListInterface';
import {ContentType} from '../interfaces/PackageListInterface';
import Image from 'next/image';
import styles from '../styles/PackageList.module.css';

const FilePreview = ({packageData}: {packageData: Package}) => {
  const {asPath} = useRouter();
  const nameAndVersion = asPath.split('/packages/')[1];
  if (!packageData) {
    return null;
  }

  if (packageData.content.type === ContentType.Picture && packageData.picture) {
    return (
      <div className={styles.imageContainer}>
        <Image
          className={styles.nextImage}
          src={'/api/v1/package/' + nameAndVersion + '/path/' + packageData.picture.file_path}
          alt={'package image'}
          width={10000}
          height={10000}
        />
      </div>
    );
  }

  return null;
};

export default FilePreview;
