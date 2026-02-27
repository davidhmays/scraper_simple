-- =================================================================
-- Direct Mail Attribution Platform Schema
-- =================================================================

-- Clean slate: remove old tables that are being replaced.
-- In a real migration, you would use an ALTER TABLE script.
PRAGMA foreign_keys = OFF;
DROP TABLE IF EXISTS listing_observations;
DROP TABLE IF EXISTS listings;
DROP TABLE IF EXISTS properties;
PRAGMA foreign_keys = ON;


-- ===============================
-- Properties Table (Source Data)
-- ===============================
-- This is the central table, representing a unique physical property.
-- It holds the CURRENT state of the fields we track.
-- A property is uniquely identified by its address components.
CREATE TABLE IF NOT EXISTS properties (
  id INTEGER PRIMARY KEY,

  -- Address fields (used for unique identification)
  address_line TEXT NOT NULL,
  city TEXT NOT NULL,
  postal_code TEXT NOT NULL,
  state_abbr TEXT, -- Can be null if not provided, but should be normalized
  county_name TEXT,

  -- Tracked fields (current state)
  status TEXT,
  list_price INTEGER,
  sold_price INTEGER,
  sold_date DATETIME,
  is_pending INTEGER,
  is_contingent INTEGER,
  is_new_listing INTEGER,
  is_foreclosure INTEGER,
  is_price_reduced INTEGER,
  is_coming_soon INTEGER,

  -- Lifecycle
  first_seen_at DATETIME NOT NULL,
  last_seen_at DATETIME NOT NULL,

  FOREIGN KEY (state_abbr) REFERENCES states(abbr),
  UNIQUE (address_line, city, postal_code)
);

CREATE INDEX IF NOT EXISTS idx_properties_status ON properties(status);
CREATE INDEX IF NOT EXISTS idx_properties_last_seen ON properties(last_seen_at);


