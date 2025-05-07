export interface GeneralSettings {
  showEntireFile: boolean;
}

export let generalSettings: GeneralSettings = {
  showEntireFile: false,
};

export const getScope = () => {
  return generalSettings.showEntireFile ? "file" : "node";
};
