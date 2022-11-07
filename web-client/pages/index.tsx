import type {NextPage} from 'next';
import Head from 'next/head';
import styles from '../styles/Home.module.css';
import MainNavbar from '../components/MainNavbar';
import Dashboard from './dashboard';

const Home: NextPage = () => (
  <div>
    <MainNavbar/>
    <Head>
      <title>Deputy Frontend</title>
      <meta name='description' content='Generated by create next app' />
    </Head>

    <main className={styles.main}>
      <Dashboard />
    </main>

    <footer className={styles.footer}>
      <span>I&apos;m a footer</span>
    </footer>
  </div>
);

export default Home;
