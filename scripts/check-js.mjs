import { readdirSync, rmSync } from "node:fs";
import { join } from "node:path";

const roots = [join("tests", "helpers"), join("tests", "node")];
const entrypoints = [join("scripts", "check-js.mjs")];

for (const root of roots) {
    for (const entry of readdirSync(root, { withFileTypes: true })) {
        if (entry.isFile() && entry.name.endsWith(".js")) {
            entrypoints.push(join(root, entry.name));
        }
    }
}

const outdir = join(".tmp", "bun-check");
rmSync(outdir, { recursive: true, force: true });

const result = await Bun.build({
    entrypoints,
    outdir,
    target: "bun",
});

if (!result.success) {
    for (const log of result.logs) {
        console.error(log);
    }
    process.exit(1);
}

rmSync(outdir, { recursive: true, force: true });
