CREATE TABLE IF NOT EXISTS recipes (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    cuisine TEXT NOT NULL,
    cooking_time_minutes INTEGER NOT NULL,
    prep_time_minutes INTEGER NOT NULL,
    servings INTEGER NOT NULL,
    calories_per_serving INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS ingredients (
    recipe_id INTEGER NOT NULL,
    ingredient TEXT NOT NULL,
    FOREIGN KEY (recipe_id) REFERENCES recipes(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS dietary_restrictions (
    recipe_id INTEGER NOT NULL,
    dietary_restriction TEXT NOT NULL,
    FOREIGN KEY (recipe_id) REFERENCES recipes(id) ON DELETE CASCADE
);
