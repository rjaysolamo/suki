-- PostgreSQL lineage for the mobile product. The legacy MariaDB history is separate.
CREATE TABLE users (
    id UUID PRIMARY KEY,
    google_issuer TEXT NOT NULL,
    google_subject TEXT NOT NULL,
    email TEXT NOT NULL,
    display_name TEXT NOT NULL,
    account_status TEXT NOT NULL DEFAULT 'active' CHECK (account_status IN ('active', 'suspended')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (google_issuer, google_subject)
);

CREATE TABLE role_grants (
    user_id UUID NOT NULL REFERENCES users(id),
    role TEXT NOT NULL CHECK (role IN ('customer', 'vendor', 'courier', 'admin')),
    granted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, role)
);

CREATE TABLE sessions (
    token_hash BYTEA PRIMARY KEY CHECK (octet_length(token_hash) = 32),
    user_id UUID NOT NULL REFERENCES users(id),
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX sessions_user_id ON sessions(user_id);

CREATE TABLE applications (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    kind TEXT NOT NULL CHECK (kind IN ('vendor', 'courier')),
    full_name TEXT NOT NULL,
    contact_number TEXT NOT NULL,
    business_name TEXT,
    stall_details TEXT,
    address TEXT,
    vehicle_type TEXT,
    vehicle_details TEXT,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'approved', 'rejected', 'suspended')),
    decided_by UUID REFERENCES users(id),
    decided_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK ((kind = 'vendor' AND business_name IS NOT NULL AND stall_details IS NOT NULL) OR
           (kind = 'courier' AND address IS NOT NULL AND vehicle_type IS NOT NULL AND vehicle_details IS NOT NULL))
);
CREATE UNIQUE INDEX one_pending_application_per_kind ON applications(user_id, kind) WHERE status = 'pending';

-- Object keys are private identifiers. Documents are not stored in public URLs.
CREATE TABLE application_documents (
    id UUID PRIMARY KEY,
    application_id UUID NOT NULL REFERENCES applications(id),
    kind TEXT NOT NULL CHECK (kind IN ('identification', 'supporting')),
    object_key TEXT NOT NULL UNIQUE,
    uploaded_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE audit_logs (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    actor_id UUID NOT NULL REFERENCES users(id),
    action TEXT NOT NULL,
    entity_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
