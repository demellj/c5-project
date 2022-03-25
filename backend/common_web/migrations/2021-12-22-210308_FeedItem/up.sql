-- Your SQL goes here
CREATE TABLE feeditems (
    id SERIAL PRIMARY KEY NOT NULL,
    created_by VARCHAR NOT NULL,
    image_id VARCHAR NOT NULL,
    caption VARCHAR,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

CREATE INDEX feeditems_updated_at on feeditems (updated_at);
