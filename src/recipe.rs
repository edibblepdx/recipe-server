pub struct Recipe {
    id: u64,
    name: String,
}

impl Default for Recipe {
    fn default() -> Self {
        Self {
            id: 0,
            name: "default".to_string(),
        }
    }
}
