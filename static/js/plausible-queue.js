/**
 * Plausible's command queue: calls made before the analytics script has
 * loaded are kept and replayed once it arrives. It lives in a file instead of
 * an inline script so the Content-Security-Policy does not need
 * 'unsafe-inline' for scripts.
 */
window.plausible = window.plausible || function () {
    (window.plausible.q = window.plausible.q || []).push(arguments);
};
