CREATE TABLE loans (
  id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  borrower_id       UUID NOT NULL REFERENCES users(id),
  lender_id         UUID REFERENCES users(id),
  amount_usdc       NUMERIC(18,6) NOT NULL,
  annual_rate       NUMERIC(5,4) NOT NULL DEFAULT 0.05,
  term_months       SMALLINT NOT NULL,
  monthly_payment   NUMERIC(18,6) NOT NULL,
  status            VARCHAR(20) NOT NULL DEFAULT 'PENDING',
  network           VARCHAR(20) NOT NULL DEFAULT 'polygon',
  contract_address  VARCHAR(100),
  deploy_tx_hash    VARCHAR(100),
  fund_tx_hash      VARCHAR(100),
  purpose           VARCHAR(50),
  funded_at         TIMESTAMPTZ,
  due_date          TIMESTAMPTZ,
  created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_loans_borrower_id ON loans(borrower_id);
CREATE INDEX idx_loans_lender_id ON loans(lender_id);
CREATE INDEX idx_loans_status ON loans(status);
CREATE INDEX idx_loans_network ON loans(network);
