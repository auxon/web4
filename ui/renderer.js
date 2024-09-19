const { ipcRenderer } = require('electron');
const urlBar = document.getElementById('url-bar');
const goButton = document.getElementById('go-button');
const backButton = document.getElementById('back-button');
const forwardButton = document.getElementById('forward-button');
const refreshButton = document.getElementById('refresh-button');
const content = document.getElementById('content');
const summary = document.getElementById('summary');

let currentWebview = null;

function loadUrl(url) {
    ipcRenderer.send('load-url', url);
}

goButton.addEventListener('click', () => {
    loadUrl(urlBar.value);
});

backButton.addEventListener('click', () => {
    if (currentWebview && currentWebview.canGoBack()) {
        currentWebview.goBack();
    }
});

forwardButton.addEventListener('click', () => {
    if (currentWebview && currentWebview.canGoForward()) {
        currentWebview.goForward();
    }
});

refreshButton.addEventListener('click', () => {
    if (currentWebview) {
        currentWebview.reload();
    }
});

ipcRenderer.on('url-loaded', (event, data) => {
    summary.innerHTML = `
        <h3>Summary:</h3>
        <p>${data.summary}</p>
        <h3>Analysis:</h3>
        <p>${data.analysis}</p>
    `;
    content.innerHTML = '<webview id="webview" src="about:blank" style="width:100%; height:600px;"></webview>';
    currentWebview = document.getElementById('webview');
    currentWebview.addEventListener('dom-ready', () => {
        currentWebview.setZoomFactor(1);
        currentWebview.loadURL(`http://localhost:3030/render?url=${encodeURIComponent(data.url)}`);
    });

    currentWebview.addEventListener('did-navigate', (e) => {
        urlBar.value = e.url;
    });
});