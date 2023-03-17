import {useRouter} from 'next/router';
import type {Package} from '../interfaces/PackageListInterface';
import {PreviewType} from '../interfaces/PackageListInterface';
import type {Slide} from 'yet-another-react-lightbox';
import Lightbox from 'yet-another-react-lightbox';
import Thumbnails from 'yet-another-react-lightbox/plugins/thumbnails';
import Fullscreen from 'yet-another-react-lightbox/plugins/fullscreen';
import Inline from 'yet-another-react-lightbox/plugins/inline';
import Video from 'yet-another-react-lightbox/plugins/video';
import 'yet-another-react-lightbox/styles.css';
import 'yet-another-react-lightbox/plugins/thumbnails.css';

const FilePreview = ({packageData}: {packageData: Package}) => {
  const {asPath} = useRouter();
  const nameAndVersion = asPath.split('/packages/')[1];
  const slides: Slide[] = [];

  if (!packageData.content.preview) {
    return null;
  }

  packageData.content.preview.forEach(preview => {
    if (preview.type) {
      if (preview.type === PreviewType.Picture) {
        preview.value.forEach(filepath => {
          slides.push({
            height: 10000, width: 10000,
            src: '/api/v1/package/' + nameAndVersion + '/path/' + filepath,
          });
        });
      }

      if (preview.type === PreviewType.Video) {
        preview.value.forEach(filepath => {
          slides.push({height: 10000, width: 10000, type: 'video', sources: [{
            src: '/api/v1/package/' + nameAndVersion + '/path/' + filepath,
            type: 'video/mp4',
          }],
          },
          );
        });
      }
    }
  });

  return (
    <Lightbox
      slides={slides}
      inline={{style: {aspectRatio: '3 / 2'}}}
      plugins={[Video, Thumbnails, Inline, Fullscreen]}
    />
  );
};

export default FilePreview;
