
-- name: GetAuthorByID :one
SELECT id, name, country
FROM Author
WHERE id = $1;

-- name: GetAuthorBooks :many
SELECT Author.Name, Book.Title, $1::int as aaaaa
FROM BookAuthor
INNER JOIN Author ON Author.id = BookAuthor.AuthorId
INNER JOIN Book ON Book.Id = BookAuthor.BookId
WHERE Author.Id = $1;

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

-- name: GetSpongeBob :one
SELECT character 
FROM SpongeBobVoiceActor;
