import styles from './VendorSelection.module.css';

const VendorSelection = () => {
  // Mock vendor data
  const vendors = [
    { id: 1, name: 'Vendor A', description: 'Premium crypto services' },
    { id: 2, name: 'Vendor B', description: 'Standard crypto services' },
    { id: 3, name: 'Vendor C', description: 'Basic crypto services' },
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