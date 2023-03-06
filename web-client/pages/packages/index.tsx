import styles from '../../styles/Packages.module.css';
import PackageListView from '../../components/PackageList';

const Packages = () => (
  <>
    <div className={styles.packagelist}>
      <PackageListView/>
    </div>

  </>
);
export default Packages;
