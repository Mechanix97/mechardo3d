/**
 * Language selector.
 *
 * To add a language: add it to the Language enum in src/language.rs and to
 * LANGUAGES below. The current language is read from <html lang>, so the
 * template needs no inline bootstrap script.
 */
const LANGUAGES = [
    { code: 'es', name: 'Español' },
    { code: 'en', name: 'English' },
];

const COOKIE_MAX_AGE = 60 * 60 * 24 * 365;

function switchLanguage(newLang) {
    document.cookie = `language=${newLang}; path=/; max-age=${COOKIE_MAX_AGE}; SameSite=Lax`;

    const codes = LANGUAGES.map(language => language.code).join('|');
    const rest = window.location.pathname.replace(new RegExp(`^/(${codes})(?=/|$)`), '');
    window.location.href = `/${newLang}${rest}${window.location.search}${window.location.hash}`;
}

function initLanguageSelector(currentLang) {
    const button = document.getElementById('lang-dropdown-btn');
    const menu = document.getElementById('lang-dropdown-menu');
    const label = document.getElementById('current-lang-flag');
    if (!button || !menu) return;

    const current = currentLang || document.documentElement.lang || LANGUAGES[0].code;
    if (label) label.textContent = current.toUpperCase();

    const items = LANGUAGES.map(language => {
        const item = document.createElement('button');
        item.type = 'button';
        item.setAttribute('role', 'menuitem');
        item.lang = language.code;
        item.textContent = language.name;
        item.className = 'w-full px-4 py-2 text-left text-white hover:bg-gray-700 dark:hover:bg-gray-600 ' +
            'focus:outline-none focus:bg-gray-700 dark:focus:bg-gray-600 transition-colors' +
            (language.code === current ? ' bg-blue-600 dark:bg-blue-700 font-bold' : '');
        if (language.code === current) item.setAttribute('aria-current', 'true');
        item.addEventListener('click', () => switchLanguage(language.code));
        menu.appendChild(item);
        return item;
    });

    const isOpen = () => !menu.classList.contains('hidden');

    function open(focusIndex) {
        menu.classList.remove('hidden');
        button.setAttribute('aria-expanded', 'true');
        if (typeof focusIndex === 'number' && items[focusIndex]) items[focusIndex].focus();
    }

    function close(returnFocus) {
        menu.classList.add('hidden');
        button.setAttribute('aria-expanded', 'false');
        if (returnFocus) button.focus();
    }

    button.addEventListener('click', (event) => {
        event.stopPropagation();
        isOpen() ? close(false) : open();
    });

    button.addEventListener('keydown', (event) => {
        if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
            event.preventDefault();
            open(event.key === 'ArrowDown' ? 0 : items.length - 1);
        }
    });

    menu.addEventListener('keydown', (event) => {
        const index = items.indexOf(document.activeElement);
        if (event.key === 'Escape') {
            event.preventDefault();
            close(true);
        } else if (event.key === 'ArrowDown' && index !== -1) {
            event.preventDefault();
            items[(index + 1) % items.length].focus();
        } else if (event.key === 'ArrowUp' && index !== -1) {
            event.preventDefault();
            items[(index - 1 + items.length) % items.length].focus();
        } else if (event.key === 'Tab') {
            close(false);
        }
    });

    document.addEventListener('click', (event) => {
        if (isOpen() && !button.contains(event.target) && !menu.contains(event.target)) close(false);
    });
}

document.addEventListener('DOMContentLoaded', () => initLanguageSelector());
