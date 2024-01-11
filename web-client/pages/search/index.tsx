import styles from '../../styles/Packages.module.css';
import SearchResults from '../../components/SearchResults';

const Search = () => (
  <div className={styles.packagelist}>
    <SearchResults />
  </div>
);

export default Search;
