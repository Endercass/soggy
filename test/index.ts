import express from "express";
import http from "http";
const app = express();
import { dirname } from "path";
import { fileURLToPath } from "url";
import { cli_logger, ModuleLoader, Server } from "wsproxy-ng";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

app.use(express.static(__dirname + "/../public"));

app.use("/dist", express.static(__dirname + "/../pkg"));
app.use(express.static(__dirname + "/public"));

let server = http.createServer(app);

const wsProxyConfig = {
  port: 3000,
  logger: cli_logger,
  modules: new ModuleLoader(`${__dirname}/../modules`),
  server,
};

let wsProxy = new Server(wsProxyConfig);
wsProxy.listen();
