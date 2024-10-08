import type { NextPage } from 'next';
import NavBar from '../components/NavBar';
import OfferForm from '../components/OfferForm';
import SubscriptionList from '../components/SubscriptionList';
import styles from '../styles/Vendor.module.css';

const Vendor: NextPage = () => {
  return (
    <>
      <NavBar />
      <main className={styles.main}>
        <h2>Vendor Dashboard</h2>
        <div className={styles.section}>
          <OfferForm />
        </div>
        <div className={styles.section}>
          <SubscriptionList />
        </div>
      </main>
    </>
  );
};

export default Vendor;