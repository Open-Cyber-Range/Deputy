import useSWR from "swr";
import styles from "../styles/Home.module.css";
import {PackageList} from "../interfaces/PackageListInterface";

const fetcher = (args: RequestInfo) => fetch(args).then((res) => res.json())

function GetAllPackages() {
  const {data, error} = useSWR('/api/v1/package', fetcher)
  if (error) return <div>Failed to load</div>
  if (!data) return <div>Loading...</div>
  return (
    <ul className={styles.noBullets}>
      {data.map((deputyPackage: PackageList) =>
        <li key={deputyPackage.package.version}>
          <div className={styles.packageRow}>
            <div key={null} className={styles.descriptionBox}>
              <div>
                <span className={styles.name}>{deputyPackage.package.name}</span>
                <span className={styles.version}>{deputyPackage.package.version}</span>
              </div>
              <div className={styles.description}>{deputyPackage.package.description}</div>
            </div>
          </div>
        </li>)}
    </ul>
  )
}

const PackageListView = () => {
  return (
  <div className={styles.packageContainer}>
    {GetAllPackages()}
  </div>
  );
}

export default PackageListView;
