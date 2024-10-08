import styles from './MoneyDisplay.module.css';

const MoneyDisplay = () => {
  return (
    <section className={styles.container}>
      <h3>Your Balances</h3>
      <div className={styles.balances}>
        <div className={styles.balance}>
          <span>Deposited:</span>
          <span>$1,000.00</span>
        </div>
        <div className={styles.balance}>
          <span>Compounded:</span>
          <span>$1,050.00</span>
        </div>
      </div>
    </section>
  );
};

export default MoneyDisplay;