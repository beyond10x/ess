// Serves react-out/www and answers the view route with a chosen status, then reads what the
// generated page shows in its header metrics and in its list section.
// Usage: node check.mjs <www dir> <playwright-core dir> [chrome executable]
import http from "node:http";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";

const [www, pwDir, chromePath] = process.argv.slice(2);
const require = createRequire(path.join(pwDir, "x.js"));
const { chromium } = require("playwright-core");

const rows = [
  { id: "a", title: "lamp", qty: 5, state: "Stored" },
  { id: "b", title: "chair", qty: 90, state: "Shipped" },
];
const mime = { ".js": "text/javascript", ".css": "text/css", ".html": "text/html" };

function serve(status) {
  return new Promise((resolve) => {
    const server = http.createServer((req, res) => {
      const url = new URL(req.url, "http://x");
      if (url.pathname === "/shelf/views/Items") {
        res.statusCode = status;
        res.setHeader("content-type", "application/json");
        res.end(status === 200 ? JSON.stringify({ rows }) : JSON.stringify({ error: `status ${status}` }));
        return;
      }
      const file = url.pathname.startsWith("/assets/") ? path.join(www, url.pathname) : path.join(www, "index.html");
      res.setHeader("content-type", mime[path.extname(file)] ?? "text/plain");
      res.end(fs.readFileSync(file));
    });
    server.listen(0, "127.0.0.1", () => resolve(server));
  });
}

const browser = await chromium.launch(chromePath ? { executablePath: chromePath } : {});
for (const status of [200, 501, 404, 500]) {
  const server = await serve(status);
  const page = await browser.newPage();
  await page.goto(`http://127.0.0.1:${server.address().port}/`);
  await page.waitForSelector(".ui-metric-value");
  await page.waitForTimeout(1500);
  const metrics = await page.$$eval(".ui-metric", (els) =>
    els.map((el) => `${el.querySelector(".ui-metric-label")?.textContent}=${el.querySelector(".ui-metric-value")?.textContent}`),
  );
  const alerts = await page.$$eval("[role=alert]", (els) => els.map((el) => el.textContent.trim().replace(/\s+/g, " ")));
  console.log(`view answers ${status}: header metrics ${JSON.stringify(metrics)}; list section alerts ${JSON.stringify(alerts)}`);
  await page.close();
  server.close();
}
await browser.close();
