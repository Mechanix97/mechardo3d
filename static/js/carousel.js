document.addEventListener('DOMContentLoaded', () => {
    const carousel = document.getElementById('carousel');
    const track = document.getElementById('carousel-items');
    const prevButton = document.getElementById('prev-slide');
    const nextButton = document.getElementById('next-slide');
    const modal = document.getElementById('image-modal');
    const modalImage = document.getElementById('modal-image');
    const closeModalButton = document.getElementById('close-modal');
    if (!carousel || !track || !prevButton || !nextButton) return;

    const triggers = Array.from(track.querySelectorAll('[data-fullscreen]'));
    const slideCount = track.children.length;
    const reduceMotion = window.matchMedia('(prefers-reduced-motion: reduce)');

    let currentIndex = 0;
    let autoAdvanceInterval = null;
    let paused = false;
    let lastFocused = null;

    const render = () => {
        track.style.transform = `translateX(-${currentIndex * 100}%)`;
    };

    const stopAutoAdvance = () => {
        clearInterval(autoAdvanceInterval);
        autoAdvanceInterval = null;
    };

    const startAutoAdvance = () => {
        // Never stack intervals, and honour a reduced-motion preference.
        stopAutoAdvance();
        if (paused || reduceMotion.matches) return;
        autoAdvanceInterval = setInterval(() => go(1), 5000);
    };

    function go(step) {
        currentIndex = (currentIndex + step + slideCount) % slideCount;
        render();
    }

    function navigate(step) {
        go(step);
        // Manual navigation wins until the visitor leaves the carousel alone.
        startAutoAdvance();
    }

    prevButton.addEventListener('click', () => navigate(-1));
    nextButton.addEventListener('click', () => navigate(1));

    // Arrow keys only apply while the carousel itself has focus, instead of
    // hijacking them for the whole page.
    carousel.addEventListener('keydown', (e) => {
        if (e.key === 'ArrowLeft') { e.preventDefault(); navigate(-1); }
        else if (e.key === 'ArrowRight') { e.preventDefault(); navigate(1); }
    });

    // Pause while the visitor is reading or tabbing through it.
    const pause = () => { paused = true; stopAutoAdvance(); };
    const resume = () => { paused = false; startAutoAdvance(); };
    carousel.addEventListener('mouseenter', pause);
    carousel.addEventListener('mouseleave', resume);
    carousel.addEventListener('focusin', pause);
    carousel.addEventListener('focusout', (e) => {
        if (!carousel.contains(e.relatedTarget)) resume();
    });
    reduceMotion.addEventListener('change', startAutoAdvance);

    if (modal && modalImage && closeModalButton) {
        const openModal = (trigger) => {
            const src = trigger.dataset.fullscreen;
            if (!src) return;
            lastFocused = trigger;
            modalImage.src = src;
            const image = trigger.querySelector('img');
            if (image) modalImage.alt = image.alt;
            modal.classList.remove('hidden');
            pause();
            closeModalButton.focus();
        };

        const closeModal = () => {
            modal.classList.add('hidden');
            modalImage.removeAttribute('src');
            resume();
            if (lastFocused) lastFocused.focus();
        };

        triggers.forEach(trigger => trigger.addEventListener('click', () => openModal(trigger)));
        closeModalButton.addEventListener('click', closeModal);
        modal.addEventListener('click', (e) => { if (e.target === modal) closeModal(); });

        // Keep focus inside the dialog while it is open.
        modal.addEventListener('keydown', (e) => {
            if (e.key === 'Escape') { e.preventDefault(); closeModal(); }
            else if (e.key === 'Tab') { e.preventDefault(); closeModalButton.focus(); }
        });
    }

    render();
    startAutoAdvance();
});
