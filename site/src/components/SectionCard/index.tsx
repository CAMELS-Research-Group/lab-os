import Link from '@docusaurus/Link';
import styles from './styles.module.css';

interface SectionCardProps {
  title: string;
  description: string;
  to: string;
}

export default function SectionCard({ title, description, to }: SectionCardProps): JSX.Element {
  return (
    <Link to={to} className={styles.card}>
      <div className={styles.body}>
        <h3 className={styles.title}>{title}</h3>
        <p className={styles.description}>{description}</p>
      </div>
      <span className={styles.arrow} aria-hidden="true">→</span>
    </Link>
  );
}
