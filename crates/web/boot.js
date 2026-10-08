/* Runs before first paint: applies the stored shell theme so the page never
 * flashes the wrong surface, then registers the service worker on load. */
(function () {
  try {
    var stored = localStorage.getItem("rotaria_theme");
    var light = window.matchMedia && window.matchMedia("(prefers-color-scheme: light)").matches;
    if (stored === "paper" || (!stored && light)) {
      document.documentElement.setAttribute("data-theme", "paper");
      var meta = document.querySelector('meta[name="theme-color"]');
      if (meta) meta.setAttribute("content", "#F9F6F0");
    }
  } catch (e) {
    /* storage unavailable (private mode): keep the default surface */
  }
  if ("serviceWorker" in navigator) {
    window.addEventListener("load", function () {
      navigator.serviceWorker.register("/sw.js").catch(function () {});
    });
  }
})();
