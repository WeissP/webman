-- Add migration script here
CREATE OR REPLACE VIEW url_visit AS
(
SELECT * 
FROM urls INNER JOIN visits on urls.id = visits.url_id
);
