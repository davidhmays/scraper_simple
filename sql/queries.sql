-- Properties changed state recently
SELECT
    p.address_line,
    p.city,
    p.state_abbr,
    l.status,
    l.last_seen_at,
    CASE
        WHEN l.status = 'for_sale' AND l.last_seen_at >= datetime('now','-7 days') THEN 1
        ELSE 0
    END AS is_active
FROM listings l
JOIN properties p ON p.id = l.property_id
WHERE l.last_seen_at >= datetime('now', '-1 day')
ORDER BY l.last_seen_at DESC;

-- listings that disappeared yesterday
SELECT
    p.address_line,
    p.city,
    p.state_abbr,
    l.status,
    l.last_seen_at
FROM listings l
JOIN properties p ON p.id = l.property_id
WHERE l.status = 'off_market'  -- or 'removed' depending on your source
  AND l.last_seen_at >= date('now', '-1 day')
ORDER BY l.last_seen_at DESC;

-- Homes listed for sale per day
SELECT
    date(first_seen_at) AS day,
    COUNT(*) AS listings_added
FROM listings
WHERE status = 'for_sale'
GROUP BY day
ORDER BY day;

-- recent changes for 4-bedroom Homes
SELECT
    p.address_line,
    p.city,
    p.state_abbr,
    l.status,
    l.last_seen_at,
    CASE
        WHEN l.status = 'for_sale' AND l.last_seen_at >= datetime('now','-7 days') THEN 1
        ELSE 0
    END AS is_active
FROM listings l
JOIN properties p ON p.id = l.property_id
WHERE p.bedrooms = 4
  AND l.last_seen_at >= datetime('now', '-7 days')
ORDER BY l.last_seen_at DESC;

-- Rebuild listings from historical observations
SELECT *
FROM listing_observations
ORDER BY observed_at;

-- Compute curent price dynamically
SELECT
    l.*,
    CASE
        WHEN l.status = 'sold' THEN l.sold_price
        ELSE COALESCE(l.list_price,0) - COALESCE(l.price_reduced,0)
    END AS current_price
FROM listings l;
