/* Runs before first paint: applies the stored shell theme so the page never
 * flashes the wrong surface, then registers the service worker on load. */
(function () {
  try {
    if (localStorage.getItem("rotaria_theme") === "paper") {
      document.documentElement.setAttribute("data-theme", "paper");
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
