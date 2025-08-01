const carousel = document.getElementById('carousel-items');
const prevButton = document.getElementById('prev-slide');
const nextButton = document.getElementById('next-slide');
const modal = document.getElementById('image-modal');
const modalImage = document.getElementById('modal-image');
const closeModalButton = document.getElementById('close-modal');
const images = document.querySelectorAll('#carousel-items img');
let currentIndex = 0;

// Carrusel
function updateCarousel() {
    carousel.style.transform = `translateX(-${currentIndex * 100}%)`;
}

prevButton.addEventListener('click', () => {
    currentIndex = (currentIndex === 0) ? carousel.children.length - 1 : currentIndex - 1;
    updateCarousel();
});

nextButton.addEventListener('click', () => {
    currentIndex = (currentIndex === carousel.children.length - 1) ? 0 : currentIndex + 1;
    updateCarousel();
});

// Modal
images.forEach(img => {
    img.addEventListener('click', () => {
        modalImage.src = img.dataset.fullscreen;
        modal.classList.remove('hidden');
    });
});

closeModalButton.addEventListener('click', () => {
    modal.classList.add('hidden');
});

// Cerrar modal al hacer clic fuera de la imagen
modal.addEventListener('click', (e) => {
    if (e.target === modal) {
        modal.classList.add('hidden');
    }
});

// Cerrar modal con tecla Esc
document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape' && !modal.classList.contains('hidden')) {
        modal.classList.add('hidden');
    }
});

// Autoavance opcional (descomentar para activar)
// setInterval(() => {
//     currentIndex = (currentIndex === carousel.children.length - 1) ? 0 : currentIndex + 1;
//     updateCarousel();
// }, 5000);
