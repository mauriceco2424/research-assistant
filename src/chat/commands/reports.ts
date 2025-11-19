// Lightweight client-side stubs for reports commands.
// The real backend wiring happens in Rust; this file exists so future
// front-end shells can register chat commands without guessing signatures.

export type ReportsCommand = 'reports regenerate' | 'reports configure' | 'reports share';

type CommandHandler = (args: string) => Promise<void> | void;

export interface CommandRegistry {
  register(command: ReportsCommand, handler: CommandHandler): void;
}

export interface ReportsRegeneratePayload {
  raw: string;
  scope?: string;
  categories: string[];
  includes: string[];
  flags: Record<string, string | boolean>;
}

type RegenerateHandler = (payload: ReportsRegeneratePayload) => Promise<void> | void;
export interface ReportsConfigurePayload {
  raw: string;
  includeFigures?: boolean;
  visualizations?: string[];
  excludedAssets?: string[];
  consentRefreshDays?: number;
  consentText?: string;
}

type ConfigureHandler = (payload: ReportsConfigurePayload) => Promise<void> | void;
export interface ReportsSharePayload {
  raw: string;
  manifest?: string;
  destination?: string;
  format?: 'zip' | 'directory';
  includeFigures?: boolean;
  includeVisualizations?: boolean;
  overwrite?: boolean;
}

type ShareHandler = (payload: ReportsSharePayload) => Promise<void> | void;

export interface ReportsCommandOptions {
  onRegenerate?: RegenerateHandler;
  onConfigure?: ConfigureHandler;
  onShare?: ShareHandler;
}

export function registerReportsHandlers(
  registry: CommandRegistry,
  options: ReportsCommandOptions = {}
): void {
  registry.register('reports regenerate', async (args) => {
    const payload = parseRegenerateArgs(args ?? '');
    if (options.onRegenerate) {
      return options.onRegenerate(payload);
    }
    defaultRegenerateHandler(payload);
  });
  registry.register('reports configure', async (args) => {
    const payload = parseConfigureArgs(args ?? '');
    if (options.onConfigure) {
      return options.onConfigure(payload);
    }
    defaultConfigureHandler(payload);
  });
  registry.register('reports share', async (args) => {
    const payload = parseShareArgs(args ?? '');
    if (options.onShare) {
      return options.onShare(payload);
    }
    defaultShareHandler(payload);
  });
}

function defaultHandler(command: ReportsCommand): CommandHandler {
  return async () => {
    console.warn(`[reports] Handler for "${command}" not implemented yet.`);
  };
}

function defaultRegenerateHandler(payload: ReportsRegeneratePayload): void {
  console.warn('[reports] regenerate invoked without backend wiring.', payload);
}

function defaultConfigureHandler(payload: ReportsConfigurePayload): void {
  console.warn('[reports] configure invoked without backend wiring.', payload);
}

function defaultShareHandler(payload: ReportsSharePayload): void {
  console.warn('[reports] share invoked without backend wiring.', payload);
}

function parseRegenerateArgs(raw: string): ReportsRegeneratePayload {
  const tokens = tokenize(raw);
  const payload: ReportsRegeneratePayload = {
    raw,
    scope: undefined,
    categories: [],
    includes: [],
    flags: {},
  };
  for (let i = 0; i < tokens.length; i += 1) {
    const token = tokens[i];
    if (!token.startsWith('--')) {
      payload.categories.push(token);
      continue;
    }
    const flag = token.slice(2);
    const next = tokens[i + 1];
    const nextIsFlag = typeof next === 'string' && next.startsWith('--');
    let value: string | boolean | undefined = true;
    if (!nextIsFlag && next !== undefined) {
      value = next.replace(/^['"]|['"]$/g, '');
      i += 1;
    }
    payload.flags[flag] = value;
    if (flag === 'scope' && typeof value === 'string') {
      payload.scope = value;
    } else if (flag === 'category' && typeof value === 'string') {
      payload.categories.push(value);
    } else if (flag === 'include' && typeof value === 'string') {
      payload.includes.push(value);
    }
  }
  return payload;
}

function parseConfigureArgs(raw: string): ReportsConfigurePayload {
  const tokens = tokenize(raw);
  const payload: ReportsConfigurePayload = { raw };
  const visualizations: string[] = [];
  const excluded: string[] = [];
  for (let i = 0; i < tokens.length; i += 1) {
    const token = tokens[i];
    if (!token.startsWith('--')) {
      continue;
    }
    const flag = token.slice(2);
    const next = tokens[i + 1];
    const nextIsFlag = typeof next === 'string' && next.startsWith('--');
    let value: string | undefined;
    if (!nextIsFlag && next !== undefined) {
      value = next.replace(/^['"]|['"]$/g, '');
      i += 1;
    }
    switch (flag) {
      case 'include-figures':
        if (value) {
          payload.includeFigures = value === 'on' || value === 'true' || value === '1';
        } else {
          payload.includeFigures = true;
        }
        break;
      case 'visualizations':
        if (value) {
          visualizations.push(
            ...value
              .split(',')
              .map((item) => item.trim())
              .filter(Boolean)
          );
        }
        break;
      case 'exclude':
        if (value) {
          excluded.push(value);
        }
        break;
      case 'consent':
        if (value) {
          payload.consentText = value;
        }
        break;
      case 'consent-refresh-days':
        if (value) {
          const parsed = Number(value);
          if (!Number.isNaN(parsed) && parsed > 0) {
            payload.consentRefreshDays = parsed;
          }
        }
        break;
      default:
        break;
    }
  }
  if (visualizations.length > 0) {
    payload.visualizations = visualizations;
  }
  if (excluded.length > 0) {
    payload.excludedAssets = excluded;
  }
  return payload;
}

function parseShareArgs(raw: string): ReportsSharePayload {
  const tokens = tokenize(raw);
  const payload: ReportsSharePayload = { raw };
  for (let i = 0; i < tokens.length; i += 1) {
    const token = tokens[i];
    if (!token.startsWith('--')) {
      continue;
    }
    const flag = token.slice(2);
    const next = tokens[i + 1];
    const nextIsFlag = typeof next === 'string' && next.startsWith('--');
    let value: string | undefined;
    if (!nextIsFlag && next !== undefined) {
      value = next.replace(/^['"]|['"]$/g, '');
      i += 1;
    }
    switch (flag) {
      case 'manifest':
        payload.manifest = value;
        break;
      case 'dest':
      case 'destination':
        payload.destination = value;
        break;
      case 'format':
        if (value === 'zip' || value === 'directory') {
          payload.format = value;
        }
        break;
      case 'include-figures':
        payload.includeFigures = parseBool(value);
        break;
      case 'include-visualizations':
        payload.includeVisualizations = parseBool(value);
        break;
      case 'overwrite':
        payload.overwrite = value ? parseBool(value) : true;
        break;
      default:
        break;
    }
  }
  return payload;
}

function parseBool(value?: string): boolean {
  if (!value) {
    return true;
  }
  return ['1', 'true', 'on', 'yes'].includes(value.toLowerCase());
}

function tokenize(input: string): string[] {
  const matches = input.match(/"[^"]*"|'[^']*'|\S+/g);
  if (!matches) {
    return [];
  }
  return matches.map((segment) => segment.replace(/^['"]|['"]$/g, ''));
}
