/**
 * DS2000 gallery: thumbnails swap the main render, the colour buttons swap the
 * colourway of every view (renders are named <colour>-<view>.webp), and the
 * enlarge button opens the render in a native <dialog> (focus, Escape and the
 * backdrop come for free). Without JS the page shows the default colourway.
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
    const colourGroup = gallery.querySelector('[data-gallery-colours]');
    const colours = Array.from(gallery.querySelectorAll('[data-gallery-colour]'));
    const colourName = gallery.querySelector('[data-gallery-colour-name]');
    const base = gallery.dataset.galleryBase;
    let colour = gallery.dataset.galleryColourCurrent;
    let current = 0;
    if (!main) return;

    const renderFor = (thumb) => `${base}/${colour}-${thumb.dataset.view}.webp`;

    function select(index) {
        const thumb = thumbs[index];
        if (!thumb) return;
        current = index;
        main.src = base && thumb.dataset.view ? renderFor(thumb) : thumb.dataset.src;
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

    function setColour(button) {
        colour = button.dataset.galleryColour;
        colours.forEach((other) => {
            const active = other === button;
            other.setAttribute('aria-pressed', String(active));
            other.classList.toggle('border-lime', active);
            other.classList.toggle('text-fg', active);
            other.classList.toggle('border-line-strong', !active);
            other.classList.toggle('text-muted', !active);
        });
        thumbs.forEach((thumb) => {
            const img = thumb.querySelector('[data-gallery-thumb-img]');
            if (img) img.src = renderFor(thumb);
        });
        if (colourName) colourName.textContent = button.dataset.label;
        select(current);
    }

    if (base && colourGroup && colours.length) {
        colours.forEach((button) => button.addEventListener('click', () => setColour(button)));
        colourGroup.hidden = false;
    }

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
