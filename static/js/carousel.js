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

    // Función para actualizar el carrusel
    function updateCarousel() {
        carousel.style.transform = `translateX(-${currentIndex * 100}%)`;
    }

    // Iniciar autoavance
    function startAutoAdvance() {
        if (!hasInteracted) {
            autoAdvanceInterval = setInterval(() => {
                currentIndex = (currentIndex === carousel.children.length - 1) ? 0 : currentIndex + 1;
                updateCarousel();
            }, 5000);
        }
    }

    // Detener autoavance
    function stopAutoAdvance() {
        clearInterval(autoAdvanceInterval);
    }

    // Reanudar autoavance tras inactividad
    function resetInteraction() {
        hasInteracted = false;
        startAutoAdvance();
    }

    // Iniciar autoavance al cargar
    startAutoAdvance();

    // Función de debounce para manejar interacciones
    function handleInteraction(direction) {
        hasInteracted = true;
        stopAutoAdvance();

        // Limpiar cualquier timeout anterior
        if (resetTimeout) {
            clearTimeout(resetTimeout);
        }

        // Actualizar índice según dirección
        if (direction === 'prev') {
            currentIndex = (currentIndex === 0) ? carousel.children.length - 1 : currentIndex - 1;
        } else if (direction === 'next') {
            currentIndex = (currentIndex === carousel.children.length - 1) ? 0 : currentIndex + 1;
        }
        updateCarousel();

        // Programar reinicio del autoavance tras 10s de inactividad
        resetTimeout = setTimeout(resetInteraction, 10000);
    }

    // Navegación con botones
    prevButton.addEventListener('click', () => handleInteraction('prev'));
    nextButton.addEventListener('click', () => handleInteraction('next'));

    // Navegación con teclas de flecha
    document.addEventListener('keydown', (e) => {
        if (e.key === 'ArrowLeft' && !modal.classList.contains('hidden')) {
            return; // Evitar navegar con flechas si el modal está abierto
        }
        if (e.key === 'ArrowRight' && !modal.classList.contains('hidden')) {
            return; // Evitar navegar con flechas si el modal está abierto
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
            stopAutoAdvance(); // Pausar autoavance al abrir modal
        });
    });

    closeModalButton.addEventListener('click', () => {
        modal.classList.add('hidden');
        if (!hasInteracted) {
            startAutoAdvance(); // Reanudar autoavance al cerrar modal si no hubo interacción
        }
    });

    // Cerrar modal al hacer clic fuera de la imagen
    modal.addEventListener('click', (e) => {
        if (e.target === modal) {
            modal.classList.add('hidden');
            if (!hasInteracted) {
                startAutoAdvance(); // Reanudar autoavance al cerrar modal si no hubo interacción
            }
        }
    });

    // Cerrar modal con tecla Esc
    document.addEventListener('keydown', (e) => {
        if (e.key === 'Escape' && !modal.classList.contains('hidden')) {
            modal.classList.add('hidden');
            if (!hasInteracted) {
                startAutoAdvance(); // Reanudar autoavance al cerrar modal si no hubo interacción
            }
        }
    });
});
