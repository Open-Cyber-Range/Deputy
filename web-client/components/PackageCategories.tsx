import { Card, Elevation } from '@blueprintjs/core';
import Link from 'next/link';
import styles from '../styles/PackageList.module.css';
import { Category } from '../interfaces/Package';

const PackageCategories = ({
  packageCategories,
}: {
  packageCategories: Category[] | undefined;
}) => {
  if (!packageCategories) {
    return null;
  }

  return (
    <div>
      <ul className={styles.noBullets}>
        {packageCategories.map((category) => (
          <li key={category.id}>
            <Card interactive={false} elevation={Elevation.ONE}>
              <span>
                <Link href="/packages" className={styles.name}>
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
