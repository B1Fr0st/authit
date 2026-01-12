-- Create role enum type
DO $$ BEGIN
    CREATE TYPE role AS ENUM ('User', 'Support', 'Dev', 'Admin');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- Create temporary users table for authentication
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
    email TEXT UNIQUE NOT NULL,
    password TEXT NOT NULL,
    role role NOT NULL DEFAULT 'User',
    banned BOOLEAN NOT NULL DEFAULT FALSE,
    hwid TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create index on email for faster lookups
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);

-- Insert a test user
-- Email: test@example.com
-- Password: test123 (hashed with argon2id)
-- Role: Admin
-- Hash generated with: echo -n "test123" | argon2 somesalt -id -e
INSERT INTO users (id, email, password, role)
VALUES (
    'BiFr0st',
    'test@example.com',
    '$argon2id$v=19$m=16,t=2,p=1$c29tZXNhbHQ$V53/277ijcl8jL2XwkuRnA',
    'Admin'
),
(
    'support_ex_01',
    'tet@example.com',
    '$argon2id$v=19$m=16,t=2,p=1$c29tZXNhbHQ$V53/277ijcl8jL2XwkuRnA',
    'Support'
),
(
    'anon_user',
    'tt@example.com',
    '$argon2id$v=19$m=16,t=2,p=1$c29tZXNhbHQ$V53/277ijcl8jL2XwkuRnA',
    'User'
),
(
    'goateD_V',
    't@example.com',
    '$argon2id$v=19$m=16,t=2,p=1$c29tZXNhbHQ$V53/277ijcl8jL2XwkuRnA',
    'Dev'
)
ON CONFLICT (email) DO NOTHING;
