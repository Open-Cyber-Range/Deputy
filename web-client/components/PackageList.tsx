import useSWR from "swr";
import styles from "../styles/PackageList.module.css";
import {PackageList} from "../interfaces/PackageListInterface";
import { Card, Elevation } from "@blueprintjs/core";

const fetcher = (args: RequestInfo) => fetch(args).then((res) => res.json())

function GetAllPackages() {
  const {data, error} = useSWR('/api/v1/package', fetcher)
  if (error) return <div>Failed to load</div>
  if (!data) return <div>Loading...</div>
  return (
    <ul className={styles.noBullets}>
      {data.map((deputyPackage: PackageList) =>
        <li key={deputyPackage.package.version}>
          <Card interactive={false} elevation={Elevation.ONE}>
            <span><a href="#" className={styles.name}>{deputyPackage.package.name}</a></span>
            <span className={styles.version}>{deputyPackage.package.version}</span>
            <div className={styles.description}>{deputyPackage.package.description}</div>
          </Card>
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
