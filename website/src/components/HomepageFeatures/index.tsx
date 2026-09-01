import type {ReactNode} from 'react';
import Link from '@docusaurus/Link';
import Heading from '@theme/Heading';
import styles from './styles.module.css';

type FeatureItem = {
  title: string;
  governs: string;
  question: string;
  description: ReactNode;
  href: string;
};

const FeatureList: FeatureItem[] = [
  {
    title: 'Model',
    governs: 'typed system intent',
    question: 'What exists, and how is it related?',
    description: (
      <>
        Systems, domains, entities, commands, outcomes, events, views, components, bindings, and
        topology are validated as concrete typed structures. References resolve to compiler-minted
        handles with total lookups.
      </>
    ),
    href: '/docs/concepts/ess',
  },
  {
    title: 'Project',
    governs: 'deterministic artifacts',
    question: 'What can be derived without guessing?',
    description: (
      <>
        Documentation, schemas, OpenAPI, AsyncAPI, implementation skeletons, and Kubernetes
        manifests come from validated IR. Each adapter declares the subset and direction it
        supports.
      </>
    ),
    href: '/docs/guides/generate-artifacts',
  },
  {
    title: 'Prove',
    governs: 'executable conformance',
    question: 'Does an implementation match the model?',
    description: (
      <>
        The specification generates semantic scenarios. A runner executes them against a target
        and publishes a standalone report whose digest names the exact specification it checked.
      </>
    ),
    href: '/docs/guides/verify-conformance',
  },
];

function Feature({title, governs, question, description, href}: FeatureItem) {
  return (
    <article className={styles.card}>
      <div className={styles.cardHeader}>
        <span className={styles.cardGoverns}>{governs}</span>
        <span className={styles.cardArrow} aria-hidden="true">→</span>
      </div>
      <Heading as="h3" className={styles.cardTitle}>
        <Link to={href} className={styles.cardLink}>{title}</Link>
      </Heading>
      <p className={styles.cardQuestion}>{question}</p>
      <p className={styles.cardBody}>{description}</p>
    </article>
  );
}

export default function HomepageFeatures(): ReactNode {
  return (
    <section className={styles.features}>
      <div className={styles.inner}>
        <div className={styles.header}>
          <div className={styles.eyebrow}>
            <span className={styles.ordinal}>02</span>
            <span>The executable loop</span>
          </div>
        </div>
        <Heading as="h2" className={styles.title}>Model, project, prove</Heading>
        <div className={styles.grid}>
          {FeatureList.map((props) => <Feature key={props.title} {...props} />)}
        </div>
      </div>
    </section>
  );
}
