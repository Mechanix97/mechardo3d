/**
 * DS2000 gallery: thumbnails swap the main render, the enlarge button opens
 * it in a native <dialog> (focus, Escape and the backdrop come for free).
 */
document.addEventListener('DOMContentLoaded', () => {
    const gallery = document.querySelector('[data-gallery]');
    if (!gallery) return;

    const main = gallery.querySelector('[data-gallery-main]');
    const thumbs = Array.from(gallery.querySelectorAll('[data-gallery-thumb]'));
    const caption = gallery.querySelector('[data-gallery-caption]');
    const count = gallery.querySelector('[data-gallery-count]');
    const openButton = gallery.querySelector('[data-gallery-open]');
    const modal = document.getElementById('image-modal');
    const modalImage = document.getElementById('modal-image');
    const closeButton = document.getElementById('close-modal');
    if (!main) return;

    function select(index) {
        const thumb = thumbs[index];
        if (!thumb) return;
        main.src = thumb.dataset.src;
        main.alt = thumb.dataset.alt;
        if (caption) caption.textContent = thumb.dataset.alt;
        if (count) count.textContent = `${index + 1} / ${thumbs.length}`;
        if (openButton) openButton.setAttribute('aria-label', `${openButton.dataset.label} ${thumb.dataset.alt}`);

        thumbs.forEach((other, i) => {
            const active = i === index;
            other.setAttribute('aria-pressed', String(active));
            other.classList.toggle('border-lime', active);
            other.classList.toggle('border-line-strong', !active);
        });
    }

    thumbs.forEach((thumb, index) => thumb.addEventListener('click', () => select(index)));

    // Arrow keys move between thumbnails while one of them has focus.
    gallery.addEventListener('keydown', (event) => {
        const index = thumbs.indexOf(document.activeElement);
        if (index === -1) return;
        if (event.key !== 'ArrowLeft' && event.key !== 'ArrowRight') return;
        event.preventDefault();
        const next = (index + (event.key === 'ArrowRight' ? 1 : -1) + thumbs.length) % thumbs.length;
        select(next);
        thumbs[next].focus();
    });

    if (openButton && modal && modalImage && typeof modal.showModal === 'function') {
        openButton.addEventListener('click', () => {
            modalImage.src = main.src;
            modalImage.alt = main.alt;
            modal.showModal();
        });
        if (closeButton) closeButton.addEventListener('click', () => modal.close());
        // A click on the backdrop lands on the dialog element itself.
        modal.addEventListener('click', (event) => {
            if (event.target === modal) modal.close();
        });
        modal.addEventListener('close', () => {
            modalImage.removeAttribute('src');
            openButton.focus();
        });
    } else if (openButton) {
        openButton.hidden = true;
    }
});
