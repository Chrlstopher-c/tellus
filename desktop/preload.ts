import { contextBridge } from 'electron';

contextBridge.exposeInMainWorld('tellus', { platform: process.platform });
