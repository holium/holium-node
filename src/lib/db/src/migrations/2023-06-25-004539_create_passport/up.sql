CREATE TABLE passports (
    ship TEXT PRIMARY KEY,
    is_public BOOLEAN NOT NULL,
    nickname TEXT NOT NULL,
    color TEXT NOT NULL,
    twitter TEXT,
    bio TEXT,
    avatar TEXT,
    cover TEXT,
    featured_url TEXT,
    phone_number TEXT,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
)