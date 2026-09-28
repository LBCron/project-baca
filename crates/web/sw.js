/* Rotaria service worker: runtime cache only.
 *
 * - Hashed WASM/CSS/JS/fonts/images under any path: cache-first (Trunk
 *   renames files every build, so no static precache list is kept).
 * - /api/* : never cached, always network (stale API data and cross-user
 *   leaks are worse than offline errors).
 * - Navigations + manifest: network-first with cache fallback so a shipped
 *   index.html never traps the user on an old shell.
 */
const SHELL_CACHE = "rotaria-shell-v1";
const ASSET_CACHE = "rotaria-assets-v1";
const ASSET_RE = /\.(wasm|js|css|png|svg|jpg|jpeg|webp|woff2?)$/;

self.addEventListener("install", (event) => {
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
          if (fresh && fresh.ok) cache.put(request, fresh.clone());
          return fresh;
        } catch (e) {
          const cached = await cache.match(request);
          if (cached) return cached;
          throw e;
        }
      })()
    );
  }
});
