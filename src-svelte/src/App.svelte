<script>
  import { onMount } from 'svelte';

  // State
  let currentView = $state('setup'); // 'setup' | 'session'
  let sessionState = $state('idle'); // 'idle' | 'recording' | 'processing'
  let appMode = $state('autonomous'); // 'autonomous' | 'collaborative'
  let currentEmotion = $state('neutral');
  let currentTrack = $state(null);
  let isPlaying = $state(false);
  let devices = $state([]);
  let selectedDevice = $state('');
  let statusMessage = $state('');

  // Audio engine state
  let musicVolume = $state(70);
  let sfxVolume = $state(80);
  let crossfadeType = $state('musical');

  // Available tracks (sample)
  let tracks = $state([
    { id: '1', name: 'Battle Theme', genre: 'combat', mood: 'angry', is_looping: true },
    { id: '2', name: 'Dungeon Ambience', genre: 'exploration', mood: 'neutral', is_looping: true },
    { id: '3', name: 'Victory Fanfare', genre: 'social', mood: 'happy', is_looping: false },
    { id: '4', name: 'Mystery Theme', genre: 'exploration', mood: 'fearful', is_looping: true },
    { id: '5', name: 'Tavern Music', genre: 'social', mood: 'happy', is_looping: true },
  ]);

  onMount(async () => {
    await loadDevices();
    // Listen for Tauri events
    if (window.__TAURI__) {
      const { listen } = window.__TAURI__.event;
      listen('session-status-changed', (event) => {
        sessionState = event.payload.status;
      });
      listen('emotion-result', (event) => {
        currentEmotion = event.payload.emotion;
      });
    }
  });

  async function loadDevices() {
    try {
      if (window.__TAURI__) {
        devices = await window.__TAURI__.invoke('get_available_devices');
        if (devices.length > 0) {
          selectedDevice = devices.find(d => d.is_default)?.id || devices[0].id;
        }
      }
    } catch (e) {
      console.error('Failed to load devices:', e);
      // Use mock devices for dev
      devices = [
        { id: 'default', name: 'Default Microphone', is_default: true }
      ];
      selectedDevice = 'default';
    }
  }

  async function startSession() {
    try {
      statusMessage = 'Starting session...';
      const result = await window.__TAURI__.invoke('start_session', {
        deviceId: selectedDevice,
        enableTranscription: true,
        enableEmotion: true
      });
      if (result.success) {
        sessionState = 'recording';
        currentView = 'session';
        statusMessage = '';
      } else {
        statusMessage = result.message;
      }
    } catch (e) {
      console.error('Failed to start session:', e);
      statusMessage = 'Failed to start: ' + e;
      // For dev, simulate success
      sessionState = 'recording';
      currentView = 'session';
    }
  }

  async function stopSession() {
    try {
      statusMessage = 'Stopping session...';
      const result = await window.__TAURI__.invoke('stop_session');
      if (result.success) {
        sessionState = 'idle';
        statusMessage = '';
      }
    } catch (e) {
      console.error('Failed to stop session:', e);
      sessionState = 'idle';
    }
  }

  async function toggleMode() {
    try {
      const newMode = appMode === 'autonomous' ? 'collaborative' : 'autonomous';
      await window.__TAURI__.invoke('set_app_mode', { mode: newMode });
      appMode = newMode;
    } catch (e) {
      // Toggle locally for dev
      appMode = appMode === 'autonomous' ? 'collaborative' : 'autonomous';
    }
  }

  function getEmotionColor(emotion) {
    const colors = {
      neutral: 'bg-gray-500',
      happy: 'bg-yellow-500',
      sad: 'bg-blue-500',
      angry: 'bg-red-500',
      fearful: 'bg-purple-500',
      surprised: 'bg-orange-500',
      disgusted: 'bg-green-500',
    };
    return colors[emotion] || 'bg-gray-500';
  }
</script>

