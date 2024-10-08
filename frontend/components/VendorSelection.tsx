import styles from './VendorSelection.module.css';

const VendorSelection = () => {
  // Mock vendor data
  const vendors = [
    { id: 1, name: 'Helius', description: 'Helius Rpc Service' },
    { id: 2, name: 'Amazon', description: 'Amazon Prime' },
    { id: 3, name: 'Nansen Pro', description: 'Nansen Pro Analytics' },
  ];

  return (
    <section className={styles.container}>
      <h3>Select a Vendor</h3>
      <div className={styles.vendors}>
        {vendors.map((vendor) => (
          <div key={vendor.id} className={styles.vendorCard}>
            <h4>{vendor.name}</h4>
            <p>{vendor.description}</p>
            <button>Select</button>
          </div>
        ))}
      </div>
    </section>
  );
};

export default VendorSelection;