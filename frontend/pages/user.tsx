import type { NextPage } from 'next';
import NavBar from '../components/NavBar';
import DepositForm from '../components/DepositForm';
import MoneyDisplay from '../components/MoneyDisplay';
import SubscriptionForm from '../components/SubscriptionForm';
import styles from '../styles/User.module.css';

const User: NextPage = () => {
  return (
    <>
      <NavBar />
      <main className={styles.main}>
        <h2>User Dashboard</h2>
        <div className={styles.section}>
          <MoneyDisplay />
        </div>
        <div className={styles.section}>
          <DepositForm />
        </div>
        <div className={styles.section}>
          <SubscriptionForm />
        </div>
      </main>
    </>
  );
};

export default User;