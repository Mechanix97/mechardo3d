/**
 * Full-screen navigation on phones.
 *
 * The panel is a modal dialog: opening it locks the page scroll and moves
 * focus inside, Escape or the close button put focus back on the opener, and
 * Tab cycles within the panel.
 */
document.addEventListener('DOMContentLoaded', () => {
    const panel = document.getElementById('site-menu');
    const openButton = document.getElementById('menu-open');
    const closeButton = document.getElementById('menu-close');
    if (!panel || !openButton || !closeButton) return;

    const focusable = () => Array.from(panel.querySelectorAll('a[href], button'));

    function open() {
        panel.hidden = false;
        openButton.setAttribute('aria-expanded', 'true');
        document.documentElement.style.overflow = 'hidden';
        closeButton.focus();
    }

    function close() {
        panel.hidden = true;
        openButton.setAttribute('aria-expanded', 'false');
        document.documentElement.style.overflow = '';
        openButton.focus();
    }

    openButton.addEventListener('click', open);
    closeButton.addEventListener('click', close);

    panel.addEventListener('keydown', (event) => {
        if (event.key === 'Escape') {
            event.preventDefault();
            close();
            return;
        }
        if (event.key !== 'Tab') return;

        const items = focusable();
        const first = items[0];
        const last = items[items.length - 1];
        if (event.shiftKey && document.activeElement === first) {
            event.preventDefault();
            last.focus();
        } else if (!event.shiftKey && document.activeElement === last) {
            event.preventDefault();
            first.focus();
        }
    });

    // Rotating a tablet past the breakpoint hides the panel with CSS; release
    // the scroll lock with it.
    window.matchMedia('(min-width: 768px)').addEventListener('change', (event) => {
        if (event.matches && !panel.hidden) close();
    });
});
