import type {Fetcher} from 'swr';
import useSWR from 'swr';
import styles from '../styles/PackageList.module.css';
import type {Package} from '../interfaces/PackageListInterface';
import {Card, Elevation} from '@blueprintjs/core';
import type {SWRResponse} from 'swr/dist/types';

const fetcher: Fetcher<Package[], string> = async (...url) => fetch(...url).then(async res => res.json());

const PackageListView = () => {
	const {data: packageList, error}: SWRResponse<Package[], string> = useSWR('/api/v1/package', fetcher);
	if (error) {
		return <div>Failed to load</div>;
	}

	if (!packageList) {
		return null;
	}

	return (
		<div className={styles.packageContainer}>
			<ul className={styles.noBullets}>
				{packageList.map((deputyPackage: Package) =>
					<li key={deputyPackage.version}>
						<Card interactive={false} elevation={Elevation.ONE}>
							<span><a href='#' className={styles.name}>{deputyPackage.name}</a></span>
							<span className={styles.version}>{deputyPackage.version}</span>
							<div className={styles.description}>{deputyPackage.description}</div>
						</Card>
					</li>)}
			</ul>
		</div>
	);
};

export default PackageListView;