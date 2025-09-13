import { ref } from 'vue';

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

// Reactive current theme state
export const currentTheme = ref<Theme | null>(null);

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
  
  // Update reactive state and save to localStorage
  currentTheme.value = theme_int;
  saveTheme(theme_int);
};

export const saveTheme = (theme: Theme) => {
  try {
    localStorage.setItem('org-roamers-theme', JSON.stringify(theme));
  } catch (error) {
    console.warn('Failed to save theme to localStorage:', error);
  }
};

export const loadTheme = (): Theme | null => {
  try {
    const savedTheme = localStorage.getItem('org-roamers-theme');
    if (savedTheme) {
      return JSON.parse(savedTheme) as Theme;
    }
  } catch (error) {
    console.warn('Failed to load theme from localStorage:', error);
  }
  return null;
};

export const initializeTheme = () => {
  const savedTheme = loadTheme();
  if (savedTheme) {
    // Restore saved theme but don't save again to avoid recursive calls
    const set = (v: string, color: string) => {
      document.documentElement.style.setProperty(v, color);
    };
    const theme = savedTheme.theme;
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
    
    // Update reactive state
    currentTheme.value = savedTheme;
    return savedTheme;
  }
  return null;
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
