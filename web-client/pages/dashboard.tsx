import {Button} from '@blueprintjs/core';
import styles from '../styles/Dashboard.module.css';

const Dashboard = () => (
  <div>
    <main className={styles.dashboard}>
      <h2>Welcome to Deputy Digital Library</h2>
      <Button large> Documentation</Button> <br/>
      <input className='bp4-input' type='search' placeholder='Search packages' dir='auto'/>
    </main>
  </div>
);
export default Dashboard;
