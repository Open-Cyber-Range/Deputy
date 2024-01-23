import { Card, Elevation } from '@blueprintjs/core';
import Link from 'next/link';
import styles from '../styles/PackageList.module.css';
import { Category } from '../interfaces/Package';
import { sortCategories } from '../utils/sort';

const PackageCategories = ({
  packageCategories,
}: {
  packageCategories: Category[] | undefined;
}) => {
  if (!packageCategories) {
    return null;
  }

  const sortedPackageCategories = packageCategories.sort(sortCategories);

  return (
    <div>
      <ul className={styles.noBullets}>
        {sortedPackageCategories.map((category) => (
          <li key={category.id}>
            <Card interactive={false} elevation={Elevation.ONE}>
              <span>
                <Link
                  href={`/search?q=&categories=${category.name}`}
                  className={styles.name}
                >
                  {category.name}
                </Link>
              </span>
            </Card>
          </li>
        ))}
      </ul>
    </div>
  );
};

export default PackageCategories;
