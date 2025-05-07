export interface Theme {
  name: string;
  flavour: "dark" | "light";
  theme: {
    base: string;
    surface: string;
    overlay: string;
    text: string;
    highlight: string;
    highlight2: string;
    warn: string;
    clickable: string;
    node: string;
    nodeborder: string;
    keyword: string;
    string: string;
    ident: string;
    comment: string;
    type: string;
  };
}

export const setTheme = (theme_int: Theme) => {
  const set = (v: string, color: string) => {
    document.documentElement.style.setProperty(v, color);
  };
  const theme = theme_int.theme;
  set("--base", theme.base);
  set("--surface", theme.surface);
  set("--overlay", theme.overlay);
  set("--text", theme.text);
  set("--highlight", theme.highlight);
  set("--highlight-2", theme.highlight2);
  set("--warn", theme.warn);
  set("--clickable", theme.clickable);
  set("--node", theme.node);
  set("--node-border", theme.nodeborder);
  set("--keyword", theme.keyword);
  set("--string", theme.string);
  set("--ident", theme.ident);
  set("--comment", theme.comment);
  set("--type", theme.type);
};

export const themeToArray = (theme_int: Theme) => {
  const theme = theme_int.theme;
  return [
    theme.base,
    theme.surface,
    theme.overlay,
    theme.text,
    theme.highlight,
    theme.highlight2,
    theme.warn,
    theme.clickable,
    theme.node,
    theme.nodeborder,
    theme.keyword,
    theme.string,
    theme.ident,
    theme.comment,
    theme.type,
  ];
};
