(function (root, factory) {
  if (typeof module === "object" && module.exports) {
    module.exports = factory();
  } else {
    root.ParloRoutes = factory();
  }
})(typeof globalThis === "object" ? globalThis : this, function () {
  const ROUTES = ["home", "meetings", "library", "voice", "insights", "settings"];
  const LIBRARY_TABS = ["files", "batch", "stories", "history"];
  const SETTINGS_TABS = [
    "you",
    "languages",
    "dictation",
    "vocabulary",
    "models",
    "shortcuts",
    "advanced",
  ];
  const ALIASES = { "library/meetings": "meetings" };

  function parseHash(hash) {
    let raw = String(hash || "#home").replace(/^#/, "");
    raw = ALIASES[raw] || raw;
    const slash = raw.indexOf("/");
    const name = slash === -1 ? raw : raw.slice(0, slash);
    const tab = slash === -1 ? "" : raw.slice(slash + 1);
    const route = ROUTES.includes(name) ? name : "home";
    const libraryTab =
      route === "library" && LIBRARY_TABS.includes(tab) ? tab : "files";
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

  return { ROUTES, LIBRARY_TABS, SETTINGS_TABS, ALIASES, parseHash };
});
