import { Button } from '@blueprintjs/core';
import styles from '../styles/PackageList.module.css';

const Pagination = ({
  currentPage,
  totalPages,
  onPageChange,
}: {
  currentPage: number;
  totalPages: number;
  onPageChange: (page: number) => void;
}) => {
  return (
    <div className={styles.pagination}>
      <Button
        onClick={() => onPageChange(Math.max(currentPage - 1, 1))}
        disabled={currentPage === 1}
        icon="chevron-left"
      />
      <span>
        {currentPage} / {totalPages}
      </span>
      <Button
        onClick={() => onPageChange(Math.min(currentPage + 1, totalPages))}
        disabled={currentPage === totalPages}
        icon="chevron-right"
      />
    </div>
  );
};

export default Pagination;
