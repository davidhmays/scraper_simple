-- counties already keyed by 5-digit geoid (SSCCC)
CREATE TABLE IF NOT EXISTS county (
  geoid TEXT PRIMARY KEY,
  state_fips TEXT NOT NULL,
  county_fips TEXT NOT NULL,
  name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS zcta (
  zcta TEXT PRIMARY KEY
);

CREATE TABLE IF NOT EXISTS zcta_county (
  zcta TEXT NOT NULL,
  county_geoid TEXT NOT NULL,
  -- Optional but useful if you want “mostly in county” ordering
  -- pop_pct REAL,
  -- land_pct REAL,
  PRIMARY KEY (zcta, county_geoid),
  FOREIGN KEY (zcta) REFERENCES zcta(zcta),
  FOREIGN KEY (county_geoid) REFERENCES county(geoid)
);

-- Indexes for dropdown queries
CREATE INDEX IF NOT EXISTS idx_county_state ON county(state_fips, name);
CREATE INDEX IF NOT EXISTS idx_zcta_county_county ON zcta_county(county_geoid, zcta);
CREATE INDEX IF NOT EXISTS idx_zcta_county_zcta ON zcta_county(zcta, county_geoid);
