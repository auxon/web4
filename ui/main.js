const { app, BrowserWindow, ipcMain } = require('electron');
const path = require('path');
const fetch = require('node-fetch');

function createWindow() {
  const win = new BrowserWindow({
    width: 800,
    height: 600,
    webPreferences: {
      nodeIntegration: true,
      contextIsolation: false,
      webviewTag: true
    }
  });

  win.loadFile('index.html');

  ipcMain.on('load-url', async (event, url) => {
    try {
      const response = await fetch(`http://localhost:3030/load?url=${encodeURIComponent(url)}`);
      const data = await response.json();
      event.reply('url-loaded', data);
    } catch (error) {
      console.error('Error:', error);
      event.reply('url-loaded', { error: error.message });
    }
  });
}

app.whenReady().then(createWindow);

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

app.on('activate', () => {
  if (BrowserWindow.getAllWindows().length === 0) {
    createWindow();
  }
});