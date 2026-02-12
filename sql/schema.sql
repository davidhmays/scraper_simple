-- ===============================
-- TEMPORARY "Option A" Schema (SQLite)
-- Mirror source IDs into internal IDs using TEXT primary keys.
-- ===============================

-- PRAGMA foreign_keys = OFF;

-- DROP TABLE IF EXISTS scrape_run_pages;
-- DROP TABLE IF EXISTS scrape_runs;

-- DROP TABLE IF EXISTS listing_observations;
-- DROP TABLE IF EXISTS listings;
-- DROP TABLE IF EXISTS properties;
-- DROP TABLE IF EXISTS mailings;

-- if you had old indexes with different names, they’ll disappear with the tables,
-- but dropping explicitly is harmless if you prefer:
-- DROP INDEX IF EXISTS idx_properties_address_key;
-- DROP INDEX IF EXISTS idx_properties_source_property_id;
-- DROP INDEX IF EXISTS idx_listings_property_id;
-- DROP INDEX IF EXISTS idx_listings_last_seen;
-- DROP INDEX IF EXISTS idx_listings_status;
-- DROP INDEX IF EXISTS idx_listings_source_lookup;
-- DROP INDEX IF EXISTS idx_listing_observations_listing_id;
-- DROP INDEX IF EXISTS idx_listing_observations_observed_at;

---- Disable above after testing.


PRAGMA foreign_keys = ON;

-- ===============================
-- Properties Table
-- ===============================
CREATE TABLE IF NOT EXISTS properties (
  -- Internal system id (TEMP): mirrors source_property_id (TEXT)
  id TEXT PRIMARY KEY,

  -- Property identifier from the source system (TEMP): same as id for now
  source_property_id TEXT NOT NULL,

  created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Helps if you ever decouple later (kept even if redundant right now)
CREATE UNIQUE INDEX IF NOT EXISTS idx_properties_source_property_id
ON properties(source_property_id);

-- ===============================
-- Listings Table
-- ===============================
CREATE TABLE IF NOT EXISTS listings (
  id TEXT PRIMARY KEY,
  property_id TEXT NOT NULL,

  source TEXT NOT NULL,
  source_id TEXT NOT NULL DEFAULT 'unknown',
  source_listing_id TEXT NOT NULL,

  address_line TEXT NOT NULL,
  city TEXT,
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

  first_seen_at DATETIME NOT NULL,
  last_seen_at DATETIME NOT NULL,
  status TEXT NOT NULL,

  list_price INTEGER NOT NULL,
  price_reduced INTEGER DEFAULT 0,  -- TODO: Duplicate field.
  is_price_reduced INTEGER DEFAULT 0,
  sold_price INTEGER,
  currency TEXT DEFAULT 'USD',

  is_coming_soon INTEGER DEFAULT 0,
  is_contingent INTEGER DEFAULT 0,
  is_foreclosure INTEGER DEFAULT 0,
  is_new_construction INTEGER DEFAULT 0,
  is_new_listing INTEGER DEFAULT 0,
  is_pending INTEGER DEFAULT 0,

  FOREIGN KEY (property_id) REFERENCES properties(id),
  UNIQUE (source, source_listing_id)
);


-- Listings indexes
CREATE INDEX IF NOT EXISTS idx_listings_property_id ON listings(property_id);
CREATE INDEX IF NOT EXISTS idx_listings_last_seen ON listings(last_seen_at);
CREATE INDEX IF NOT EXISTS idx_listings_status ON listings(status);
CREATE INDEX IF NOT EXISTS idx_listings_source_lookup ON listings(source, source_listing_id);

-- ===============================
-- Listing Observations (per scrape)
-- ===============================
CREATE TABLE IF NOT EXISTS listing_observations (
  id INTEGER PRIMARY KEY,
  listing_id TEXT NOT NULL,

  observed_at DATETIME NOT NULL,
  page_url TEXT NOT NULL,
  raw_json TEXT,

  FOREIGN KEY (listing_id) REFERENCES listings(id)
);

CREATE INDEX IF NOT EXISTS idx_listing_observations_listing_id
ON listing_observations(listing_id);

CREATE INDEX IF NOT EXISTS idx_listing_observations_observed_at
ON listing_observations(observed_at);

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

  success INTEGER,
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

  success INTEGER,
  properties_found INTEGER,

  FOREIGN KEY (scrape_run_id) REFERENCES scrape_runs(id),
  UNIQUE (scrape_run_id, page_number)
);


-- ===============================
-- Mailings
-- ===============================

create table if not exists mailings (
  id integer primary key,

  property_id text not null,
  listing_id text not null,

  -- a/b + attribution
  campaign text not null,
  variant text not null,
  description text,

  media_type text not null,
  media_size text not null,
  template_key text,  -- like "canva_postcard_v3" ?

  -- address snapshot (what was printed)
  address_line text not null,
  city text not null,
  state_abbr text not null,
  postal_code text not null,

  -- QR tracking
  qr_token text not null unique,
  scanned_count integer not null default 0,
  first_scanned_at datetime,
  last_scanned_at datetime,

  -- lifecycle
  created_at datetime default current_timestamp,
  printed_at datetime,
  sent_at datetime,

  status text not null default 'created', --created|exported|printed|sent|canceled

  foreign key (property_id) references properties(id),
  foreign key (listing_id)  references listings(id)
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_mailings_property_campaign_variant ON mailings(property_id, campaign, variant);
create index if not exists idx_mailings_property_id on mailings(property_id);
create index if not exists idx_mailings_campaign_variant on mailings(campaign, variant);
create index if not exists idx_mailings_status on mailings(status);

-- mailing events beyond counter.
create table if not exists  mailing_events (
  id integer primary key,
  mailing_id integer not null,
  event_type text not null,
  occurred_at datetime not null,
  user_agent text,
  referrer text,
  foreign key (mailing_id) references mailings(id)
);

create index if not exists idx_mailing_events_mailing_id on mailing_events(mailing_id);
create index if not exists idx_mailing_events_occurred_at on mailing_events(occurred_at);



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
  plan_code  text not null,          -- 'free' or 'lifetime'
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
  format     text not null,     -- 'csv', 'xlsx'
  created_at integer not null,
  foreign key(user_id) references users(id) on delete cascade
);

create index if not exists idx_download_events_user_time
  on download_events(user_id, created_at);

create table if not exists purchases (
  id                  integer primary key,
  user_id              integer not null,
  product_code         text not null,     -- 'lifetime'
  amount_cents         integer not null,
  currency             text not null,
  provider             text,              -- 'stripe'
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
  ('free', 'Free', 0, 4, 0, 'month'),
  ('lifetime', 'Lifetime', 1900, null, 0, 'month');
