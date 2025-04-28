CREATE TABLE IF NOT EXISTS recipes (
    id INTEGER PRIMARY KEY,
    recipe_name TEXT NOT NULL,
    cuisine_id TEXT NOT NULL,
    ingredients TEXT NOT NULL,
    cooking_time_minutes INTEGER,
    prep_time_minutes INTEGER,
    servings INTEGER,
    calories_per_serving INTEGER,
    dietary_restrictions TEXT,
    FOREIGN KEY (cuisine_id) REFERENCES cuisines(id)
);


CREATE TABLE IF NOT EXISTS cuisines (
    id TEXT PRIMARY KEY
);
