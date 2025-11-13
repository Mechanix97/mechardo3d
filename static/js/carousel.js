document.addEventListener('DOMContentLoaded', () => {
    const carousel = document.getElementById('carousel-items');
    const prevButton = document.getElementById('prev-slide');
    const nextButton = document.getElementById('next-slide');
    const modal = document.getElementById('image-modal');
    const modalImage = document.getElementById('modal-image');
    const closeModalButton = document.getElementById('close-modal');
    const images = document.querySelectorAll('#carousel-items img');
    let currentIndex = 0;
    let hasInteracted = false;
    let autoAdvanceInterval;
    let resetTimeout = null;

    // Function to update carousel
    function updateCarousel() {
        carousel.style.transform = `translateX(-${currentIndex * 100}%)`;
    }

    // Start auto-advance
    function startAutoAdvance() {
        if (!hasInteracted) {
            autoAdvanceInterval = setInterval(() => {
                currentIndex = (currentIndex === carousel.children.length - 1) ? 0 : currentIndex + 1;
                updateCarousel();
            }, 5000);
        }
    }

    // Stop auto-advance
    function stopAutoAdvance() {
        clearInterval(autoAdvanceInterval);
    }

    // Resume auto-advance after inactivity
    function resetInteraction() {
        hasInteracted = false;
        startAutoAdvance();
    }

    // Start auto-advance on load
    startAutoAdvance();

    // Debounce function to handle interactions
    function handleInteraction(direction) {
        hasInteracted = true;
        stopAutoAdvance();

        // Clear any previous timeout
        if (resetTimeout) {
            clearTimeout(resetTimeout);
        }

        // Update index based on direction
        if (direction === 'prev') {
            currentIndex = (currentIndex === 0) ? carousel.children.length - 1 : currentIndex - 1;
        } else if (direction === 'next') {
            currentIndex = (currentIndex === carousel.children.length - 1) ? 0 : currentIndex + 1;
        }
        updateCarousel();

        // Schedule auto-advance restart after 10s of inactivity
        resetTimeout = setTimeout(resetInteraction, 10000);
    }

    // Button navigation
    prevButton.addEventListener('click', () => handleInteraction('prev'));
    nextButton.addEventListener('click', () => handleInteraction('next'));

    // Arrow key navigation
    document.addEventListener('keydown', (e) => {
        if (e.key === 'ArrowLeft' && !modal.classList.contains('hidden')) {
            return; // Avoid navigating with arrows if modal is open
        }
        if (e.key === 'ArrowRight' && !modal.classList.contains('hidden')) {
            return; // Avoid navigating with arrows if modal is open
        }
        if (e.key === 'ArrowLeft') {
            handleInteraction('prev');
        } else if (e.key === 'ArrowRight') {
            handleInteraction('next');
        }
    });

    // Modal
    images.forEach(img => {
        img.addEventListener('click', () => {
            modalImage.src = img.dataset.fullscreen;
            modal.classList.remove('hidden');
            stopAutoAdvance(); // Pause auto-advance when opening modal
        });
    });

    closeModalButton.addEventListener('click', () => {
        modal.classList.add('hidden');
        if (!hasInteracted) {
            startAutoAdvance(); // Resume auto-advance when closing modal if no interaction
        }
    });

    // Close modal when clicking outside the image
    modal.addEventListener('click', (e) => {
        if (e.target === modal) {
            modal.classList.add('hidden');
            if (!hasInteracted) {
                startAutoAdvance(); // Resume auto-advance when closing modal if no interaction
            }
        }
    });

    // Close modal with Esc key
    document.addEventListener('keydown', (e) => {
        if (e.key === 'Escape' && !modal.classList.contains('hidden')) {
            modal.classList.add('hidden');
            if (!hasInteracted) {
                startAutoAdvance(); // Resume auto-advance when closing modal if no interaction
            }
        }
    });
});
