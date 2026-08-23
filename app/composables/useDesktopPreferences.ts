export type DesktopThemeMode = "system" | "light" | "dark";

export interface DesktopPreferences {
  theme: DesktopThemeMode;
  autostartEnabled: boolean;
  silentStart: boolean;
  minimizeToTray: boolean;
}

const defaults: DesktopPreferences = {
  theme: "system",
  autostartEnabled: false,
  silentStart: true,
  minimizeToTray: true,
};

export function useDesktopPreferences() {
  const { invokeCommand } = useRelayCommand();
  const preferences = useState<DesktopPreferences>(
    "desktop-preferences",
    () => ({ ...defaults }),
  );
  const loaded = useState("desktop-preferences-loaded", () => false);

  function applyTheme(theme: DesktopThemeMode) {
    if (!import.meta.client) return;
    const useLightTheme =
      theme === "light" ||
      (theme === "system" &&
        window.matchMedia("(prefers-color-scheme: light)").matches);
    document.documentElement.classList.toggle("light", useLightTheme);
    window.dispatchEvent(new Event("prelay:theme-changed"));
  }

  async function load() {
    if (!import.meta.client || !("__TAURI_INTERNALS__" in globalThis)) {
      preferences.value = { ...defaults };
    } else {
      preferences.value = await invokeCommand<DesktopPreferences>(
        "desktop_preferences_get",
      );
    }
    applyTheme(preferences.value.theme);
    loaded.value = true;
    return preferences.value;
  }

  async function save(next: DesktopPreferences) {
    const saved = await invokeCommand<DesktopPreferences>(
      "desktop_preferences_save",
      { preferences: next },
    );
    preferences.value = saved;
    applyTheme(saved.theme);
    return saved;
  }

  return { preferences, loaded, applyTheme, load, save };
}
