
-- name: GetBools :many
SELECT *
FROM BoolTable;

-- name: GetNumerics :many
SELECT *
FROM NumericTable;

-- name: GetCharacters :many
SELECT *
FROM CharacterTable;

-- name: GetBinaries :many
SELECT *
FROM BinaryTable;


-- name: GetCustomType :many
SELECT *
FROM SpongeBobVoiceActor;

-- name: CreateVoiceActor :one
INSERT INTO SpongeBobVoiceActor
(voice_actor,character)
VALUES ($1, $2)
RETURNING *;
