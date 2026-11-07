(function (root, factory) {
  if (typeof module === "object" && module.exports) {
    module.exports = factory();
  } else {
    root.ParloRoutes = factory();
  }
})(typeof globalThis === "object" ? globalThis : this, function () {
  const ROUTES = ["home", "meetings", "voice", "library", "insights", "settings"];
  const LIBRARY_TABS = ["history", "models", "vocabulary", "files", "batch", "stories"];
  const SETTINGS_TABS = ["you", "language", "dictation", "languages", "shortcuts", "advanced"];

  function parseHash(hash) {
    const raw = String(hash || "#home").replace(/^#/, "");
    const slash = raw.indexOf("/");
    const name = slash === -1 ? raw : raw.slice(0, slash);
    const tab = slash === -1 ? "" : raw.slice(slash + 1);
    const route = ROUTES.includes(name) ? name : "home";
    const libraryTab =
      route === "library" && LIBRARY_TABS.includes(tab) ? tab : "history";
    const settingsTab =
      route === "settings" && SETTINGS_TABS.includes(tab) ? tab : "dictation";
    const canonical =
      route === "library"
        ? "library/" + libraryTab
        : route === "settings"
          ? "settings/" + settingsTab
          : route;
    return { route, tab, libraryTab, settingsTab, canonical };
  }

  return { ROUTES, LIBRARY_TABS, SETTINGS_TABS, parseHash };
});
