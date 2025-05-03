// sw.js

// Choose a cache name - increment version when assets change
import("./cache-version.js").then((m) => {

const CACHE_NAME = m.CACHE_NAME;

console.log("PWA cache name " + CACHE_NAME);
// List the files you want to cache initially
// Include the iframe's HTML, the JS glue code, and the WASM file itself

const addSubFolderUrls = (name) => ([name + "/wasm.d.ts", name + "/wasm.js", name + "/wasm_bg.wasm", name + "/wasm_bg.wasm.d.ts"]) 

const URLS_TO_CACHE = [
  'game.html',
  'main.js',
  'sw.js',
  "style.css",
  ...addSubFolderUrls("map_preview"),
  ...addSubFolderUrls("character_tester")
];

// --- Installation Event ---
// This runs when the SW is first installed.
// It opens the cache and adds the core files.
self.addEventListener('install', (event) => {
  console.log('[Service Worker] Installing...');
  // Prevent the SW from becoming active until caching is complete
  event.waitUntil(
    caches.open(CACHE_NAME)
      .then((cache) => {
        console.log('[Service Worker] Opened cache:', CACHE_NAME);
        // Add all specified URLs to the cache
        return cache.addAll(URLS_TO_CACHE);
      })
      .then(() => {
        console.log('[Service Worker] Core assets cached successfully.');
        // Force the waiting service worker to become the active service worker.
        // Useful for development/immediate activation after changes.
        // Remove self.skipWaiting() for production if you want controlled updates.
        return self.skipWaiting();
      })
      .catch(error => {
        console.error('[Service Worker] Cache addAll failed:', error);
      })
  );
});

// --- Activation Event ---
// This runs after install and when the SW becomes active.
// It's a good place to clean up old caches.
self.addEventListener('activate', (event) => {
  console.log('[Service Worker] Activating...');
  event.waitUntil(
    caches.keys().then((cacheNames) => {
      return Promise.all(
        cacheNames.map((cacheName) => {
          // If the cache name is different from the current one, delete it
          if (cacheName !== CACHE_NAME) {
            console.log('[Service Worker] Deleting old cache:', cacheName);
            return caches.delete(cacheName);
          }
        })
      );
    }).then(() => {
        console.log('[Service Worker] Claiming clients...');
        // Take control of currently open clients (pages/iframes) immediately
        return self.clients.claim();
    })
  );
});

// --- Fetch Event ---
// This runs every time the page controlled by the SW makes a network request.
// It intercepts the request and decides whether to serve from cache or network.
self.addEventListener('fetch', (event) => {
  // Only handle GET requests for caching purposes
  if (event.request.method !== 'GET') {
    return;
  }

  // Cache-First Strategy: Try cache, fallback to network
  event.respondWith(
    caches.match(event.request)
      .then((cachedResponse) => {
        // 1. If a cached response is found, return it
        if (cachedResponse) {
          // console.log('[Service Worker] Serving from cache:', event.request.url);
          return cachedResponse;
        }

        // 2. If not in cache, fetch from the network
        // console.log('[Service Worker] Fetching from network:', event.request.url);
        return fetch(event.request).then(
          (networkResponse) => {
            // 2a. Optional: Cache the newly fetched resource for next time
            // Clone the response because response streams can only be consumed once
            const responseToCache = networkResponse.clone();
            caches.open(CACHE_NAME)
              .then((cache) => {
                // console.log('[Service Worker] Caching new resource:', event.request.url);
                cache.put(event.request, responseToCache);
              });

            // 2b. Return the network response to the browser
            return networkResponse;
          }
        ).catch(error => {
            console.error('[Service Worker] Fetch failed:', error);
            // Optional: Return a fallback offline page/resource here
            // return caches.match('/offline.html');
        });
      })
  );
});
});