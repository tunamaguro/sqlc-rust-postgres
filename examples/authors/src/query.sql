-- name: GetAuthor :one
SELECT * FROM authors
WHERE id = $1 LIMIT 1;

-- name: ListAuthors :many
SELECT * FROM authors
ORDER BY name;

-- name: CreateAuthor :one
INSERT INTO authors (
          name, bio
) VALUES (
  $1, $2
)
RETURNING *;

-- name: DeleteAuthor :exec
DELETE FROM authors
WHERE id = $1;

-- name: GetAuthorByIdAndAge :one
-- Test query for nullable Copy type (age) with non-nullable Copy type (id)
SELECT * FROM authors
WHERE id = $1 AND (age = $2 OR $2 IS NULL)
LIMIT 1;

-- name: UpdateAuthorStatus :exec  
-- Test query with nullable Copy type (age) and non-nullable Copy type (is_active)
UPDATE authors
SET is_active = $1, age = $2
WHERE id = $3;