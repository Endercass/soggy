import express, { Request, Response } from "express";

import { spawn } from "node:child_process";

spawn("node", ["index.js"], {
  cwd: "wsProxy/",
  env: {
    PORT: "8001",
  },
  stdio: [process.stdout, process.stderr],
});

const app = express();

app.use("/dist", express.static(__dirname + "/../pkg"));
app.use(express.static(__dirname + "/public"));

app.listen({ port: 3000 }, () => {
  console.log("Listening on port 3000");
});
