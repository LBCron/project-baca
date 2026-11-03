/* Rotaria service worker.
 *
 * - Content-hashed build output (wasm, js, css) and the self-hosted fonts:
 *   cache-first. Their names change whenever their bytes do, so entries the
 *   newest index.html no longer references are pruned after each shell fetch.
 * - Every other static file (boot.js, icons, illustrations, manifest):
 *   network-first, the cache answering only offline, so a deploy reaches a
 *   returning reader on the next visit without bumping a cache version.
 * - Navigations: network-first; the one shell entry ("/") is refreshed and
 *   answers every route offline, since the SPA serves the same document for
 *   all of them.
 * - /api/* : never cached, always network (stale API data and cross-user
 *   leaks are worse than offline errors).
 * - Install precaches the shell, every static file it names, the house fonts,
 *   and the reader-facing illustrations, so one visit is enough to go offline.
 */
const SHELL_CACHE = "rotaria-shell-v3";
const ASSET_CACHE = "rotaria-assets-v3";
const SHELL_URL = "/";
const STATIC_RE = /\.(?:wasm|js|css|png|svg|jpg|jpeg|webp|woff2?|webmanifest)$/;
const IMMUTABLE_RE = /^\/(?:[a-z0-9_-]+-[0-9a-f]{8,}(?:_bg)?\.(?:wasm|js|css)|fonts\/[^/]+\.woff2)$/i;
const HASHED_BUILD_RE = /^\/[a-z0-9_-]+-[0-9a-f]{8,}(?:_bg)?\.(?:wasm|js|css)$/i;
/* Fetched at install, so a first visit that goes offline still has the house
 * type and the illustrations the reader-facing pages draw. The latin subsets
 * cover Indonesian and English; latin-ext and admin artwork are cached when
 * first used. */
const PRECACHE_ASSETS = [
  "/fonts/eb-garamond-normal-latin.woff2",
  "/fonts/eb-garamond-italic-latin.woff2",
  "/fonts/newsreader-normal-latin.woff2",
  "/fonts/newsreader-italic-latin.woff2",
  "/fonts/plus-jakarta-sans-normal-latin.woff2",
  "/assets/library-bookshelf-ladder.webp",
  "/assets/cozy-reader-armchair-owl.webp",
  "/assets/retro-rocket-discovery.webp",
  "/assets/rotaria-mark.svg",
];

/* Paths the shell document references (href/src), quoted or not: release
 * builds minify it and drop the quotes. */
function referencedPaths(html) {
  const paths = new Set();
  for (const match of html.matchAll(/(?:href|src)=["']?([^"'\s>]+)/g)) {
    paths.add(new URL(match[1], self.location.origin).pathname);
  }
  return paths;
}

/* The first visit fetches the app before this worker controls the page, so
 * nothing it loaded is cached yet: take the static files the shell names,
 * the house fonts, and the illustrations now. One by one, so a missing file
 * does not drop the rest as addAll would. */
async function precache() {
  const shell = await caches.open(SHELL_CACHE);
  await shell.add(SHELL_URL);
  const response = await shell.match(SHELL_URL);
  const build = response
    ? [...referencedPaths(await response.text())].filter((p) => STATIC_RE.test(p))
    : [];
  const assets = await caches.open(ASSET_CACHE);
  await Promise.allSettled([...build, ...PRECACHE_ASSETS].map((url) => assets.add(url)));
}

self.addEventListener("install", (event) => {
  event.waitUntil(precache().catch(() => {}));
  self.skipWaiting();
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    (async () => {
      const keys = await caches.keys();
      await Promise.all(
        keys
          .filter((k) => k !== SHELL_CACHE && k !== ASSET_CACHE)
          .map((k) => caches.delete(k))
      );
      await self.clients.claim();
    })()
  );
});

/* Drops hashed build artifacts that the freshly fetched shell no longer
 * references; fonts and unhashed files keep their entries. */
async function pruneStaleAssets(shellResponse) {
  try {
    const live = referencedPaths(await shellResponse.text());
    const cache = await caches.open(ASSET_CACHE);
    for (const request of await cache.keys()) {
      const pathname = new URL(request.url).pathname;
      if (HASHED_BUILD_RE.test(pathname) && !live.has(pathname)) {
        await cache.delete(request);
      }
    }
  } catch (e) {
    /* pruning is best effort */
  }
}

async function cacheFirst(request) {
  const cache = await caches.open(ASSET_CACHE);
  const hit = await cache.match(request);
  if (hit) return hit;
  const fresh = await fetch(request);
  if (fresh && fresh.ok) cache.put(request, fresh.clone());
  return fresh;
}

/* Network answer stored under `key`; offline, the stored copy. */
async function networkFirst(request, cacheName, key) {
  const cache = await caches.open(cacheName);
  try {
    const fresh = await fetch(request);
    if (fresh && fresh.ok) cache.put(key, fresh.clone());
    return fresh;
  } catch (e) {
    const cached = await cache.match(key);
    if (cached) return cached;
    throw e;
  }
}

self.addEventListener("fetch", (event) => {
  const { request } = event;
  if (request.method !== "GET") return;
  const url = new URL(request.url);
  if (url.origin !== self.location.origin || url.pathname.startsWith("/api/")) return;

  if (request.mode === "navigate") {
    event.respondWith(
      (async () => {
        const response = await networkFirst(request, SHELL_CACHE, SHELL_URL);
        if (response.ok) pruneStaleAssets(response.clone());
        return response;
      })()
    );
    return;
  }
  if (IMMUTABLE_RE.test(url.pathname)) {
    event.respondWith(cacheFirst(request));
    return;
  }
  if (STATIC_RE.test(url.pathname)) {
    event.respondWith(networkFirst(request, ASSET_CACHE, request));
  }
});
