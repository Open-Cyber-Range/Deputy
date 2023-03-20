import type {Package} from '../interfaces/PackageListInterface';
import {useRouter} from 'next/router';
import type {Fetcher} from 'swr';
import useSWR from 'swr';
import {CodeBlock, dracula} from 'react-code-blocks';

const codeFetcher: Fetcher<string, string> = async (...url) => fetch(...url).then(async res => res.text());

const CodePreview = ({packageData, filepath}: {packageData: Package; filepath: string}) => {
  const {asPath} = useRouter();
  const nameAndVersion = asPath.split('/packages/')[1];
  const {data: codeData} = useSWR('/api/v1/package/' + nameAndVersion + '/path/' + filepath, codeFetcher);

  if (!codeData || !packageData.content.preview) {
    return null;
  }

  return (
    <pre>
      <hr/>
      <h4>{filepath}</h4>
      <CodeBlock
        text={codeData}
        language={filepath.split('.').slice(-1)[0].toLowerCase()}
        showLineNumbers={true}
        theme={dracula}
      />
    </pre>
  );
};

export default CodePreview;
