// TTRPG Companion - Frontend JavaScript

// Global state
const state = {
    isRecording: false,
    devices: [],
    currentDevice: null
};

// DOM Elements
const elements = {
    audioDeviceSelect: document.getElementById('audio-device'),
    enableTranscription: document.getElementById('enable-transcription'),
    enableEmotion: document.getElementById('enable-emotion'),
    startBtn: document.getElementById('start-btn'),
    stopBtn: document.getElementById('stop-btn'),
    statusIndicator: document.getElementById('status-indicator'),
    statusText: document.getElementById('status-text'),
    configSection: document.getElementById('config-section'),
    sessionSection: document.getElementById('session-section'),
    resultsSection: document.getElementById('results-section'),
    transcriptionResult: document.getElementById('transcription-result'),
    emotionResult: document.getElementById('emotion-result'),
    logOutput: document.getElementById('log-output')
};

// Initialize application
async function init() {
    log('info', 'Initializing TTRPG Companion...');

    // Load audio devices
    await loadAudioDevices();

    // Set up event listeners
    setupEventListeners();

    // Get initial session status
    await checkSessionStatus();

    log('success', 'Application initialized');
}

// Load available audio devices
async function loadAudioDevices() {
    try {
        // Try to get devices from Tauri backend
        const devices = await window.__TAURI__.invoke('get_available_devices');
        state.devices = devices;

        elements.audioDeviceSelect.innerHTML = '';

        if (devices.length === 0) {
            elements.audioDeviceSelect.innerHTML = '<option value="">No devices found</option>';
        } else {
            devices.forEach((device, index) => {
                const option = document.createElement('option');
                option.value = device.name;
                option.textContent = device.name;
                if (index === 0) {
                    option.selected = true;
                    state.currentDevice = device.name;
                }
                elements.audioDeviceSelect.appendChild(option);
            });
        }

        log('info', `Found ${devices.length} audio device(s)`);
    } catch (error) {
        log('error', `Failed to load devices: ${error}`);
        // Fallback to default option
        elements.audioDeviceSelect.innerHTML = '<option value="default">Default Device</option>';
    }
}

// Set up event listeners
function setupEventListeners() {
    // Audio device selection
    elements.audioDeviceSelect.addEventListener('change', (e) => {
        state.currentDevice = e.target.value;
        log('info', `Selected device: ${state.currentDevice}`);
    });

    // Start button
    elements.startBtn.addEventListener('click', startRecording);

    // Stop button
    elements.stopBtn.addEventListener('click', stopRecording);
}

// Start recording session
async function startRecording() {
    try {
        log('info', 'Starting recording session...');

        const response = await window.__TAURI__.invoke('start_session');

        if (response.success) {
            state.isRecording = true;
            updateUIState('recording');
            log('success', 'Recording started');
        } else {
            log('error', `Failed to start: ${response.message}`);
        }
    } catch (error) {
        log('error', `Error starting session: ${error}`);
    }
}

// Stop recording session
async function stopRecording() {
    try {
        log('info', 'Stopping recording session...');

        updateUIState('processing');

        const response = await window.__TAURI__.invoke('stop_session');

        state.isRecording = false;

        if (response.success) {
            // Show results
            elements.resultsSection.classList.remove('hidden');

            // Parse message for results
            const lines = response.message.split('\n');
            lines.forEach(line => {
                if (line.startsWith('Transcription:')) {
                    elements.transcriptionResult.textContent = line.replace('Transcription: ', '');
                } else if (line.startsWith('Emotion:')) {
                    elements.emotionResult.textContent = line.replace('Emotion: ', '');
                }
            });

            updateUIState('idle');
            log('success', 'Recording stopped and processed');
        } else {
            updateUIState('error');
            log('error', `Failed to stop: ${response.message}`);
        }
    } catch (error) {
        log('error', `Error stopping session: ${error}`);
        updateUIState('error');
    }
}

// Check current session status
async function checkSessionStatus() {
    try {
        const status = await window.__TAURI__.invoke('get_session_status');
        state.isRecording = status.is_recording;

        if (status.is_recording) {
            updateUIState('recording');
        } else {
            updateUIState('idle');
        }
    } catch (error) {
        log('error', `Error checking status: ${error}`);
    }
}

// Update UI based on state
function updateUIState(sessionState) {
    switch (sessionState) {
        case 'idle':
            elements.statusIndicator.className = 'status-indicator';
            elements.statusText.textContent = 'Idle';
            elements.startBtn.disabled = false;
            elements.stopBtn.disabled = true;
            elements.configSection.classList.remove('hidden');
            break;

        case 'recording':
            elements.statusIndicator.className = 'status-indicator recording';
            elements.statusText.textContent = 'Recording...';
            elements.startBtn.disabled = true;
            elements.stopBtn.disabled = false;
            elements.configSection.classList.add('hidden');
            break;

        case 'processing':
            elements.statusIndicator.className = 'status-indicator processing';
            elements.statusText.textContent = 'Processing...';
            elements.startBtn.disabled = true;
            elements.stopBtn.disabled = true;
            break;

        case 'error':
            elements.statusIndicator.className = 'status-indicator';
            elements.statusText.textContent = 'Error';
            elements.startBtn.disabled = false;
            elements.stopBtn.disabled = true;
            break;
    }
}

// Add log entry
function log(level, message) {
    const entry = document.createElement('div');
    entry.className = `log-entry ${level}`;
    entry.textContent = `[${new Date().toLocaleTimeString()}] ${message}`;
    elements.logOutput.appendChild(entry);

    // Auto-scroll to bottom
    elements.logOutput.scrollTop = elements.logOutput.scrollHeight;

    // Also log to console
    console[level === 'error' ? 'error' : 'log'](message);
}

// Wait for Tauri to be ready
document.addEventListener('DOMContentLoaded', init);
