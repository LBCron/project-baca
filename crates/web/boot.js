/* Runs before first paint: applies the stored shell theme so the page never
 * flashes the wrong surface, picks the language and the static home shell,
 * then registers the service worker on load.
 * Inside the Tauri app the bundle ships with the app and the WebView keeps
 * no service worker, so none is registered there. */
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
  // The same rule as Lang::load, so the static home shell in index.html
  // shows the language the app is about to draw.
  var lang = null;
  var resumeHint = null;
  try {
    lang = localStorage.getItem("rotaria_lang");
    resumeHint = localStorage.getItem("rotaria_resume_hint");
  } catch (e) {}
  if (lang !== "ID" && lang !== "EN") {
    lang = /^id/i.test(navigator.language || "") ? "ID" : "EN";
  }
  document.documentElement.setAttribute("lang", lang === "ID" ? "id" : "en");
  // A reader with a book to resume is greeted differently once the app
  // knows it (home.rs writes the hint), so the fresh-visitor copy stays off.
  if (location.pathname === "/" && resumeHint !== "1") {
    document.documentElement.setAttribute("data-boot", "home");
    revealWhenFontsLoad();
  }
  // The static heading stays hidden until both of its faces are in: painted
  // in a fallback face, or half italic, it measures smaller than the app's
  // heading, which then becomes the largest paint again. The faces are only
  // known once the stylesheet is parsed, hence DOMContentLoaded. Preloaded on
  // the home page only: elsewhere nothing paints before the WASM, and the
  // fonts would take its bandwidth.
  function revealWhenFontsLoad() {
    ["eb-garamond-normal-latin", "eb-garamond-italic-latin"].forEach(function (name) {
      var link = document.createElement("link");
      link.rel = "preload";
      link.as = "font";
      link.type = "font/woff2";
      link.crossOrigin = "anonymous";
      link.href = "/fonts/" + name + ".woff2";
      document.head.appendChild(link);
    });
    var shown = false;
    var show = function () {
      if (!shown) {
        shown = true;
        document.documentElement.setAttribute("data-boot-fonts", "");
      }
    };
    setTimeout(show, 3000);
    if (!document.fonts || !document.fonts.load) {
      show();
      return;
    }
    document.addEventListener("DOMContentLoaded", function () {
      Promise.all([
        document.fonts.load('500 1em "EB Garamond"', "a"),
        document.fonts.load('italic 500 1em "EB Garamond"', "a")
      ]).then(show, show);
    });
  }
  var nativeShell = "__TAURI_INTERNALS__" in window || "__TAURI__" in window;
  if ("serviceWorker" in navigator && !nativeShell) {
    window.addEventListener("load", function () {
      navigator.serviceWorker.register("/sw.js").catch(function () {});
    });
  }
})();
