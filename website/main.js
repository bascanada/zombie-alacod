// Get references to HTML elements
const sidebarButtons = document.querySelectorAll('#sidebar button[data-app]');
const reloadButton = document.getElementById('reload-button');
const unloadButton = document.getElementById('unload-button'); // Get unload button
const appFrame = document.getElementById('app-frame');

// --- Attach Event Listeners ---

const getFrameSrc = (appName, online, lobby) => {
    return "./game.html?name=" + appName + "&online=" + online + "&lobby=" + lobby;
}

// App loading buttons
sidebarButtons.forEach(button => {
    button.addEventListener('click', () => {

        const urlParams = new URLSearchParams(window.location.search);
        console.log(button.dataset);
        const appPath = button.dataset.app; // Get the HTML path
        const onlinePath = button.dataset.online;
        const lobby = urlParams.get("lobby") || "";
        if (appPath) {
            console.log(`Loading app from: ${appPath}`);
            appFrame.src = getFrameSrc(appPath, onlinePath, lobby); // Set the iframe source
        }
    });
});

// Reload button
reloadButton.addEventListener('click', () => {
    const currentSrc = appFrame.getAttribute('src'); // Get current src safely
    if (currentSrc && currentSrc !== 'about:blank') {
        console.log(`Reloading iframe content: ${currentSrc}`);
        // Re-assigning the src forces the iframe to reload
        appFrame.src = currentSrc;
    } else {
        console.log("No app loaded in iframe to reload.");
    }
});

// Unload button (Optional)
unloadButton.addEventListener('click', () => {
    console.log("Unloading app from iframe.");
    appFrame.src = 'about:blank'; // Load an empty page
});

// --- Initial State ---
// The iframe starts empty (src="about:blank")
console.log("Iframe loader initialized. Select an app.");


// --- Service Worker Registration ---
if ('serviceWorker' in navigator) {
    const hostname = window.location.hostname;

    if (hostname !== 'localhost' && hostname !== '127.0.0.1') {
        window.addEventListener('load', () => {
            navigator.serviceWorker.register('/sw.js') // Path relative to origin root
                .then((registration) => {
                    console.log('Service Worker registered successfully with scope: ', registration.scope);
                    document.getElementById('status').textContent = 'Service Worker registered. App should load faster next time!';
                })
                .catch((error) => {
                    console.error('Service Worker registration failed: ', error);
                    document.getElementById('status').textContent = 'Service Worker registration failed.';
                });
        });
    }
} else {
    console.log('Service Workers not supported in this browser.');
    document.getElementById('status').textContent = 'Service Workers not supported. App will load normally.';
}