/// <reference lib="dom" />
import { contextBridge, ipcRenderer, type IpcRendererEvent } from 'electron';
import { SIM_PORT_CHANNEL } from '../interface/src/sim-bridge/protocol';

// Un MessagePort ne traverse pas contextBridge (il arrive en objet vide) : la page le reçoit par window.postMessage.
const simPort = new Promise<MessagePort>((resolve) => {
  ipcRenderer.once(SIM_PORT_CHANNEL, (event: IpcRendererEvent) => {
    const [port] = event.ports;
    if (port) resolve(port);
  });
});

contextBridge.exposeInMainWorld('tellus', { platform: process.platform });
contextBridge.exposeInMainWorld('tellusSim', {
  requestPort: (): void => {
    void simPort.then((port) => window.postMessage(SIM_PORT_CHANNEL, '*', [port]));
  },
});
