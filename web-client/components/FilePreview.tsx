import {useRouter} from 'next/router';
import type {Package} from '../interfaces/PackageListInterface';
import {ContentType} from '../interfaces/PackageListInterface';
import Image from 'next/image';
import styles from '../styles/PackageList.module.css';

const FilePreview = ({packageData}: {packageData: Package}) => {
  const {asPath} = useRouter();
  const nameAndVersion = asPath.split('/packages/')[1];

  if (packageData.content.type === ContentType.Picture && packageData.picture) {
    return (
      <Image
        className={styles.nextImage}
        src={'/api/v1/package/' + nameAndVersion + '/path/' + packageData.picture.file_path}
        alt={'Package contents can\'t be displayed'}
        width={10000}
        height={10000}
      />
    );
  }

  if (packageData.content.type === ContentType.Video && packageData.video) {
    return (
      <video
        controls
        className={styles.nextImage}
        title={'Package contents can\'t be displayed'}
        src={'/api/v1/package/' + nameAndVersion + '/path/' + packageData.video.file_path}/>
    );
  }

  return null;
};

export default FilePreview;
