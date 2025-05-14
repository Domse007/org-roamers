use orgize::{
    export::{Event, TraversalContext, Traverser},
    Org,
};
use std::fmt::Write;

pub struct TitleSanitizer {
    output: String,
}

impl TitleSanitizer {
    pub fn new() -> Self {
        Self {
            output: String::new(),
        }
    }

    pub fn process(mut self, title: &str) -> String {
        Org::parse(title).traverse(&mut self);
        self.output
    }
}

impl Traverser for TitleSanitizer {
    fn event(&mut self, event: Event, _ctx: &mut TraversalContext) {
        match event {
            Event::Text(text) => {
                let _ = write!(&mut self.output, "{}", text);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TitleSanitizer;

    #[test]
    fn test_title_sanitizer() {
        let title = "[[id:id][Link]] to =some= *heading*";
        let expected = "Link to some heading";
        let sanitizer = TitleSanitizer::new();
        assert_eq!(sanitizer.process(title), expected);
    }
}
