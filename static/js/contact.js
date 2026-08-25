document.addEventListener('DOMContentLoaded', function () {
    const form = document.getElementById('contact-form');
    const errorAlert = document.getElementById('error-alert');
    const errorMessage = document.getElementById('error-message');

    if (!form) {
        console.error('Form with id="contact-form" not found');
        return;
    }
    if (!errorAlert || !errorMessage) {
        console.error('Error alert elements not found: error-alert or error-message');
        return;
    }

    const submitUrl = form.getAttribute('action') || '/contact';
    const successUrl = form.dataset.successUrl || '/contact_success';

    function showError(message) {
        errorMessage.textContent = message;
        errorAlert.classList.remove('hidden');
    }

    form.addEventListener('submit', function (event) {
        event.preventDefault();
        errorAlert.classList.add('hidden');

        grecaptcha.ready(function () {
            grecaptcha.execute(window.recaptchaSiteKey, { action: 'submit' }).then(function (token) {
                document.getElementById('g-recaptcha-response').value = token;

                const urlEncodedData = new URLSearchParams(new FormData(form)).toString();

                fetch(submitUrl, {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/x-www-form-urlencoded',
                    },
                    body: urlEncodedData,
                })
                    .then(response => {
                        if (response.ok) {
                            window.location.href = response.redirected ? response.url : successUrl;
                            return;
                        }

                        return response.json()
                            .then(data => showError(data.error || window.contactTranslations.unknownError))
                            .catch(() => showError(window.contactTranslations.submitError));
                    })
                    .catch(error => {
                        console.error('Fetch error:', error);
                        showError(window.contactTranslations.submitError);
                    });
            }).catch(error => {
                console.error('reCAPTCHA error:', error);
                showError(window.contactTranslations.recaptchaError);
            });
        });
    });
});
