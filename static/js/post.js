/**
 * Blog post enhancements. Every piece is optional: without this script the
 * post still reads top to bottom.
 *
 * - Reading progress bar along the top edge.
 * - Table of contents built from the body's h2s, numbered 01, 02..., with the
 *   section in view marked as current.
 * - A bar with a copy button above each code block.
 */
document.addEventListener('DOMContentLoaded', () => {
    const body = document.querySelector('[data-post-body]');
    if (!body) return;

    initProgress(body);
    initToc(body);
    initCodeBlocks(body);
});

function initProgress(body) {
    const bar = document.getElementById('read-progress');
    if (!bar) return;

    let queued = false;
    const update = () => {
        queued = false;
        const rect = body.getBoundingClientRect();
        const total = rect.height - window.innerHeight;
        const done = total > 0 ? Math.min(Math.max(-rect.top / total, 0), 1) : 1;
        bar.style.transform = `scaleX(${done})`;
    };
    const schedule = () => {
        if (!queued) {
            queued = true;
            requestAnimationFrame(update);
        }
    };

    window.addEventListener('scroll', schedule, { passive: true });
    window.addEventListener('resize', schedule);
    update();
}

function slugify(text) {
    return text
        .toLowerCase()
        .normalize('NFD')
        .replace(/[̀-ͯ]/g, '')
        .replace(/[^a-z0-9]+/g, '-')
        .replace(/^-|-$/g, '');
}

function initToc(body) {
    const aside = document.querySelector('[data-toc]');
    const lists = document.querySelectorAll('[data-toc-list]');
    const headings = Array.from(body.querySelectorAll('h2'));
    if (!aside || lists.length === 0 || headings.length < 2) return;

    const used = new Set();
    const entries = headings.map((heading, index) => {
        const number = String(index + 1).padStart(2, '0');
        const title = heading.textContent.trim();

        if (!heading.id) {
            let id = slugify(title) || `section-${number}`;
            while (used.has(id) || document.getElementById(id)) id += '-x';
            heading.id = id;
        }
        used.add(heading.id);

        const marker = document.createElement('span');
        marker.className = 'h-index';
        marker.setAttribute('aria-hidden', 'true');
        marker.textContent = number;
        heading.prepend(marker);

        return { heading, number, title };
    });

    const links = [];
    lists.forEach((list) => {
        entries.forEach(({ heading, number, title }) => {
            const link = document.createElement('a');
            link.href = `#${heading.id}`;
            const index = document.createElement('span');
            index.textContent = `${number} `;
            link.append(index, title);
            list.appendChild(link);
            links.push({ link, heading });
        });
    });
    aside.hidden = false;

    // The current section is the last heading that has scrolled past the
    // upper third of the viewport.
    let queued = false;
    const update = () => {
        queued = false;
        const line = window.innerHeight / 3;
        let current = entries[0].heading;
        entries.forEach(({ heading }) => {
            if (heading.getBoundingClientRect().top <= line) current = heading;
        });
        links.forEach(({ link, heading }) => {
            if (heading === current) link.setAttribute('aria-current', 'true');
            else link.removeAttribute('aria-current');
        });
    };
    window.addEventListener('scroll', () => {
        if (!queued) {
            queued = true;
            requestAnimationFrame(update);
        }
    }, { passive: true });
    update();
}

function initCodeBlocks(body) {
    const copyLabel = body.dataset.copy || 'Copy';
    const copiedLabel = body.dataset.copied || 'Copied';

    body.querySelectorAll('pre').forEach((pre) => {
        const code = pre.querySelector('code') || pre;
        const language = Array.from(code.classList)
            .find((name) => name.startsWith('language-'));

        const wrapper = document.createElement('div');
        wrapper.className = 'code-block';
        const bar = document.createElement('div');
        bar.className = 'code-bar';
        const label = document.createElement('span');
        label.textContent = language ? language.replace('language-', '').toUpperCase() : '';

        const button = document.createElement('button');
        button.type = 'button';
        button.className = 'code-copy';
        button.textContent = copyLabel;
        button.addEventListener('click', () => {
            if (!navigator.clipboard) return;
            navigator.clipboard.writeText(code.textContent).then(() => {
                button.textContent = copiedLabel;
                setTimeout(() => { button.textContent = copyLabel; }, 1600);
            });
        });

        bar.append(label, button);
        pre.replaceWith(wrapper);
        wrapper.append(bar, pre);
    });
}
