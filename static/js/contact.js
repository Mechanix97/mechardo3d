document.addEventListener('DOMContentLoaded', function () {
    const form = document.getElementById('contact-form');
    const errorAlert = document.getElementById('error-alert');
    const errorMessage = document.getElementById('error-message');
    const submitButton = document.getElementById('contact-submit');

    if (!form || !errorAlert || !errorMessage) {
        console.error('Contact form markup is incomplete');
        return;
    }

    // Configuration and copy come from data attributes, so translations never
    // travel through a JavaScript string literal.
    const config = form.dataset;
    const submitUrl = form.getAttribute('action') || '/contact';
    const submitLabel = submitButton ? submitButton.textContent : '';

    function showError(message) {
        errorMessage.textContent = message;
        errorAlert.classList.remove('hidden');
    }

    function setSending(sending) {
        if (!submitButton) return;
        submitButton.disabled = sending;
        submitButton.textContent = sending && config.sending ? config.sending : submitLabel;
    }

    form.addEventListener('submit', function (event) {
        event.preventDefault();
        errorAlert.classList.add('hidden');
        if (submitButton && submitButton.disabled) return;
        setSending(true);

        grecaptcha.ready(function () {
            grecaptcha.execute(config.recaptchaKey, { action: 'submit' }).then(function (token) {
                document.getElementById('g-recaptcha-response').value = token;

                return fetch(submitUrl, {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
                    body: new URLSearchParams(new FormData(form)).toString(),
                }).then(response => {
                    if (response.ok) {
                        window.location.href = response.redirected
                            ? response.url
                            : (config.successUrl || '/contact_success');
                        return;
                    }

                    setSending(false);
                    return response.json()
                        .then(data => showError(data.error || config.errorUnknown))
                        .catch(() => showError(config.errorSubmit));
                }).catch(error => {
                    // Network failure: distinct from a reCAPTCHA failure below.
                    console.error('Submit failed:', error);
                    setSending(false);
                    showError(config.errorSubmit);
                });
            }).catch(error => {
                console.error('reCAPTCHA error:', error);
                setSending(false);
                showError(config.errorRecaptcha);
            });
        });
    });
});
