// settingsStore.js
import { writable } from 'svelte/store';


export interface Settings {
  matchboxServer: string,
  lobbyName: string,
  playerCount: number
}

function generateRandomString(length: number) {
    const characters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    let result = '';
    for (let i = 0; i < length; i++) {
        const randomIndex = Math.floor(Math.random() * characters.length);
        result += characters.charAt(randomIndex);
    }
    return result;
}

// Default settings
const defaultSettings: Settings = {
  matchboxServer: 'wss://matchbox.example.com',
  lobbyName: generateRandomString(6),
  playerCount: 2
};

// Create a writable store with default values
const createSettingsStore = () => {
  // Initialize with defaults
  const { subscribe, set, update } = writable(defaultSettings);

  return {
    subscribe,
    set,
    update,
    // Load settings from localStorage
    load: () => {
      try {
        const storedSettings = localStorage.getItem('appSettings');
        if (storedSettings) {
          // Merge with defaults in case new settings were added
          set({ ...defaultSettings, ...JSON.parse(storedSettings) });
        }
      } catch (error) {
        console.error('Failed to load settings:', error);
      }
    },
    // Save current settings to localStorage
    save: (settings: Settings) => {
      try {
        localStorage.setItem('appSettings', JSON.stringify(settings));
        return true;
      } catch (error) {
        console.error('Failed to save settings:', error);
        return false;
      }
    },
    // Reset to defaults
    reset: () => {
      set(defaultSettings);
      localStorage.setItem('appSettings', JSON.stringify(defaultSettings));
    }
  };
};

// Create and export the settings store
export const settingsStore = createSettingsStore();

// Initialize settings on app start
if (typeof window !== 'undefined') {
  settingsStore.load();
}