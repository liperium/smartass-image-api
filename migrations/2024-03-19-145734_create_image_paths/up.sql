-- Your SQL goes here
CREATE TYPE image_function AS ENUM ('help', 'proof');

CREATE TABLE image_path (
    id SERIAL PRIMARY KEY,
    filename VARCHAR NOT NULL,
    task_id VARCHAR NOT NULL,
    user_id VARCHAR NOT NULL,
    function image_function NOT NULL
)
