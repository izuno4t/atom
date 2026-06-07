import fs from "node:fs/promises";
import path from "node:path";
import mammoth from "mammoth";
import TurndownService from "turndown";

if (process.argv.length !== 5) {
  console.error("usage: mammoth-js-runner <input> <output-md> <report-json>");
  process.exit(64);
}

const [, , inputPath, outputPath, reportPath] = process.argv;
await fs.mkdir(path.dirname(outputPath), { recursive: true });
await fs.mkdir(path.dirname(reportPath), { recursive: true });

const htmlResult = await mammoth.convertToHtml({ path: inputPath });
const turndown = new TurndownService();
const markdown = turndown.turndown(htmlResult.value);
await fs.writeFile(outputPath, markdown, "utf8");

const report = {
  tool: "mammoth-js",
  output: outputPath,
  bytes: Buffer.byteLength(markdown, "utf8"),
  messages: htmlResult.messages,
};
await fs.writeFile(reportPath, `${JSON.stringify(report)}\n`, "utf8");
console.log(JSON.stringify({ tool: "mammoth-js", status: "ok", output: outputPath }));
