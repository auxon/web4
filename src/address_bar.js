document.body.innerHTML = `
    <div style="display: flex; align-items: center; padding: 5px;">
        <button id="back">←</button>
        <button id="forward">→</button>
        <button id="reload">↻</button>
        <input type="text" id="url" style="flex-grow: 1; margin: 0 5px;">
        <button id="go">Go</button>
    </div>
`;

document.getElementById('back').addEventListener('click', () => {
    window.external.invoke('back');
});

document.getElementById('forward').addEventListener('click', () => {
    window.external.invoke('forward');
});

document.getElementById('reload').addEventListener('click', () => {
    window.external.invoke('reload');
});

document.getElementById('go').addEventListener('click', () => {
    const url = document.getElementById('url').value;
    window.external.invoke(`navigate:${url}`);
});

document.getElementById('url').addEventListener('keypress', (e) => {
    if (e.key === 'Enter') {
        const url = document.getElementById('url').value;
        window.external.invoke(`navigate:${url}`);
    }
});