use crate::recipe::Recipe;

use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    recipe: &'a Recipe,
    stylesheet: &'static str,
}

impl<'a> IndexTemplate<'a> {
    pub fn recipe(recipe: &'a Recipe) -> Self {
        Self {
            recipe,
            stylesheet: "/recipe.css",
        }
    }
}
