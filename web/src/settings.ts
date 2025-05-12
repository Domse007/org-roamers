export const STATUS_INTERVAL: number = 10 * 1000;

export interface GeneralSettings {
  showEntireFile: boolean;
  stopLayoutAfter: number | null;
}

export let generalSettings: GeneralSettings = {
  showEntireFile: false,
  stopLayoutAfter: 15,
};

export const getScope = () => {
  return generalSettings.showEntireFile ? "file" : "node";
};
