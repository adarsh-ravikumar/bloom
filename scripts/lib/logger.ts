import readline from "node:readline";
import type { ChildProcess } from "node:child_process";

export enum LogLevel {
  Debug,
  Info,
  Warn,
  Error,
}

export enum LogColor {
  Gray,
  Blue,
  Green,
  Yellow,
  Red,
  Magenta,
  Cyan,
}

const RESET = "\x1b[0m";

function ansi(color: LogColor): string {
  switch (color) {
    case LogColor.Gray: return "\x1b[90m";
    case LogColor.Blue: return "\x1b[94m";
    case LogColor.Green: return "\x1b[92m";
    case LogColor.Yellow: return "\x1b[93m";
    case LogColor.Red: return "\x1b[91m";
    case LogColor.Magenta: return "\x1b[95m";
    case LogColor.Cyan: return "\x1b[96m";
  }
}

export class LogTask implements Disposable {
  private readonly started = performance.now();
  private finished = false;

  private didFail = false;

  constructor(
    private readonly channel: LogChannel,
    private readonly title: string,
  ) {
    this.channel.info(`⏳ ${title}`);
  }

  public complete(message = this.title) {
    if (this.finished) return;

    this.finished = true;

    const elapsed = performance.now() - this.started;

    this.channel.info(
      `✓ ${message} ${ansi(LogColor.Gray)}(${elapsed.toFixed(0)} ms)${RESET}`
    );
  }

  public fail(error?: unknown) {
    if (this.finished) return;

    this.didFail = true;
    this.finished = true;

    this.channel.error(`✗ ${this.title}`);

    if (error instanceof Error)
      this.channel.error(error.message);
  }

  [Symbol.dispose]() {
    if (!this.didFail)
      this.complete();
  }
}

export class LogChannel {
  private level = LogLevel.Info;
  private indent = 0;

  constructor(
    public readonly name: string,
    public readonly color: LogColor,
  ) { }

  private write(
    level: LogLevel,
    writer: (...args: any[]) => void,
    message: string,
  ) {
    if (level < this.level)
      return;

    writer(
      `${ansi(this.color)}[${this.name}]${RESET} ${"  ".repeat(this.indent)}${message}`
    );
  }

  public debug(message: string) {
    this.write(LogLevel.Debug, console.debug, message);
  }

  public info(message: string) {
    this.write(LogLevel.Info, console.log, message);
  }

  public warn(message: string) {
    this.write(LogLevel.Warn, console.warn, message);
  }

  public error(message: string) {
    this.write(LogLevel.Error, console.error, message);
  }

  public task(title: string) {
    return new LogTask(this, title);
  }

  public timer(title: string) {
    return this.task(title);
  }

  public push() {
    this.indent++;
  }

  public pop() {
    this.indent = Math.max(0, this.indent - 1);
  }

  public setLevel(level: LogLevel) {
    this.level = level;
  }

  public raw(message: string) {
    console.log(message);
  }

  public attach(
    process: ChildProcess,
    options: {
      stdout?: (line: string) => boolean;
      stderr?: (line: string) => boolean;
    } = {},
  ) {
    if (process.stdout) {
      const rl = readline.createInterface({
        input: process.stdout,
      });

      rl.on("line", (line) => {
        if (options.stdout?.(line) ?? true)
          this.info(line);
      });
    }

    if (process.stderr) {
      const rl = readline.createInterface({
        input: process.stderr,
      });

      rl.on("line", (line) => {
        if (options.stderr?.(line) ?? true)
          this.error(line);
      });
    }
  }
}

export class Logger {
  private readonly channels =
    new Map<string, LogChannel>();

  public channel(
    name: string,
    color: LogColor = LogColor.Gray,
  ) {
    let channel = this.channels.get(name);

    if (!channel) {
      channel = new LogChannel(name, color);
      this.channels.set(name, channel);
    }

    return channel;
  }

  get bloom() {
    return this.channel("Bloom", LogColor.Magenta);
  }

  get vite() {
    return this.channel("Vite", LogColor.Green);
  }

  get tauri() {
    return this.channel("Tauri", LogColor.Cyan);
  }

  get rust() {
    return this.channel("Rust", LogColor.Yellow);
  }

  get sync() {
    return this.channel("Sync", LogColor.Blue);
  }

  get schema() {
    return this.channel("Schema", LogColor.Blue);
  }

  get widget() {
    return this.channel("Widget", LogColor.Blue);
  }
}
