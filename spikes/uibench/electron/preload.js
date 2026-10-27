const { contextBridge, ipcRenderer } = require('electron');
contextBridge.exposeInMainWorld('api', { ready: () => ipcRenderer.send('ready'), runJob: () => ipcRenderer.invoke('run_job') });
