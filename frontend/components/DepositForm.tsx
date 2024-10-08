import { useState } from 'react';
import styles from './DepositForm.module.css';

const DepositForm = () => {
  const [amount, setAmount] = useState('');

  const handleDeposit = (e: React.FormEvent) => {
    e.preventDefault();
    // Mock deposit action
    alert(`Deposited $${amount}`);
    setAmount('');
  };

  return (
    <section className={styles.container}>
      <h3>Deposit Money</h3>
      <form onSubmit={handleDeposit} className={styles.form}>
        <input
          type="number"
          placeholder="Amount in USD"
          value={amount}
          onChange={(e) => setAmount(e.target.value)}
          required
          min="1"
        />
        <button type="submit">Deposit</button>
      </form>
    </section>
  );
};

export default DepositForm;