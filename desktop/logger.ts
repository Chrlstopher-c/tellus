import { join } from 'node:path';
import pino from 'pino';
import pretty from 'pino-pretty';

const LOG_FILE = join(__dirname, '../../logs/app.log');
const LEVEL = process.env['TELLUS_LOG_LEVEL'] ?? 'info';

export const logger = pino(
  { name: 'tellus-main', level: LEVEL },
  pino.multistream([
    { level: LEVEL, stream: pretty({ colorize: true }) },
    { level: LEVEL, stream: pino.destination({ dest: LOG_FILE, append: false, mkdir: true, sync: true }) },
  ]),
);
