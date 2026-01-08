-- Create cd_keys table for license key management
CREATE TABLE IF NOT EXISTS cd_keys (
    key TEXT PRIMARY KEY,
    time_hours BIGINT NOT NULL,
    product_id TEXT NOT NULL,

    CONSTRAINT check_time_positive CHECK (time_hours > 0)
);

-- Create index on product_id for analytics/bulk operations
CREATE INDEX IF NOT EXISTS idx_cd_keys_product_id ON cd_keys(product_id);

-- Insert test data (720 hours = 30 days)
INSERT INTO cd_keys (key, time_hours, product_id)
VALUES
    ('TEST-1234-ABCD-5678', 720, 'arena-breakout'),
    ('TEST-1234-ABCD-567', 720, 'arena-breakout')
ON CONFLICT (key) DO NOTHING;