import { HTMLSelect } from '@blueprintjs/core';
import { ChangeEvent } from 'react';

const PageLimitSelect = ({
  selectedLimit,
  onChange,
}: {
  selectedLimit: number;
  onChange: (event: ChangeEvent<HTMLSelectElement>) => void;
}) => {
  return (
    <HTMLSelect
      id="limit"
      iconName="caret-down"
      value={selectedLimit}
      onChange={onChange}
    >
      <option value="5">5</option>
      <option value="10">10</option>
      <option value="20">20</option>
      <option value="50">50</option>
    </HTMLSelect>
  );
};

export default PageLimitSelect;
