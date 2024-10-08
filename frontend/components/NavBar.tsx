import Link from 'next/link';
import styles from './NavBar.module.css';

const NavBar = () => {
  return (
    <nav className={styles.nav}>
      <h1 className={styles.logo}>Vaultpay</h1>
      <div className={styles.links}>
        <Link href="/user" legacyBehavior>
          <a>User Dashboard</a>
        </Link>
        <Link href="/vendor" legacyBehavior>
          <a>Vendor Dashboard</a>
        </Link>
      </div>
    </nav>
  );
};

export default NavBar;