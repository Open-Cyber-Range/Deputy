import type { NextPage } from "next";
import Head from "next/head";
import styles from "../styles/Home.module.css";
import PackageListView from "../components/PackageList";

const Home: NextPage = () => {
  return (
    <div className={styles.container}>
      <Head>
        <title>Deputy Frontend</title>
        <meta name="description" content="Generated by create next app" />
      </Head>

      <header className={styles.header}>
        <a>I&apos;m a searchbox</a>
      </header>

      <main className={styles.main}>
        {PackageListView()}
      </main>

      <footer className={styles.footer}>
        <a>I&apos;m a footer</a>
      </footer>
    </div>
  );
};

export default Home;
