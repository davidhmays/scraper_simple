--TODO: indices

CREATE TABLE if not exists a_properties (
  id integer primary key
);

create table if not exists a_listings (
  id integer primary key
);

create table if not exists a_sources (
  -- ID example: "SAUT"
  id integer primary key
);



create table if not exists at_source_description (
  listing_id text not null,
  name text,
  type

  valid_from integer not null,
  valid_to integer
)



-- One property to many listings.
create table if not exists t_property_listings (
  property_id integer not null,
  mls_id integer not null,
  listing_id integer not null,

  valid_from integer not null,
  valid_to integer

  primary key (listing_id),

  foreign key (property_id) references a_properties(id),
  foreign key (listing_id) references a_listings(id)
  foreign key (mls_id)

  check (valid_to is null or valid_to > valid_from)
);

create table if not exists at_address (

);

create table if not exists a_
