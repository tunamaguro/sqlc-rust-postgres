
-- name: GetAuthorByID :one
SELECT id, name, country
FROM Author
WHERE id = $1;

-- name: GetBookByID :one
SELECT id, title, translations
FROM Book
WHERE id = $1;

-- name: ListAuthors :many
SELECT id, name, country
FROM Author
ORDER BY name;

-- name: ListBooks :many
SELECT id, title, translations
FROM Book
ORDER BY id;

-- name: CreateAuthor :exec
INSERT INTO Author (name, country)
VALUES ($1, $2);

-- name: CreateBook :exec
INSERT INTO Book (title)
VALUES ($1);

-- name: AssignAuthorToBook :exec
INSERT INTO BookAuthor (AuthorId, BookId)
VALUES ($1, $2);
