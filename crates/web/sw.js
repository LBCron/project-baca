/* Rotaria service worker.
 *
 * - Hashed WASM/CSS/JS/fonts/images: cache-first. Trunk renames files every
 *   build, so entries that the newest index.html no longer references are
 *   pruned after each successful shell fetch.
 * - /api/* : never cached, always network (stale API data and cross-user
 *   leaks are worse than offline errors).
 * - Navigations + manifest: network-first; offline, the cached page or the
 *   cached shell ("/") answers, so deep links open on the saved app too.
 */
const SHELL_CACHE = "rotaria-shell-v2";
const ASSET_CACHE = "rotaria-assets-v2";
const SHELL_URL = "/";
const ASSET_RE = /\.(wasm|js|css|png|svg|jpg|jpeg|webp|woff2?)$/;
const HASHED_ASSET_RE = /^\/(?:[a-z0-9_-]+-[0-9a-f]{8,}(?:_bg)?\.(?:wasm|js|css))$/i;

self.addEventListener("install", (event) => {
  event.waitUntil(
    caches
      .open(SHELL_CACHE)
      .then((cache) => cache.add(SHELL_URL))
      .catch(() => {})
  );
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
 * references; images and fonts keep their own lifetime. */
async function pruneStaleAssets(shellResponse) {
  try {
    const html = await shellResponse.text();
    const live = new Set();
    for (const match of html.matchAll(/(?:href|src)="([^"]+)"/g)) {
      live.add(new URL(match[1], self.location.origin).pathname);
    }
    const cache = await caches.open(ASSET_CACHE);
    for (const request of await cache.keys()) {
      const pathname = new URL(request.url).pathname;
      if (HASHED_ASSET_RE.test(pathname) && !live.has(pathname)) {
        await cache.delete(request);
      }
    }
  } catch (e) {
    /* pruning is best effort */
  }
}

self.addEventListener("fetch", (event) => {
  const { request } = event;
  if (request.method !== "GET") return;
  const url = new URL(request.url);

  if (url.pathname.startsWith("/api/")) return;

  if (ASSET_RE.test(url.pathname)) {
    event.respondWith(
      (async () => {
        const cache = await caches.open(ASSET_CACHE);
        const hit = await cache.match(request);
        if (hit) return hit;
        const fresh = await fetch(request);
        if (fresh && fresh.ok) cache.put(request, fresh.clone());
        return fresh;
      })()
    );
    return;
  }

  if (request.mode === "navigate" || url.pathname.endsWith("manifest.webmanifest")) {
    event.respondWith(
      (async () => {
        const cache = await caches.open(SHELL_CACHE);
        try {
          const fresh = await fetch(request);
          if (fresh && fresh.ok) {
            cache.put(request, fresh.clone());
            if (request.mode === "navigate") pruneStaleAssets(fresh.clone());
          }
          return fresh;
        } catch (e) {
          const cached = await cache.match(request);
          if (cached) return cached;
          if (request.mode === "navigate") {
            const shell = await cache.match(SHELL_URL);
            if (shell) return shell;
          }
          throw e;
        }
      })()
    );
  }
});
