import { Card, Elevation, H4 } from '@blueprintjs/core';
import Link from 'next/link';
import styles from '../styles/PackageList.module.css';

const PackageCategories = ({
  packageCategories,
}: {
  packageCategories: string[] | undefined;
}) => {
  if (!packageCategories) {
    return null;
  }

  return (
    <div>
      <ul className={styles.noBullets}>
        {packageCategories.sort().map((category) => (
          <li key={category}>
            <Card interactive={false} elevation={Elevation.ONE}>
              <Link href={`/search?q=&categories=${category}`}>
                <H4 className={styles.name}>{category}</H4>
              </Link>
            </Card>
          </li>
        ))}
      </ul>
    </div>
  );
};

export default PackageCategories;
