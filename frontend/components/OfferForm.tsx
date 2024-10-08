import { useState } from 'react';
import styles from './OfferForm.module.css';

const OfferForm = () => {
  const [offerName, setOfferName] = useState('');
  const [price, setPrice] = useState('');
  const [duration, setDuration] = useState('');

  const handleOffer = (e: React.FormEvent) => {
    e.preventDefault();
    // Mock offer creation
    alert(`Created offer ${offerName} for $${price} over ${duration} months`);
    setOfferName('');
    setPrice('');
    setDuration('');
  };

  return (
    <section className={styles.container}>
      <h3>Create New Offer</h3>
      <form onSubmit={handleOffer} className={styles.form}>
        <input
          type="text"
          placeholder="Offer Name"
          value={offerName}
          onChange={(e) => setOfferName(e.target.value)}
          required
        />
        <input
          type="number"
          placeholder="Price in USD"
          value={price}
          onChange={(e) => setPrice(e.target.value)}
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
        <button type="submit">Create Offer</button>
      </form>
    </section>
  );
};

export default OfferForm;