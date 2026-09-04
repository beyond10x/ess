import type {ReactNode} from 'react';
import clsx from 'clsx';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';
import CodeBlock from '@theme/CodeBlock';
import Heading from '@theme/Heading';
import HomepageFeatures from '@site/src/components/HomepageFeatures';

import styles from './index.module.css';

const SPECIFICATION = `outcomes:
  - name: accepted
    when: amount.amount > 0
    creates: billing.invoice.Invoice
    emits:
      - billing.invoice.InvoiceCreated

  - name: rejected
    error: billing.invoice.InvalidAmount`;

const GENERATED = `responses:
  '202':
    description: 'Outcome accepted: the branch the
      specification declares for this input.'
    ...
  '422':
    description: 'Outcome rejected: the request was
      understood and refused on domain grounds.'
    ...`;

type PanelProps = {
  ordinal: string;
  label: string;
  title: string;
  alt?: boolean;
  children: ReactNode;
};

function PanelSection({ordinal, label, title, alt, children}: PanelProps) {
  return (
    <section className={clsx(styles.section, alt && styles.sectionAlt)}>
      <div className={styles.sectionInner}>
        <div className={styles.panel}>
          <div className={styles.panelHeader}>
            <div className={styles.panelEyebrow}>
              <span className={styles.panelOrdinal}>{ordinal}</span>
              <span>{label}</span>
            </div>
          </div>
          <Heading as="h2" className={styles.panelTitle}>
            {title}
          </Heading>
          <div className={styles.panelBody}>{children}</div>
        </div>
      </div>
    </section>
  );
}

function HomepageHeader() {
  const {siteConfig} = useDocusaurusContext();
  return (
    <header className={clsx('hero', styles.heroBanner)}>
      <div className={styles.heroInner}>
        <Heading as="h1" className="hero__title">
          {siteConfig.title}
        </Heading>
        <p className="hero__subtitle">{siteConfig.tagline}</p>
        <div className={styles.buttons}>
          <Link className="button button--primary button--lg" to="/docs/getting-started">
            Compile a specification
          </Link>
          <Link className="button button--secondary button--lg" to="/lab">
            Run the browser lab
          </Link>
        </div>
      </div>
    </header>
  );
}

function TheProblem() {
  return (
    <PanelSection ordinal="01" label="The problem" title="System intent drifts when it lives only in prose">
      <p>
        Architecture notes, API descriptions, infrastructure manifests, and test plans each hold a
        different fragment of the system. Nothing proves that they still describe the same thing.
      </p>
      <CodeBlock language="text">
        {`"A paid invoice cannot be cancelled."
        → prose beside code, tests, and an API contract

"This deployment exposes only the public API."
        → manifests whose observed topology may say otherwise`}
      </CodeBlock>
      <p>
        ESS makes those claims typed input. Validation refuses unresolved meaning, compilation
        produces deterministic IR, and every supported projection states its coverage and
        obligations.
      </p>
    </PanelSection>
  );
}

function TheClaim() {
  return (
    <PanelSection ordinal="03" label="The result" title="The contract is generated from the model" alt>
      <p>
        On the left is one outcome from the committed billing specification. On the right is part
        of the OpenAPI contract derived from it. The repository gate fails if generated output
        drifts from its source.
      </p>
      <div className={styles.compare}>
        <div className={styles.compareSide}>
          <CodeBlock language="yaml" title="examples/billing/domains/invoice.yaml">
            {SPECIFICATION}
          </CodeBlock>
        </div>
        <div className={styles.compareArrow} aria-hidden="true">
          <span>→</span>
        </div>
        <div className={styles.compareSide}>
          <CodeBlock language="yaml" title="generated/openapi/invoice-service.yaml">
            {GENERATED}
          </CodeBlock>
        </div>
      </div>
      <p className={styles.panelMore}>
        <Link to="/docs/examples/specification-to-contracts">Follow the complete example →</Link>
      </p>
    </PanelSection>
  );
}

function HonestStatus() {
  return (
    <PanelSection ordinal="04" label="Status" title="Typed where semantics are known; explicit where they are not">
      <div className={styles.ledger}>
        <p className={styles.ledgerBuilt}>
          ESS validates and compiles system specifications; inspects, graphs, diffs, and analyzes
          impact; generates documentation and interface contracts; synthesizes structural code;
          runs conformance suites; imports OpenAPI and sanitized Kubernetes observations; projects
          supported IR; and carries independently released components through verified OCI bundles
          to affected-only Helm reconciliation.
        </p>
        <p className={styles.ledgerNot}>
          Deterministic compilation and projection do not apply infrastructure. Live imports and
          commands named execute, publish, fetch, or reconcile are explicit credential edges.
          Importers do not invent missing semantics; unsupported constructs remain coverage gaps,
          obligations, unresolved references, or refusals—not silently successful conversions.
        </p>
      </div>
      <p className={styles.panelMore}>
        <Link to="/docs/status/where-this-stands">Current capabilities →</Link>
        {' · '}
        <Link to="/docs/status/limitations">Limitations and trust boundary →</Link>
      </p>
    </PanelSection>
  );
}

export default function Home(): ReactNode {
  const {siteConfig} = useDocusaurusContext();
  return (
    <Layout title="Executable system specifications" description={siteConfig.tagline as string}>
      <HomepageHeader />
      <main>
        <TheProblem />
        <HomepageFeatures />
        <TheClaim />
        <HonestStatus />
      </main>
    </Layout>
  );
}
