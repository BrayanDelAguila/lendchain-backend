CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE users (
  id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  email                 VARCHAR(255) NOT NULL UNIQUE,
  password_hash         VARCHAR(255) NOT NULL,
  full_name             VARCHAR(255) NOT NULL,
  document_number       VARCHAR(50),
  phone                 VARCHAR(30),
  wallet_address        VARCHAR(100) NOT NULL UNIQUE,
  encrypted_private_key TEXT NOT NULL,
  kyc_status            VARCHAR(20) NOT NULL DEFAULT 'PENDING',
  role                  VARCHAR(20) NOT NULL DEFAULT 'USER',
  is_active             BOOLEAN NOT NULL DEFAULT TRUE,
  created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at            TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_wallet_address ON users(wallet_address);
