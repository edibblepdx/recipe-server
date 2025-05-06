use crate::recipe::Recipe;

use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    recipe: &'a Recipe,
    stylesheet: &'static str,
}

impl<'a> IndexTemplate<'a> {
    pub fn new(recipe: &'a Recipe) -> Self {
        Self {
            recipe,
            stylesheet: "/recipe.css",
        }
    }
}
