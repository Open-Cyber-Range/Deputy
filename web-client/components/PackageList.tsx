import type {Fetcher} from 'swr';
import useSWR from 'swr';
import styles from '../styles/PackageList.module.css';
import type {PackageList} from '../interfaces/PackageListInterface';
import {Card, Elevation} from '@blueprintjs/core';
import type {SWRResponse} from 'swr/dist/types';

const fetcher: Fetcher<PackageList[], string> = async (...url) => fetch(...url).then(async res => res.json());

const PackageListView = () => {
	const {data: packageList, error}: SWRResponse<PackageList[], string> = useSWR('/api/v1/package', fetcher);
	if (error) {
		return <div>Failed to load</div>;
	}

	if (!packageList) {
		return null;
	}

	return (
		<div className={styles.packageContainer}>
			<ul className={styles.noBullets}>
				{packageList.map((deputyPackage: PackageList) =>
					<li key={deputyPackage.package.version}>
						<Card interactive={false} elevation={Elevation.ONE}>
							<span><a href='#' className={styles.name}>{deputyPackage.package.name}</a></span>
							<span className={styles.version}>{deputyPackage.package.version}</span>
							<div className={styles.description}>{deputyPackage.package.description}</div>
						</Card>
					</li>)}
			</ul>
		</div>
	);
};

export default PackageListView;
