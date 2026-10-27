// Throwaway spike (Phase 1 step 6): Electron candidate, same content as the other two.
const { app, BrowserWindow, ipcMain } = require('electron');
const { Worker } = require('worker_threads');
const os = require('os');
app.whenReady().then(() => {
  const w = new BrowserWindow({ width: 640, height: 480, webPreferences: { preload: __dirname + '/preload.js' } });
  w.loadFile('index.html');
});
ipcMain.on('ready', () => console.log('READY'));
ipcMain.handle('run_job', () => new Promise((res) => {
  const wk = new Worker('const t=Date.now();let h=0;while(Date.now()-t<5000){for(let i=0;i<1e6;i++)h=(h*31+i)|0;}', { eval: true });
  try { os.setPriority(wk.threadId, 19); } catch (e) {}
  wk.on('exit', res);
}));
if (process.env.UIBENCH_JOB_AT) setTimeout(() => { console.log('JOB start'); const wk = new Worker('const t=Date.now();let h=0;while(Date.now()-t<5000){for(let i=0;i<1e6;i++)h=(h*31+i)|0;}', { eval: true }); wk.on('exit', () => console.log('JOB end')); }, Number(process.env.UIBENCH_JOB_AT) * 1000);
app.on('window-all-closed', () => app.quit());