-- ===============================
-- Property History Table (Source Data)
-- ===============================
-- This is the audit log. Every time a tracked field on a property changes,
-- a new row is inserted here. This table is the source of truth for deltas.
CREATE TABLE IF NOT EXISTS property_history (
  id INTEGER PRIMARY KEY,
  property_id INTEGER NOT NULL,

  observed_at DATETIME NOT NULL,
  field_name TEXT NOT NULL,      -- e.g., 'status', 'list_price'
  previous_value TEXT,           -- The old value (can be NULL for the first time)
  current_value TEXT NOT NULL,   -- The new value

  FOREIGN KEY (property_id) REFERENCES properties(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_property_history_property_id ON property_history(property_id);
CREATE INDEX IF NOT EXISTS idx_property_history_field_name ON property_history(field_name);
CREATE INDEX IF NOT EXISTS idx_property_history_observed_at ON property_history(observed_at);


-- ===============================
-- Property Sources Table (Source Data)
-- ===============================
-- This table links our internal property record back to one or more source
-- listings (e.g., from Realtor.com). This handles the M:1 listing-to-property
-- relationship and deduplication.
CREATE TABLE IF NOT EXISTS property_sources (
  id INTEGER PRIMARY KEY,
  property_id INTEGER NOT NULL,
  source_name TEXT NOT NULL,       -- e.g., 'realtor', 'zillow'
  source_listing_id TEXT NOT NULL, -- The ID from the original source

  first_seen_at DATETIME NOT NULL,
  last_seen_at DATETIME NOT NULL,

  FOREIGN KEY (property_id) REFERENCES properties(id) ON DELETE CASCADE,
  UNIQUE(source_name, source_listing_id)
);

CREATE INDEX IF NOT EXISTS idx_property_sources_property_id ON property_sources(property_id);


-- =================================================================
-- Direct Mail Platform Schema (Campaigns, Media, Lists, Mailings)
-- =================================================================

-- Campaigns (Strategic)
CREATE TABLE IF NOT EXISTS campaigns (
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'draft', -- draft, active, archived
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Media (Creative)
CREATE TABLE IF NOT EXISTS media (
    id INTEGER PRIMARY KEY,
    campaign_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    media_type TEXT NOT NULL, -- e.g., 'postcard_4x6', 'letter_8.5x11'
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (campaign_id) REFERENCES campaigns(id)
);

-- Lists (Data)
CREATE TABLE IF NOT EXISTS lists (
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    source_type TEXT NOT NULL, -- 'system_snapshot', 'upload', 'marketplace'
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- List Rows (Recipients)
-- This links back to our properties if it's a system list,
-- or holds raw data if it's an upload.
CREATE TABLE IF NOT EXISTS list_rows (
    id INTEGER PRIMARY KEY,
    list_id INTEGER NOT NULL,
    property_id INTEGER, -- Optional link to our scraped properties

    -- Snapshot data (in case property changes later, we know what we mailed)
    address_line TEXT NOT NULL,
    city TEXT NOT NULL,
    state_abbr TEXT NOT NULL,
    postal_code TEXT NOT NULL,
    name TEXT, -- "Current Resident" or specific name

    FOREIGN KEY (list_id) REFERENCES lists(id),
    FOREIGN KEY (property_id) REFERENCES properties(id)
);

-- Mailings (Operational)
CREATE TABLE IF NOT EXISTS mailings (
    id INTEGER PRIMARY KEY,
    campaign_id INTEGER NOT NULL,
    list_id INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'draft', -- draft, pending_print, sent
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    scheduled_at DATETIME,

    FOREIGN KEY (campaign_id) REFERENCES campaigns(id),
    FOREIGN KEY (list_id) REFERENCES lists(id)
);

-- Recipient Instances (Atomic Tracking)
CREATE TABLE IF NOT EXISTS recipient_instances (
    id INTEGER PRIMARY KEY,
    mailing_id INTEGER NOT NULL,
    list_row_id INTEGER NOT NULL,
    media_id INTEGER NOT NULL, -- Which creative variant did they get?

    qr_token TEXT NOT NULL UNIQUE, -- The magic string in the QR code

    FOREIGN KEY (mailing_id) REFERENCES mailings(id),
    FOREIGN KEY (list_row_id) REFERENCES list_rows(id),
    FOREIGN KEY (media_id) REFERENCES media(id)
);

-- Click Events (Analytics)
CREATE TABLE IF NOT EXISTS click_events (
    id INTEGER PRIMARY KEY,
    recipient_instance_id INTEGER NOT NULL,
    scanned_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    ip_address TEXT,
    user_agent TEXT,

    FOREIGN KEY (recipient_instance_id) REFERENCES recipient_instances(id)
);


-- =================================================================
-- Supporting Tables (Largely Unchanged)
-- These tables support application features like user auth, operational
-- logging, and mailings. They are kept from the previous schema.
-- =================================================================

-- ===============================
-- States Table
-- ===============================
create table if not exists states (
  abbr text primary key,
  name text not null
);
insert or ignore into states (abbr, name) values
('AL','Alabama'),('AK','Alaska'),('AZ','Arizona'),('AR','Arkansas'),('CA','California'),('CO','Colorado'),('CT','Connecticut'),('DE','Delaware'),('FL','Florida'),('GA','Georgia'),('HI','Hawaii'),('ID','Idaho'),('IL','Illinois'),('IN','Indiana'),('IA','Iowa'),('KS','Kansas'),('KY','Kentucky'),('LA','Louisiana'),('ME','Maine'),('MD','Maryland'),('MA','Massachusetts'),('MI','Michigan'),('MN','Minnesota'),('MS','Mississippi'),('MO','Missouri'),('MT','Montana'),('NE','Nebraska'),('NV','Nevada'),('NH','New Hampshire'),('NJ','New Jersey'),('NM','New Mexico'),('NY','New York'),('NC','North Carolina'),('ND','North Dakota'),('OH','Ohio'),('OK','Oklahoma'),('OR','Oregon'),('PA','Pennsylvania'),('RI','Rhode Island'),('SC','South Carolina'),('SD','South Dakota'),('TN','Tennessee'),('TX','Texas'),('UT','Utah'),('VT','Vermont'),('VA','Virginia'),('WA','Washington'),('WV','West Virginia'),('WI','Wisconsin'),('WY','Wyoming');

-- ===============================
-- Scrape Runs & Pages
-- ===============================
CREATE TABLE IF NOT EXISTS scrape_runs (
  id INTEGER PRIMARY KEY,
  state TEXT,
  started_at DATETIME NOT NULL,
  finished_at DATETIME,
  pages_fetched INTEGER,
  properties_seen INTEGER,
  success INTEGER,
  error_message TEXT
);
CREATE TABLE IF NOT EXISTS scrape_run_pages (
  id INTEGER PRIMARY KEY,
  scrape_run_id INTEGER NOT NULL,
  page_number INTEGER NOT NULL,
  page_url TEXT NOT NULL,
  success INTEGER,
  properties_found INTEGER,
  FOREIGN KEY (scrape_run_id) REFERENCES scrape_runs(id),
  UNIQUE (scrape_run_id, page_number)
);

-- ===============================
-- Users, Auth, and Billing
-- ===============================
create table if not exists users (
  id            integer primary key,
  email         text not null unique,
  created_at    integer not null,
  last_login_at integer,
  is_admin      integer not null default 0
);
create table if not exists magic_links (
  id          integer primary key,
  user_id     integer not null,
  token_hash  blob not null,
  created_at  integer not null,
  expires_at  integer not null,
  used_at     integer,
  foreign key(user_id) references users(id) on delete cascade
);
create index if not exists idx_magic_links_hash on magic_links(token_hash);
create index if not exists idx_magic_links_user on magic_links(user_id);
create table if not exists plans (
  id             integer primary key,
  code           text not null unique,
  name           text not null,
  price_cents    integer not null default 0,
  download_limit integer,
  trial_days     integer not null default 0,
  limit_window   text not null default 'month'
);
create table if not exists entitlements (
  id         integer primary key,
  user_id    integer not null unique,
  plan_code  text not null,
  granted_at integer not null,
  foreign key(user_id) references users(id) on delete cascade,
  foreign key(plan_code) references plans(code)
);
create index if not exists idx_entitlements_user on entitlements(user_id);
create index if not exists idx_entitlements_plan on entitlements(plan_code);
create table if not exists download_events (
  id         integer primary key,
  user_id    integer not null,
  state      text not null,
  format     text not null,
  created_at integer not null,
  foreign key(user_id) references users(id) on delete cascade
);
create index if not exists idx_download_events_user_time
  on download_events(user_id, created_at);
create table if not exists purchases (
  id                  integer primary key,
  user_id              integer not null,
  product_code         text not null,
  amount_cents         integer not null,
  currency             text not null,
  provider             text,
  provider_payment_id  text unique,
  created_at           integer not null,
  foreign key(user_id) references users(id) on delete cascade
);
create index if not exists idx_purchases_user on purchases(user_id);
create index if not exists idx_purchases_provider_payment on purchases(provider_payment_id);
create table if not exists sessions (
  id          integer primary key,
  user_id     integer not null,
  token_hash  blob not null unique,
  created_at  integer not null,
  expires_at  integer not null,
  revoked_at  integer,
  foreign key(user_id) references users(id) on delete cascade
);
create index if not exists idx_sessions_user on sessions(user_id);
create index if not exists idx_sessions_expires on sessions(expires_at);


-- Seed plans (idempotent)
insert or ignore into plans (code, name, price_cents, download_limit, trial_days, limit_window)
values
  ('free', 'Free', 0, 0, 0, 'month'),
  ('lifetime', 'Lifetime', 1900, null, 0, 'month');
