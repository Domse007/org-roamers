use std::{
    env, fs,
    hash::{DefaultHasher, Hash, Hasher},
    path::PathBuf,
};

const PREAMBLE: &str = concat!(
    "\\documentclass{article}\n",
    "\\usepackage[T1]{fontenc}\n",
    "\\usepackage[active,tightpage]{preview}\n",
    "\\usepackage{amsmath}\n",
    "\\usepackage{amssymb}\n",
);

pub struct LatexBuilder<'a> {
    headers: Vec<&'a str>,
    body: Vec<&'a str>,
}

impl<'a> LatexBuilder<'a> {
    pub fn new() -> Self {
        Self {
            headers: vec![],
            body: vec![],
        }
    }

    pub fn headers<T: AsRef<str>>(&mut self, headers: &'a [T]) {
        self.headers = headers.iter().map(|e| e.as_ref()).collect()
    }

    pub fn body(&mut self, body: &[&'a str]) {
        self.body = body.into();
    }

    pub fn build(self, color: &str) -> String {
        let mut s = String::new();
        s.push_str(PREAMBLE);
        s.push_str(&self.headers.join("\n"));
        s.push_str("\\usepackage{xcolor}\n");
        s.push_str(&format!("\\definecolor{{mycolor}}{{HTML}}{{{color}}}\n"));
        s.push_str("\\begin{document}\n");
        s.push_str("\\begin{preview}\n");
        s.push_str("\\color{mycolor}\n");
        s.push_str(&self.body.join("\n"));
        s.push_str("\n\\end{preview}\n");
        s.push_str("\\end{document}\n");
        s
    }
}

pub struct LatexPathBuilder {
    path: PathBuf,
}

impl LatexPathBuilder {
    pub fn new() -> Self {
        let mut dir = env::temp_dir();
        dir.push("org-roamers/");
        if !dir.exists() {
            let _ = fs::create_dir_all(&dir);
        }
        Self { path: dir }
    }

    pub fn build(&mut self, filename: &str) -> (PathBuf, PathBuf, PathBuf) {
        let mut hasher = DefaultHasher::default();
        filename.hash(&mut hasher);
        let hash = hasher.finish();
        let mut path_tex = self.path.clone();
        let mut path_dvi = self.path.clone();
        let mut path_svg = self.path.clone();
        path_tex.push(format!("{hash}.tex"));
        path_dvi.push(format!("{hash}.dvi"));
        path_svg.push(format!("{hash}.svg"));
        (path_tex, path_dvi, path_svg)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::latex::builder::{LatexBuilder, LatexPathBuilder};

    #[test]
    fn test_latex_builder() {
        let mut builder = LatexBuilder::new();
        builder.headers(&["\\usepackage{tikz}", "\\usepackage{algorithmic}"]);
        builder.body(&["$f(x)=x^2$"]);
        let exp = concat!(
            "\\documentclass{article}\n",
            "\\usepackage[T1]{fontenc}\n",
            "\\usepackage[active,tightpage]{preview}\n",
            "\\usepackage{amsmath}\n",
            "\\usepackage{amssymb}\n",
            "\\usepackage{tikz}\n",
            "\\usepackage{algorithmic}",
            "\\usepackage{xcolor}\n",
            "\\definecolor{mycolor}{HTML}{green}\n",
            "\\begin{document}\n",
            "\\begin{preview}\n",
            "\\color{mycolor}\n",
            "$f(x)=x^2$\n",
            "\\end{preview}\n",
            "\\end{document}\n"
        );
        assert_eq!(builder.build("green").as_str(), exp);
    }

    #[test]
    fn test_latex_path_builder() {
        let mut builder = LatexPathBuilder::new();
        assert_eq!(
            builder.build("test"),
            (
                PathBuf::from("/tmp/org-roamers/14402189752926126668.tex"),
                PathBuf::from("/tmp/org-roamers/14402189752926126668.dvi"),
                PathBuf::from("/tmp/org-roamers/14402189752926126668.svg")
            )
        );
    }
}
