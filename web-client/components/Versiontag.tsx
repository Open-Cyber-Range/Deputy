import { Intent, Tag } from '@blueprintjs/core';

const VersionTag = ({
  version,
  intent = 'none',
}: {
  version: string;
  intent?: Intent;
}) => (
  <Tag large minimal round intent={intent}>
    v{version}
  </Tag>
);

VersionTag.defaultProps = {
  intent: 'none',
};

export default VersionTag;
