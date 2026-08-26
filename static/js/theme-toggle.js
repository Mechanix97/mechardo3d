document.addEventListener('DOMContentLoaded', () => {
    const html = document.documentElement;
    const themeToggle = document.getElementById('theme-toggle');
    const sunIcon = document.getElementById('sun');
    const moonIcon = document.getElementById('moon');
    if (!themeToggle) return;

    // Reflect the current theme in the icon and in the button's state, so the
    // control is understandable without seeing it.
    const render = () => {
        const isDark = html.classList.contains('dark');
        if (sunIcon && moonIcon) {
            sunIcon.classList.toggle('hidden', !isDark);
            moonIcon.classList.toggle('hidden', isDark);
        }
        themeToggle.setAttribute('aria-pressed', String(isDark));
        const label = isDark ? themeToggle.dataset.labelLight : themeToggle.dataset.labelDark;
        if (label) themeToggle.setAttribute('aria-label', label);
    };

    render();

    themeToggle.addEventListener('click', () => {
        const isDark = html.classList.toggle('dark');
        localStorage.setItem('theme', isDark ? 'dark' : 'light');
        render();
    });

    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
        if (!localStorage.getItem('theme')) {
            html.classList.toggle('dark', e.matches);
            render();
        }
    });
});