<main class="min-h-screen bg-gray-900 text-white">
  {#if currentView === 'setup'}
    <!-- Setup Screen -->
    <div class="container mx-auto px-4 py-8 max-w-2xl">
      <h1 class="text-3xl font-bold text-center mb-8 text-primary-500">TTRPG Companion</h1>
      <p class="text-gray-400 text-center mb-8">Real-time audio companion for tabletop RPG sessions</p>

      <!-- Device Selection -->
      <div class="bg-gray-800 rounded-lg p-6 mb-6">
        <h2 class="text-xl font-semibold mb-4">Audio Input</h2>
        <select
          bind:value={selectedDevice}
          class="w-full bg-gray-700 border border-gray-600 rounded px-4 py-2 focus:outline-none focus:border-primary-500"
        >
          {#each devices as device}
            <option value={device.id}>{device.name}</option>
          {/each}
        </select>
      </div>

      <!-- Quick Settings -->
      <div class="bg-gray-800 rounded-lg p-6 mb-6">
        <h2 class="text-xl font-semibold mb-4">Quick Settings</h2>

        <div class="mb-4">
          <label class="block mb-2">Music Volume: {musicVolume}%</label>
          <input type="range" min="0" max="100" bind:value={musicVolume} class="w-full" />
        </div>

        <div class="mb-4">
          <label class="block mb-2">SFX Volume: {sfxVolume}%</label>
          <input type="range" min="0" max="100" bind:value={sfxVolume} class="w-full" />
        </div>

        <div>
          <label class="block mb-2">Crossfade: {crossfadeType}</label>
          <select bind:value={crossfadeType} class="w-full bg-gray-700 border border-gray-600 rounded px-4 py-2">
            <option value="instant">Instant</option>
            <option value="quick">Quick (0.5s)</option>
            <option value="musical">Musical (2s)</option>
            <option value="long">Long (5s)</option>
          </select>
        </div>
      </div>

      <!-- Status Message -->
      {#if statusMessage}
        <div class="bg-yellow-600/20 border border-yellow-600 rounded p-4 mb-6 text-yellow-400">
          {statusMessage}
        </div>
      {/if}

      <!-- Start Button -->
      <button
        onclick={startSession}
        class="w-full bg-primary-600 hover:bg-primary-700 text-white font-bold py-4 px-8 rounded-lg transition-colors text-lg"
      >
        Start Session
      </button>
    </div>

  {:else}
    <!-- Session Screen -->
    <div class="container mx-auto px-4 py-6">
      <!-- Header -->
      <div class="flex justify-between items-center mb-6">
        <h1 class="text-2xl font-bold">Session Active</h1>
        <div class="flex items-center gap-4">
          <span class="text-gray-400">Mode: {appMode}</span>
          <button
            onclick={toggleMode}
            class="px-3 py-1 bg-gray-700 rounded hover:bg-gray-600"
          >
            Toggle
          </button>
        </div>
      </div>

      <!-- Status Bar -->
      <div class="bg-gray-800 rounded-lg p-4 mb-6">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-4">
            <div class={`w-4 h-4 rounded-full ${sessionState === 'recording' ? 'bg-red-500 animate-pulse' : 'bg-gray-500'}`}></div>
            <span class="text-lg capitalize">{sessionState}</span>
          </div>

          <!-- Emotion Indicator -->
          <div class="flex items-center gap-2">
            <span class="text-gray-400">Detected:</span>
            <span class={`px-3 py-1 rounded ${getEmotionColor(currentEmotion)}`}>
              {currentEmotion}
            </span>
          </div>
        </div>
      </div>

      <!-- Now Playing -->
      <div class="bg-gray-800 rounded-lg p-6 mb-6">
        <h2 class="text-xl font-semibold mb-4">Now Playing</h2>
        {#if currentTrack}
          <div class="flex items-center gap-4">
            <div class="w-16 h-16 bg-primary-700 rounded flex items-center justify-center">
              <svg class="w-8 h-8" fill="currentColor" viewBox="0 0 24 24">
                <path d="M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z"/>
              </svg>
            </div>
            <div>
              <p class="font-semibold">{currentTrack.name}</p>
              <p class="text-gray-400 text-sm">{currentTrack.genre} - {currentTrack.mood}</p>
            </div>
          </div>
        {:else}
          <p class="text-gray-500">No track playing</p>
        {/if}
      </div>

      <!-- Track Library -->
      <div class="bg-gray-800 rounded-lg p-6 mb-6">
        <h2 class="text-xl font-semibold mb-4">Track Library</h2>
        <div class="grid gap-2">
          {#each tracks as track}
            <button
              onclick={() => currentTrack = track}
              class="flex items-center justify-between p-3 bg-gray-700 rounded hover:bg-gray-600 transition-colors text-left"
            >
              <div>
                <p class="font-medium">{track.name}</p>
                <p class="text-sm text-gray-400">{track.genre} - {track.mood}</p>
              </div>
              {#if track.is_looping}
                <span class="text-xs bg-primary-900 text-primary-300 px-2 py-1 rounded">Loop</span>
              {/if}
            </button>
          {/each}
        </div>
      </div>

      <!-- Volume Controls -->
      <div class="bg-gray-800 rounded-lg p-6 mb-6">
        <h2 class="text-xl font-semibold mb-4">Volume</h2>
        <div class="space-y-4">
          <div>
            <label class="block mb-2">Music: {musicVolume}%</label>
            <input type="range" min="0" max="100" bind:value={musicVolume} class="w-full" />
          </div>
          <div>
            <label class="block mb-2">SFX: {sfxVolume}%</label>
            <input type="range" min="0" max="100" bind:value={sfxVolume} class="w-full" />
          </div>
        </div>
      </div>

      <!-- Stop Button -->
      <button
        onclick={stopSession}
        class="w-full bg-red-600 hover:bg-red-700 text-white font-bold py-3 px-6 rounded-lg transition-colors"
      >
        Stop Session
      </button>
    </div>
  {/if}
</main>
