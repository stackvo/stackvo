import { spawn } from 'node:child_process';
import { setTimeout as sleep } from 'node:timers/promises';
import { chromium } from '@playwright/test';
import { stage } from '../tests/e2e/stage.js';
const PORT = 4186, ORIGIN = `http://localhost:${PORT}`;
const server = spawn('npx', ['vite','preview','--port',String(PORT),'--strictPort','--host','localhost'],
  { cwd: process.cwd(), stdio: ['ignore','pipe','inherit'] });
for (let i=0;i<200;i++){ try { if ((await fetch(ORIGIN)).ok) break; } catch {} await sleep(200); }
const browser = await chromium.launch();
const ctx = await browser.newContext({ viewport: { width: 1600, height: 1000 } });
const page = await ctx.newPage();
await stage(page, JSON.parse(process.env.STAGE_JSON));
await page.goto(`${ORIGIN}${process.env.ROUTE}`);
await page.waitForTimeout(1800);
const out = await page.evaluate(() => {
  const main = document.querySelector('main') || document.body;
  return [...main.querySelectorAll('button,[role="button"]')].map((b) => ({
    label: (b.getAttribute('aria-label') || b.textContent || '').trim().slice(0, 45),
    disabled: b.disabled === true || b.classList.contains('v-btn--disabled'),
  })).filter((x) => x.label);
});
console.log(JSON.stringify(out, null, 0));
await browser.close(); server.kill();
