-- =================================================================
-- Property-Centric, Change-Tracking Schema
--
-- This schema pivots from a listing-centric model to a property-centric one,
-- focusing on tracking changes to key fields over time, which is the core
-- business requirement.
-- =================================================================

-- Clean slate: remove old tables that are being replaced.
-- In a real migration, you would use an ALTER TABLE script.
-- PRAGMA foreign_keys = OFF;
-- DROP TABLE IF EXISTS listing_observations;
-- DROP TABLE IF EXISTS listings;
-- DROP TABLE IF EXISTS properties;
-- PRAGMA foreign_keys = ON;


-- ===============================
-- Properties Table (New)
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
-- Property History Table (New)
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
-- Property Sources Table (New)
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

-- ===============================
-- Mailings (Adapted for new schema)
-- ===============================
create table if not exists mailings (
  id integer primary key,
  property_id integer not null, -- Changed to INTEGER to match new properties.id
  campaign text not null,
  variant text not null,
  description text,
  media_type text not null,
  media_size text not null,
  template_key text,
  address_line text not null,
  city text not null,
  state_abbr text not null,
  postal_code text not null,
  country text not null,
  qr_token text not null unique,
  scanned_count integer not null default 0,
  first_scanned_at datetime,
  last_scanned_at datetime,
  created_at datetime default current_timestamp,
  printed_at datetime,
  sent_at datetime,
  status text not null default 'created',
  check (state_abbr = upper(state_abbr) and length(state_abbr) = 2),
  foreign key (state_abbr) references states(abbr),
  foreign key (property_id) references properties(id)
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_mailings_property_campaign_variant ON mailings(property_id, campaign, variant);
create index if not exists idx_mailings_property_id on mailings(property_id);
create index if not exists idx_mailings_campaign_variant on mailings(campaign, variant);
create index if not exists idx_mailings_status on mailings(status);

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


-- Seed plans (idempotent)
insert or ignore into plans (code, name, price_cents, download_limit, trial_days, limit_window)
values
  ('free', 'Free', 0, 0, 0, 'month'),
  ('lifetime', 'Lifetime', 1900, null, 0, 'month');
