import { useState } from 'react';
import styles from './SubscriptionForm.module.css';

const SubscriptionForm = () => {
  const [vendor, setVendor] = useState('');
  const [amount, setAmount] = useState('');
  const [duration, setDuration] = useState('');

  const handleSubscription = (e: React.FormEvent) => {
    e.preventDefault();
    // Mock subscription action
    alert(`Subscribed to ${vendor} for $${amount} over ${duration} months`);
    setVendor('');
    setAmount('');
    setDuration('');
  };

  // Hardcoded list of vendors
  const vendorOptions = [
    { value: '', label: 'Select Vendor' },
    { value: 'Helius', label: 'Helius Rpc Service' },
    { value: 'Amazon', label: 'Amazon Prime' },
    { value: 'Nansen Pro', label: 'Nansen Pro Analytics' },
  ];

  return (
    <section className={styles.container}>
      <h3>Add Subscription</h3>
      <form onSubmit={handleSubscription} className={styles.form}>
        <select
          value={vendor}
          onChange={(e) => setVendor(e.target.value)}
          required
          className={styles.select}
        >
          {vendorOptions.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
        <input
          type="number"
          placeholder="Amount in USD"
          value={amount}
          onChange={(e) => setAmount(e.target.value)}
          required
          min="1"
        />
        <input
          type="number"
          placeholder="Duration (Months)"
          value={duration}
          onChange={(e) => setDuration(e.target.value)}
          required
          min="1"
        />
        <button type="submit">Add Subscription</button>
      </form>
    </section>
  );
};

export default SubscriptionForm;