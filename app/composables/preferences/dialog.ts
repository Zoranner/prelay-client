export function useDesktopPreferencesDialog() {
  const visible = useState("desktop-preferences-dialog-visible", () => false);

  function open() {
    visible.value = true;
  }

  function close() {
    visible.value = false;
  }

  return { visible, open, close };
}
