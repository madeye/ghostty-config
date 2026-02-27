// Keyboard shortcut capture for keybinding editor
let capturing = false;

function startCapture() {
    capturing = true;
    const input = document.getElementById('keybind-trigger');
    const btn = document.getElementById('capture-btn');
    input.value = '';
    input.placeholder = 'Press your shortcut...';
    input.classList.add('ring-2', 'ring-indigo-500', 'bg-indigo-50');
    btn.textContent = 'Listening... (press Escape to cancel)';

    // Focus the input so key events work
    input.focus();
}

function stopCapture() {
    capturing = false;
    const input = document.getElementById('keybind-trigger');
    const btn = document.getElementById('capture-btn');
    input.classList.remove('ring-2', 'ring-indigo-500', 'bg-indigo-50');
    input.placeholder = 'e.g., super+shift+n';
    btn.textContent = 'Click to record shortcut...';
}

function selectMouseButton(select) {
    const value = select.value;
    if (value) {
        const input = document.getElementById('keybind-trigger');
        input.value = value;
    }
    select.selectedIndex = 0;
}

document.addEventListener('keydown', function(e) {
    if (!capturing) return;

    e.preventDefault();
    e.stopPropagation();

    // Cancel on Escape
    if (e.key === 'Escape') {
        stopCapture();
        return;
    }

    // Skip modifier-only presses
    if (['Control', 'Alt', 'Shift', 'Meta'].includes(e.key)) {
        return;
    }

    const parts = [];

    // Build modifier string using Ghostty naming
    if (e.metaKey) parts.push('super');
    if (e.ctrlKey) parts.push('ctrl');
    if (e.altKey) parts.push('alt');
    if (e.shiftKey) parts.push('shift');

    // Map key name to Ghostty format
    let keyName = e.key.toLowerCase();

    // Common key mappings
    const keyMap = {
        'arrowup': 'arrow_up',
        'arrowdown': 'arrow_down',
        'arrowleft': 'arrow_left',
        'arrowright': 'arrow_right',
        'enter': 'enter',
        'return': 'enter',
        'backspace': 'backspace',
        'delete': 'delete',
        'tab': 'tab',
        'escape': 'escape',
        'space': 'space',
        ' ': 'space',
        'pageup': 'page_up',
        'pagedown': 'page_down',
        'home': 'home',
        'end': 'end',
        '[': 'bracket_left',
        ']': 'bracket_right',
        ',': 'comma',
        '.': 'period',
        '/': 'slash',
        '\\': 'backslash',
        ';': 'semicolon',
        "'": 'apostrophe',
        '`': 'grave_accent',
        '-': 'minus',
        '=': 'equal',
    };

    if (keyMap[keyName]) {
        keyName = keyMap[keyName];
    }

    // Function keys
    if (e.key.match(/^F\d+$/i)) {
        keyName = e.key.toLowerCase();
    }

    parts.push(keyName);

    const input = document.getElementById('keybind-trigger');
    input.value = parts.join('+');

    stopCapture();
});
