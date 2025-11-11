/**
 * Language Selector Module
 *
 * This module manages the language selection dropdown.
 * To add a new language:
 * 1. Add it to the Language enum in src/language.rs
 * 2. Add it to the LANGUAGES array below
 * 3. Update any language-specific strings in templates as needed
 */

// Language configuration - Add new languages here easily
const LANGUAGES = [
    { code: 'es', name: 'Español' },
    { code: 'en', name: 'English' }
];

/**
 * Initialize the language selector dropdown
 * @param {string} currentLang - The current language code (e.g., 'en', 'es')
 */
function initLanguageSelector(currentLang) {
    const dropdownBtn = document.getElementById('lang-dropdown-btn');
    const dropdownMenu = document.getElementById('lang-dropdown-menu');
    const currentLangFlag = document.getElementById('current-lang-flag');

    if (!dropdownBtn || !dropdownMenu) {
        console.error('Language selector elements not found');
        return;
    }

    // Update the current language code display
    const currentLanguage = LANGUAGES.find(lang => lang.code === currentLang);
    if (currentLanguage) {
        currentLangFlag.textContent = currentLanguage.code.toUpperCase();
    }

    // Build the dropdown menu
    dropdownMenu.innerHTML = LANGUAGES
        .map(lang => {
            const isActive = lang.code === currentLang;
            return `
                <button onclick="switchLanguage('${lang.code}')"
                    class="w-full px-4 py-2 text-left text-white hover:bg-gray-700 dark:hover:bg-gray-600 transition-colors ${
                        isActive ? 'bg-blue-600 dark:bg-blue-700 font-bold' : ''
                    }">
                    ${lang.name}
                </button>
            `;
        })
        .join('');

    // Toggle dropdown visibility
    dropdownBtn.addEventListener('click', (e) => {
        e.stopPropagation();
        dropdownMenu.classList.toggle('hidden');
    });

    // Close dropdown when clicking outside
    document.addEventListener('click', (e) => {
        if (!dropdownBtn.contains(e.target) && !dropdownMenu.contains(e.target)) {
            dropdownMenu.classList.add('hidden');
        }
    });

    // Close dropdown when a language is selected
    dropdownMenu.addEventListener('click', () => {
        dropdownMenu.classList.add('hidden');
    });
}

/**
 * Switch to a different language
 * @param {string} newLang - The language code to switch to
 */
function switchLanguage(newLang) {
    const currentPath = window.location.pathname;

    // Remove current language prefix if present
    // Matches patterns like /en, /es, /fr, etc. at the start
    let pathWithoutLang = currentPath.replace(/^\/(en|es|fr|de|it|pt|ja|zh|ru)(?:\/|$)/, '/');

    // If path is just /, make it empty
    if (pathWithoutLang === '/') {
        pathWithoutLang = '';
    }

    // Navigate to new language path
    window.location.href = '/' + newLang + pathWithoutLang;
}
