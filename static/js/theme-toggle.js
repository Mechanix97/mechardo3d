(function () {
    console.log('theme-toggle.js loaded');

    const toggleButton = document.getElementById('theme-toggle');
    const sunPath = document.getElementById('sun');
    const moonPath = document.getElementById('moon');

    if (!toggleButton || !sunPath || !moonPath) {
        console.error('Theme toggle elements not found');
        return;
    }

    const savedTheme = localStorage.getItem('theme');
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;

    const isDarkMode = savedTheme === 'dark' || (!savedTheme && prefersDark);

    document.documentElement.classList.toggle('dark', isDarkMode);
    sunPath.classList.toggle('hidden', isDarkMode);
    moonPath.classList.toggle('hidden', !isDarkMode);

    toggleButton.addEventListener('click', () => {
        const isDark = document.documentElement.classList.toggle('dark');
        localStorage.setItem('theme', isDark ? 'dark' : 'light');
        sunPath.classList.toggle('hidden', isDark);
        moonPath.classList.toggle('hidden', !isDark);
        console.log('Toggled theme to', isDark ? 'dark' : 'light');
    });
})();


