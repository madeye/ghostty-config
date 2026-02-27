// Ghostty Config UI - Helpers

// Auto-dismiss toasts
document.addEventListener('htmx:afterSwap', function(e) {
    if (e.detail.target.id === 'toast-container') {
        const toast = e.detail.target.firstElementChild;
        if (toast) {
            setTimeout(() => {
                toast.style.transition = 'opacity 0.3s ease-out';
                toast.style.opacity = '0';
                setTimeout(() => toast.remove(), 300);
            }, 2000);
        }
    }
});

// Color picker sync: when typing hex, update the color input
document.addEventListener('input', function(e) {
    if (e.target.type === 'text' && e.target.id && e.target.id.startsWith('input-')) {
        const key = e.target.id.replace('input-', '');
        const colorInput = document.getElementById('color-' + key);
        if (colorInput && e.target.value.match(/^#[0-9a-fA-F]{6}$/)) {
            colorInput.value = e.target.value;
        }
    }
});

// Debounce helper
function debounce(fn, delay) {
    let timer;
    return function(...args) {
        clearTimeout(timer);
        timer = setTimeout(() => fn.apply(this, args), delay);
    };
}
