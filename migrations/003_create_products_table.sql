-- Create products table
CREATE TABLE IF NOT EXISTS products (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT check_id_format CHECK (id ~ '^[a-z0-9-]+$')  -- Enforce kebab-case
);

-- Insert initial products
INSERT INTO products (id, name)
VALUES
    ('arena-breakout', 'Arena Breakout: Infinite')
ON CONFLICT (id) DO NOTHING;

-- Create user_licenses table for tracking user product ownership
CREATE TABLE IF NOT EXISTS user_licenses (
    user_id TEXT NOT NULL,
    product_id TEXT NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (user_id, product_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE RESTRICT,

    CONSTRAINT check_expires_future CHECK (expires_at > created_at)
);

-- Create indexes for common queries
CREATE INDEX IF NOT EXISTS idx_user_licenses_user_id ON user_licenses(user_id);
CREATE INDEX IF NOT EXISTS idx_user_licenses_expires_at ON user_licenses(expires_at);

-- Add foreign key to cd_keys table
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.table_constraints
        WHERE constraint_name = 'fk_cd_keys_product'
    ) THEN
        ALTER TABLE cd_keys
        ADD CONSTRAINT fk_cd_keys_product
        FOREIGN KEY (product_id) REFERENCES products(id)
        ON DELETE RESTRICT;
    END IF;
END $$;
