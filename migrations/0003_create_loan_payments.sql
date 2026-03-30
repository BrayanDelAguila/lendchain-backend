CREATE TABLE loan_payments (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  loan_id         UUID NOT NULL REFERENCES loans(id),
  payment_number  SMALLINT NOT NULL,
  amount_usdc     NUMERIC(18,6) NOT NULL,
  principal       NUMERIC(18,6) NOT NULL,
  interest        NUMERIC(18,6) NOT NULL,
  tx_hash         VARCHAR(100),
  status          VARCHAR(20) NOT NULL DEFAULT 'PENDING',
  paid_at         TIMESTAMPTZ,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_payments_loan_id ON loan_payments(loan_id);
CREATE INDEX idx_payments_status ON loan_payments(status);
