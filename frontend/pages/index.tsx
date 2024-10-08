import type { NextPage } from 'next';
import NavBar from '../components/NavBar';
import styles from '../styles/Home.module.css';

const Home: NextPage = () => {
  return (
    <>
      <NavBar />
      <main className={styles.main}>
        <h2>Welcome to Vaultpay</h2>
        <p>
          Manage your subscriptions and payments seamlessly.
        </p>
      </main>
    </>
  );
};

export default Home;