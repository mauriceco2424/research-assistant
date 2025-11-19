// Lightweight client-side stubs for reports commands.
// The real backend wiring happens in Rust; this file exists so future
// front-end shells can register chat commands without guessing signatures.

export type ReportsCommand = 'reports regenerate' | 'reports configure' | 'reports share';

type CommandHandler = (args: string) => Promise<void> | void;

export interface CommandRegistry {
  register(command: ReportsCommand, handler: CommandHandler): void;
}

export interface ReportsCommandOptions {
  onRegenerate?: CommandHandler;
  onConfigure?: CommandHandler;
  onShare?: CommandHandler;
}

export function registerReportsHandlers(
  registry: CommandRegistry,
  options: ReportsCommandOptions = {}
): void {
  registry.register('reports regenerate', options.onRegenerate ?? defaultHandler('reports regenerate'));
  registry.register('reports configure', options.onConfigure ?? defaultHandler('reports configure'));
  registry.register('reports share', options.onShare ?? defaultHandler('reports share'));
}

function defaultHandler(command: ReportsCommand): CommandHandler {
  return async () => {
    console.warn(`[reports] Handler for "${command}" not implemented yet.`);
  };
}
