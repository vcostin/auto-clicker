const invoke = window.__TAURI_INVOKE__;

let isRunning = false;
let selectedWindow = '';
let cpsValue = 10;
let triggerKey = 'Alt';

const refreshWindowsBtn = document.getElementById('refreshWindows');
const windowSelect = document.getElementById('windowSelect');
const cpsSlider = document.getElementById('cpsSlider');
const cpsInput = document.getElementById('cpsValue');
const startBtn = document.getElementById('startBtn');
const stopBtn = document.getElementById('stopBtn');
const statusDiv = document.getElementById('status');
const triggerKeyInputs = document.querySelectorAll('input[name="triggerKey"]');

async function refreshWindows() {
    try {
        const windows = await invoke('get_windows');
        windowSelect.innerHTML = '<option value="">Select a window...</option>';

        windows.forEach(window => {
            const option = document.createElement('option');
            option.value = window;
            option.textContent = window;
            windowSelect.appendChild(option);
        });
    } catch (error) {
        console.error('Failed to get windows:', error);
        alert('Failed to get windows: ' + error);
    }
}

async function startAutoClicker() {
    if (!selectedWindow) {
        alert('Please select a target window first!');
        return;
    }

    try {
        await invoke('start_autoclicker', {
            windowTitle: selectedWindow,
            cps: cpsValue,
            triggerKey: triggerKey
        });

        isRunning = true;
        startBtn.disabled = true;
        stopBtn.disabled = false;
        statusDiv.textContent = 'Waiting for trigger key...';
        statusDiv.classList.add('active');
    } catch (error) {
        console.error('Failed to start autoclicker:', error);
        alert('Failed to start autoclicker: ' + error);
    }
}

async function stopAutoClicker() {
    try {
        await invoke('stop_autoclicker');

        isRunning = false;
        startBtn.disabled = false;
        stopBtn.disabled = true;
        statusDiv.textContent = 'Inactive';
        statusDiv.classList.remove('active');
    } catch (error) {
        console.error('Failed to stop autoclicker:', error);
        alert('Failed to stop autoclicker: ' + error);
    }
}

refreshWindowsBtn.addEventListener('click', refreshWindows);

windowSelect.addEventListener('change', (e) => {
    selectedWindow = e.target.value;
});

cpsSlider.addEventListener('input', (e) => {
    cpsValue = parseInt(e.target.value);
    cpsInput.value = cpsValue;
});

cpsInput.addEventListener('input', (e) => {
    let value = parseInt(e.target.value);
    if (isNaN(value)) value = 1;
    if (value < 1) value = 1;
    if (value > 50) value = 50;

    cpsValue = value;
    cpsSlider.value = value;
    cpsInput.value = value;
});

triggerKeyInputs.forEach(input => {
    input.addEventListener('change', (e) => {
        if (e.target.checked) {
            triggerKey = e.target.value;
        }
    });
});

startBtn.addEventListener('click', startAutoClicker);
stopBtn.addEventListener('click', stopAutoClicker);

refreshWindows();
