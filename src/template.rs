use askama::Template;
#[derive(Template)]
#[template(path = "index.2.html")]
pub struct ImageFallTemplate {
    pub imgs: Vec<Imgs>,
    pub column_width:usize,
}
pub struct Imgs {
    path: String,
}

impl Imgs {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}


