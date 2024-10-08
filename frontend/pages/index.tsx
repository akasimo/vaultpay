import type { NextPage } from 'next';
import NavBar from '../components/NavBar';
import styles from '../styles/Home.module.css';
import Link from 'next/link';

const Home: NextPage = () => {
  return (
    <>
      <NavBar />
      <main className={styles.main}>
        <h2>Welcome to Vaultpay</h2>
        <p>
          Manage your subscriptions and payments seamlessly with our state-of-the-art crypto solutions.
        </p>
        <Link href="/user" legacyBehavior>
          <a className={styles.button}>Get Started</a>
        </Link>
      </main>
    </>
  );
};

export default Home;