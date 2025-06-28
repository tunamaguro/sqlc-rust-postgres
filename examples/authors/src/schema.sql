CREATE TABLE authors (
          id   BIGSERIAL PRIMARY KEY,
          name text      NOT NULL,
          bio  text,
          age  integer,         -- nullable Copy type
          is_active boolean     -- non-nullable Copy type for mixed testing
);