-- Test Case 1: Multiple table JOIN with potential column name conflicts
-- name: GetBookWithAuthorAndCategories :many
SELECT 
    b.id,
    b.title,
    b.published_year,
    a.id,
    a.name,
    a.birth_year,
    c.id,
    c.name,
    c.description
FROM books b
JOIN authors a ON b.author_id = a.id
JOIN book_categories bc ON b.id = bc.book_id
JOIN categories c ON bc.category_id = c.id
WHERE b.published_year > $1;

-- Test Case 2: Self-join scenario
-- name: GetEmployeesWithManagers :many
SELECT 
    e.id,
    e.name,
    e.department,
    e.salary,
    m.id,
    m.name,
    m.department
FROM employees e
LEFT JOIN employees m ON e.manager_id = m.id;

-- Test Case 3: Subquery with single table (should use simple names)
-- name: GetTopRatedBooks :many
SELECT id, title, published_year
FROM books
WHERE id IN (
    SELECT book_id 
    FROM reviews 
    WHERE rating >= $1
);

-- Test Case 4: Complex aggregation without CTE (simplified for sqlc compatibility)
-- name: GetAuthorBookStats :many
SELECT 
    a.id,
    a.name,
    COUNT(DISTINCT b.id) as book_count,
    AVG(r.rating) as avg_rating,
    b.id,
    b.title
FROM authors a
LEFT JOIN books b ON a.id = b.author_id
LEFT JOIN reviews r ON b.id = r.book_id
WHERE b.id IS NOT NULL
GROUP BY a.id, a.name, b.id, b.title
HAVING COUNT(DISTINCT b.id) > $1;

-- Test Case 5: Table aliases with same column names
-- name: CompareBookYears :many
SELECT 
    old_books.id,
    old_books.title,
    old_books.published_year,
    new_books.id,
    new_books.title,
    new_books.published_year
FROM books old_books
CROSS JOIN books new_books
WHERE old_books.published_year < $1 
  AND new_books.published_year > $2
  AND old_books.id != new_books.id
LIMIT 10;

-- Test Case 6: Column aliases in single table (should still use simple names)
-- name: GetBooksWithAliases :many
SELECT 
    id as book_id,
    title as book_title,
    published_year as year
FROM books
WHERE published_year BETWEEN $1 AND $2;

-- Test Case 7: Complex aggregation with multiple tables
-- name: GetCategoryStats :many
SELECT 
    c.id,
    c.name,
    COUNT(DISTINCT b.id) as book_count,
    COUNT(DISTINCT a.id) as author_count,
    AVG(r.rating) as avg_rating
FROM categories c
LEFT JOIN book_categories bc ON c.id = bc.category_id
LEFT JOIN books b ON bc.book_id = b.id
LEFT JOIN authors a ON b.author_id = a.id
LEFT JOIN reviews r ON b.id = r.book_id
GROUP BY c.id, c.name
HAVING COUNT(DISTINCT b.id) > 0;