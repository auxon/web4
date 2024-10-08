// Function to initialize the page
function initializePage(initialUrl) {
    console.log("Initializing page...");
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', () => setupPage(initialUrl));
    } else {
        setupPage(initialUrl);
    }
}

function setupPage(initialUrl) {
    console.log("Setting up page...");
    console.log("Document ready state:", document.readyState);
    console.log("URL input element:", document.getElementById('url-input'));
    console.log("Go button element:", document.getElementById('go-button'));

    // Ensure the browser chrome is visible
    document.getElementById('browser-chrome').style.display = 'block';

    // Address bar functionality
    setupEventListeners();

    // Load initial URL if provided
    if (initialUrl && initialUrl !== "") {
        loadUrl(initialUrl);
    }

    // Update address bar when page loads
    updateAddressBar();

    // Notify Rust when the page URL changes
    let lastUrl = window.location.href;
    setInterval(() => {
        if (window.location.href !== lastUrl) {
            lastUrl = window.location.href;
            updateAddressBar();
            window.ipc.postMessage('loadUrl:' + lastUrl);
        }
    }, 100);
}

function setupEventListeners() {
    const urlInput = document.getElementById('url-input');
    const goButton = document.getElementById('go-button');
    const newTabButton = document.getElementById('new-tab-button');

    if (urlInput && goButton && newTabButton) {
        urlInput.addEventListener('keydown', (event) => {
            if (event.key === 'Enter') {
                event.preventDefault();
                loadUrl(urlInput.value);
            }
        });

        goButton.addEventListener('click', (event) => {
            event.preventDefault();
            loadUrl(urlInput.value);
        });

        newTabButton.addEventListener('click', (event) => {
            event.preventDefault();
            createNewTab();
        });
    } else {
        console.error('URL input, Go button, or New Tab button not found');
    }

    // Handle clicks on the entire document
    document.addEventListener('click', (event) => {
        let target = event.target;
        
        // Handle link clicks
        if (target.tagName === 'A') {
            event.preventDefault();
            loadUrl(target.href);
        }
        
        // Handle button and submit input clicks
        if (target.tagName === 'BUTTON' || (target.tagName === 'INPUT' && target.type === 'submit')) {
            const form = target.closest('form');
            if (form) {
                event.preventDefault();
                const formData = new FormData(form);
                const searchParams = new URLSearchParams(formData);
                const fullUrl = new URL(form.action);
                fullUrl.search = searchParams.toString();
                loadUrl(fullUrl.toString());
            }
        }
    });

    // Handle form submissions
    document.querySelectorAll('form').forEach(form => {
        form.addEventListener('submit', (e) => {
            e.preventDefault();
            const formData = new FormData(form);
            const searchParams = new URLSearchParams(formData);
            const fullUrl = new URL(form.action);
            fullUrl.search = searchParams.toString();
            loadUrl(fullUrl.toString());
        });
    });

    // Handle clicks on buttons within forms
    document.addEventListener('click', (event) => {
        const target = event.target;
        if (target.tagName === 'INPUT' && (target.type === 'submit' || target.type === 'button')) {
            const form = target.closest('form');
            if (form) {
                event.preventDefault();
                const formData = new FormData(form);
                formData.append(target.name, target.value);
                const searchParams = new URLSearchParams(formData);
                const fullUrl = new URL(form.action);
                fullUrl.search = searchParams.toString();
                loadUrl(fullUrl.toString());
            }
        }
    });
}

// Function to load URL
let lastLoadedUrl = '';

function loadUrl(url) {
    console.log("Loading URL:", url);
    if (!url.startsWith('http://') && !url.startsWith('https://')) {
        url = 'https://' + url;
    }
    
    if (url === lastLoadedUrl) {
        console.log("URL already loaded, skipping:", url);
        return Promise.resolve();
    }
    
    lastLoadedUrl = url;
    
    return fetch('http://localhost:3030/load?url=' + encodeURIComponent(url))
        .then(response => response.json())
        .then(data => {
            const contentArea = document.getElementById('content-area');
            if (data.error) {
                contentArea.innerHTML = `<p>Error loading content: ${data.error}</p>`;
            } else {
                contentArea.innerHTML = data.content || '';
                document.getElementById('url-input').value = data.url || url;
                console.log('Content loaded:', data.content ? data.content.substring(0, 100) + '...' : 'No content');
                
                const baseUrl = new URL(data.url);
                contentArea.querySelectorAll('*').forEach(el => {
                    ['src', 'href', 'action'].forEach(attr => {
                        if (el.hasAttribute(attr)) {
                            try {
                                let attrValue = el.getAttribute(attr);
                                if (attrValue.startsWith('//')) {
                                    attrValue = 'https:' + attrValue;
                                }
                                el.setAttribute(attr, new URL(attrValue, baseUrl).href);
                            } catch (e) {
                                console.error('Error updating attribute:', attr, 'for element:', el, 'Error:', e);
                            }
                        }
                    });
                });

                // Handle stylesheets
                const stylesheets = contentArea.querySelectorAll('link[rel="stylesheet"]');
                stylesheets.forEach(stylesheet => {
                    const href = stylesheet.getAttribute('href');
                    if (href) {
                        fetch(new URL(href, baseUrl).href)
                            .then(response => response.text())
                            .then(css => {
                                const style = document.createElement('style');
                                style.textContent = css;
                                stylesheet.parentNode.replaceChild(style, stylesheet);
                            })
                            .catch(error => console.error('Error loading stylesheet:', error));
                    }
                });
            }

            setupEventListeners();
        })
        .catch(error => {
            console.error('Error:', error);
            document.getElementById('content-area').innerHTML = '<p>Error loading content: ' + error.message + '</p>';
        });
}

// Function to switch tabs
function switchTab(tabIndex) {
    console.log("Switching to tab:", tabIndex);
    window.ipc.postMessage(`switchTab:${tabIndex}`);
}

// Function to create a new tab
function createNewTab() {
    console.log("Creating new tab");
    window.ipc.postMessage('createNewTab');
}

// Function to update the address bar
function updateAddressBar() {
    const urlInput = document.getElementById('url-input');
    if (urlInput) {
        urlInput.value = window.location.href;
    }
}

// Function to update the tab UI
function updateTabUI(url) {
    document.getElementById('url-input').value = url;
    document.getElementById('browser-chrome').style.display = 'block';
}

// Initialize the page
// Note: We're not calling initializePage() here anymore, it will be called from Rust with the initial URL

// Expose functions to the global scope
window.loadUrl = loadUrl;
window.switchTab = switchTab;
window.createNewTab = createNewTab;
window.setupEventListeners = setupEventListeners;
window.updateTabUI = updateTabUI;