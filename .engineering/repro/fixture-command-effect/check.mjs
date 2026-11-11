// Serves the generated fixture-mode app, approves the first row of the pending list through its
// confirm, and reports what the page shows before and after: the list's rows and the notices.
// Usage: node check.mjs <www dir> <playwright-core dir> [chrome executable]
import http from "node:http";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";

const [www, pwDir, chromePath] = process.argv.slice(2);
const require = createRequire(path.join(pwDir, "x.js"));
const { chromium } = require("playwright-core");
const mime = { ".js": "text/javascript", ".css": "text/css", ".html": "text/html" };

const server = http.createServer((req, res) => {
  const url = new URL(req.url, "http://x");
  const file = url.pathname.startsWith("/assets/") ? path.join(www, url.pathname) : path.join(www, "index.html");
  res.setHeader("content-type", mime[path.extname(file)] ?? "text/plain");
  res.end(fs.readFileSync(file));
});
await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));

const browser = await chromium.launch(chromePath ? { executablePath: chromePath } : {});
const page = await browser.newPage();
const requests = [];
page.on("request", (request) => {
  if (!request.url().includes("/assets/") && request.resourceType() !== "document") requests.push(request.url());
});
await page.goto(`http://127.0.0.1:${server.address().port}/`);
const list = '[data-ui-path="pages/requests.inbox/sections/list"]';
const rowsShown = () =>
  page.$$eval(`${list} tbody tr`, (trs) => trs.map((tr) => tr.textContent.trim().replace(/\s+/g, " ")));
await page.waitForSelector(`${list} tbody tr`);
await page.waitForTimeout(500);
console.log(`before: rows ${JSON.stringify(await rowsShown())}`);

await page.locator(`${list} tbody tr`).first().getByRole("button", { name: "Approve" }).click();
const dialog = "[role=dialog]";
await page.waitForSelector(dialog);
console.log(`confirm dialog: data-ui-path ${await page.locator(dialog).getAttribute("data-ui-path")}`);
await page.locator(dialog).getByRole("button", { name: "Confirm" }).click();
await page.waitForSelector(".ui-notice");
await page.waitForTimeout(1500);
const notices = await page.$$eval(".ui-notice", (els) => els.map((el) => el.textContent.trim()));
console.log(`after confirm: notices ${JSON.stringify(notices)}`);
console.log(`after confirm: rows ${JSON.stringify(await rowsShown())}`);
console.log(`network requests other than the page and its assets: ${requests.length}`);
await browser.close();
server.close();
