document.addEventListener('DOMContentLoaded', function () {
    const form = document.getElementById('contact-form');
    form.addEventListener('submit', function (event) {
        event.preventDefault();
        grecaptcha.ready(function () {
            grecaptcha.execute('6LfuI5YrAAAAAOEUv-Xp1Ewo4dhr1TgCrCG_aqa8', { action: 'submit' }).then(function (token) {
                document.getElementById('g-recaptcha-response').value = token;
                form.submit();
            });
        });
    });
});