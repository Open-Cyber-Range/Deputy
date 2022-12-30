import PackageListView from '../components/PackageList';
import styles from '../styles/Packages.module.css';

const Packages = () => (
  <>
    <div className={styles.packagelist}>
      <PackageListView/>
    </div>

  </>
);
export default Packages;
