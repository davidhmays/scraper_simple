SELECT county_name, COUNT(*) AS n
FROM listings
WHERE state_abbr = ?1
  AND county_name IS NOT NULL
  AND TRIM(county_name) <> ''
GROUP BY county_name
ORDER BY county_name ASC;
