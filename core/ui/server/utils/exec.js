import { exec } from 'child_process';
import { promisify } from 'util';

/**
 * Promisified exec function
 */
export const execAsync = promisify(exec);
