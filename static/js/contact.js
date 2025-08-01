document.addEventListener('DOMContentLoaded', function () {
    console.log('contact.js loaded');
    const form = document.getElementById('contact-form');
    const errorAlert = document.getElementById('error-alert');
    const errorMessage = document.getElementById('error-message');

    // Check if elements exist
    if (!form) {
        console.error('Form with id="contact-form" not found');
        return;
    }
    if (!errorAlert || !errorMessage) {
        console.error('Error alert elements not found: error-alert or error-message');
        return;
    }

    form.addEventListener('submit', function (event) {
        event.preventDefault();
        console.log('Form submit event triggered');

        // Hide any existing error message
        errorAlert.classList.add('hidden');

        grecaptcha.ready(function () {
            console.log('reCAPTCHA ready');
            grecaptcha.execute('6LfuI5YrAAAAAOEUv-Xp1Ewo4dhr1TgCrCG_aqa8', { action: 'submit' }).then(function (token) {
                console.log('reCAPTCHA token obtained:', token);
                document.getElementById('g-recaptcha-response').value = token;

                // Convert FormData to URL-encoded string
                const formData = new FormData(form);
                const urlEncodedData = new URLSearchParams(formData).toString();

                // Submit form via fetch
                fetch('/contact', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/x-www-form-urlencoded',
                    },
                    body: urlEncodedData,
                })
                    .then(response => {
                        console.log('Fetch response received:', response.status);
                        if (response.ok) {
                            console.log('Form submission successful, redirecting to /contact_success');
                            window.location.href = '/contact_success';
                        } else {
                            return response.json().then(data => {
                                console.log('Error response data:', data);
                                if (data.error) {
                                    errorMessage.textContent = data.error;
                                    errorAlert.classList.remove('hidden');
                                } else {
                                    console.error('No error message in response');
                                    errorMessage.textContent = 'Ocurrió un error desconocido. Probá de nuevo.';
                                    errorAlert.classList.remove('hidden');
                                }
                            }).catch(err => {
                                console.error('Failed to parse JSON response:', err);
                                errorMessage.textContent = 'Error al enviar el formulario. Probá de nuevo.';
                                errorAlert.classList.remove('hidden');
                            });
                        }
                    })
                    .catch(error => {
                        console.error('Fetch error:', error);
                        errorMessage.textContent = 'Error al enviar el formulario. Probá de nuevo.';
                        errorAlert.classList.remove('hidden');
                    });
            }).catch(error => {
                console.error('reCAPTCHA error:', error);
                errorMessage.textContent = 'Error con reCAPTCHA. Probá de nuevo.';
                errorAlert.classList.remove('hidden');
            });
        });
    });
});
