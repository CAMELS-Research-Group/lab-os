import type { ReactNode } from 'react';
import styles from './styles.module.css';

interface OutcomeCardProps {
  title: string;
  description: string;
  children?: ReactNode;
}

export default function OutcomeCard({ title, description, children }: OutcomeCardProps): JSX.Element {
  return (
    <div className={styles.card}>
      <h3 className={styles.title}>{title}</h3>
      <p className={styles.description}>{description}</p>
      {children}
    </div>
  );
}
