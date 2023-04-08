-- Add migration script here
CREATE OR REPLACE FUNCTION upsert_visits (p_id SMALLINT, b browser, url_ids integer[], visit_counts integer[], last_visit_time timestamp[]  )
  RETURNS integer AS $$
  DECLARE
  len integer;
    BEGIN
      SELECT array_length(url_ids,1) INTO len;
      INSERT INTO visits
      SELECT * FROM UNNEST(url_ids::integer[], array_fill(p_id::smallint, array[len]), array_fill(b::browser, array[len]), visit_counts::integer[], last_visit_time::timestamp[])
       ON CONFLICT ON CONSTRAINT visits_pkey DO 
       UPDATE SET visit_count = EXCLUDED.visit_count, last_visit_time = EXCLUDED.last_visit_time;
      RETURN len;
    END; $$ LANGUAGE plpgsql;


