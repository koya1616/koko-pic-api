ALTER TABLE pictures
ADD COLUMN request_id INTEGER REFERENCES requests(id) ON DELETE CASCADE;

CREATE INDEX idx_pictures_request_id ON pictures(request_id);
ALTER TABLE pictures ADD CONSTRAINT pictures_user_request_unique UNIQUE (user_id, request_id);
