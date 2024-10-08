import styles from './SubscriptionList.module.css';

const SubscriptionList = () => {
  // Mock subscription data
  const subscriptions = [
    { id: 1, user: 'Alice', offer: 'Premium Plan', status: 'Active' },
    { id: 2, user: 'Bob', offer: 'Standard Plan', status: 'Pending' },
    { id: 3, user: 'Charlie', offer: 'Basic Plan', status: 'Cancelled' },
  ];

  return (
    <section className={styles.container}>
      <h3>Subscriptions</h3>
      <table className={styles.table}>
        <thead>
          <tr>
            <th>User</th>
            <th>Offer</th>
            <th>Status</th>
          </tr>
        </thead>
        <tbody>
          {subscriptions.map((sub) => (
            <tr key={sub.id}>
              <td>{sub.user}</td>
              <td>{sub.offer}</td>
              <td className={styles[sub.status.toLowerCase()]}>{sub.status}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </section>
  );
};

export default SubscriptionList;