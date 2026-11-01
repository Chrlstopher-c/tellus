import pino from 'pino';

export const rendererLogger = pino({
  name: 'tellus-renderer',
  level: 'info',
  browser: { asObject: true },
});
