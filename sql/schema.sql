-- ===============================
-- Properties Table
-- ===============================
CREATE TABLE if not exists properties (
    id INTEGER PRIMARY KEY,
    source_id TEXT UNIQUE,
    source TEXT,

    address_line TEXT NOT NULL,
    city TEXT,
    state TEXT,
    state_abbr TEXT,
    postal_code TEXT,
    county_name TEXT,
    county_fips INTEGER,
    country TEXT,

    latitude REAL,
    longitude REAL,

    bedrooms INTEGER,
    bathrooms INTEGER,
    lot_sqft INTEGER,
    property_type TEXT,

    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- ===============================
-- Listings Table (historical pricing)
-- Will derive a "current price" on read.
-- ===============================
CREATE TABLE if not exists listings (
    id INTEGER PRIMARY KEY,

    property_id INTEGER NOT NULL,
    realtor_listing_id TEXT UNIQUE,

    first_seen_at DATETIME NOT NULL,
    last_seen_at DATETIME NOT NULL,

    status TEXT NOT NULL,

    -- Pricing
    list_price INTEGER NOT NULL,
    price_reduced INTEGER DEFAULT 0,
    is_price_reduced BOOLEAN DEFAULT 0,
    sold_price INTEGER,
    currency TEXT DEFAULT 'USD',

    FOREIGN KEY (property_id) REFERENCES properties(id)
);

-- ===============================
-- Listing Observations (per scrape)
-- ===============================
CREATE TABLE IF NOT EXISTS listing_observations (
    id INTEGER PRIMARY KEY,
    listing_id INTEGER NOT NULL,

    observed_at DATETIME NOT NULL,
    page_url TEXT NOT NULL,
    raw_json TEXT,

    FOREIGN KEY (listing_id) REFERENCES listings(id)
);

-- ===============================
-- Scrape Runs
-- ===============================
CREATE TABLE IF NOT EXISTS scrape_runs (
    id INTEGER PRIMARY KEY,

    state TEXT,
    started_at DATETIME NOT NULL,
    finished_at DATETIME,
    pages_fetched INTEGER,
    properties_seen INTEGER,

    success BOOLEAN,
    error_message TEXT
);

-- ===============================
-- Scrape Run Pages
-- ===============================
CREATE TABLE IF NOT EXISTS scrape_run_pages (
    id INTEGER PRIMARY KEY,

    scrape_run_id INTEGER NOT NULL,
    page_number INTEGER NOT NULL,
    page_url TEXT NOT NULL,

    success BOOLEAN,
    properties_found INTEGER,

    FOREIGN KEY (scrape_run_id) REFERENCES scrape_runs(id),
    UNIQUE (scrape_run_id, page_number)
);

-- ===============================
-- Indexes for faster queries
-- ===============================
CREATE INDEX IF NOT EXISTS idx_listings_property_id ON listings(property_id);
CREATE INDEX IF NOT EXISTS idx_listings_last_seen ON listings(last_seen_at);
CREATE INDEX IF NOT EXISTS idx_listings_status ON listings(status);

CREATE INDEX IF NOT EXISTS idx_obs_listing_id ON listing_observations(listing_id);
CREATE INDEX IF NOT EXISTS idx_obs_observed_at ON listing_observations(observed_at);

CREATE INDEX IF NOT EXISTS idx_properties_state ON properties(state);
CREATE INDEX IF NOT EXISTS idx_properties_bedrooms ON properties(bedrooms);
