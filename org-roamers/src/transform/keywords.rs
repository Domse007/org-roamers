use orgize::{
    export::{Container, Event, TraversalContext, Traverser},
    Org,
};

pub struct KeywordCollector {
    pub keywords: Vec<String>,
    to_match: String,
}

impl KeywordCollector {
    pub fn new(to_match: impl ToString) -> KeywordCollector {
        KeywordCollector {
            keywords: vec![],
            to_match: to_match.to_string().to_uppercase(),
        }
    }

    pub fn perform(mut self, org: &str) -> Vec<String> {
        Org::parse(org).traverse(&mut self);
        self.keywords
    }
}

impl Traverser for KeywordCollector {
    fn event(&mut self, event: Event, _ctx: &mut TraversalContext) {
        match event {
            Event::Enter(Container::Keyword(kw)) => {
                if kw.key().to_string().to_uppercase() == self.to_match {
                    self.keywords.push(kw.value().trim().to_string());
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_get_latex_header() {
        const ORG: &str = "
#+title: Test
#+subtitle: test
#+author: Joakim Brodén
#+filetags: :test2:test1:
#+options: date:nil author:t num:nil toc:nil
#+latex_header: \\usepackage{parskip}
* Some other
asfdjsadn

#+latex_header: \\usepackage[margin=3cm]{geometry}
\\( a = b \\)";
        let keywords = KeywordCollector::new("latex_header").perform(ORG);
        assert_eq!(
            keywords,
            vec![
                "\\usepackage{parskip}",
                "\\usepackage[margin=3cm]{geometry}"
            ]
        );
    }
}
