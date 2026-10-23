/**
 * DS2000 guide screenshots: a click on one opens it full size in a native
 * <dialog> on the same page (focus, Escape and the backdrop come for free),
 * instead of following the link to the bare image. A modified click (new tab,
 * new window) still follows the link. Without JS, or without <dialog>, the
 * link opens the image in the same tab.
 */
document.addEventListener('DOMContentLoaded', () => {
    const viewer = document.getElementById('figure-viewer');
    if (!viewer || typeof viewer.showModal !== 'function') return;
    const image = viewer.querySelector('[data-viewer-image]');
    const closeButton = viewer.querySelector('[data-viewer-close]');
    if (!image) return;

    let opener = null;

    document.querySelectorAll('[data-figure-viewer]').forEach((link) => {
        link.addEventListener('click', (event) => {
            if (event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;
            event.preventDefault();
            const thumbnail = link.querySelector('img');
            image.src = link.href;
            image.alt = thumbnail ? thumbnail.alt : '';
            opener = link;
            viewer.showModal();
        });
    });

    if (closeButton) closeButton.addEventListener('click', () => viewer.close());
    // A click on the backdrop lands on the dialog element itself.
    viewer.addEventListener('click', (event) => {
        if (event.target === viewer) viewer.close();
    });
    viewer.addEventListener('close', () => {
        image.removeAttribute('src');
        if (opener) opener.focus();
    });
});
