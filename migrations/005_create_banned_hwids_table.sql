-- Create banned_hwids table for hardware banning
CREATE TABLE IF NOT EXISTS banned_hwids (
    hwid TEXT PRIMARY KEY,
    reason TEXT,
    banned_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    banned_by TEXT, -- Admin user ID who banned this HWID
    notes TEXT
);

-- Create index for faster lookups
CREATE INDEX IF NOT EXISTS idx_banned_hwids_banned_at ON banned_hwids(banned_at);

-- Insert test data (optional)
-- INSERT INTO banned_hwids (hwid, reason, banned_by, notes)
-- VALUES ('TEST-BANNED-HWID-001', 'Cheating', 'admin-user-id', 'Detected using unauthorized modifications')
-- ON CONFLICT (hwid) DO NOTHING;
