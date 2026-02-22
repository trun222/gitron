import { isTauri } from './transport';
import type { Transport } from './transport';
import { TauriTransport } from './transport-tauri';
import { HttpTransport } from './transport-http';

let _transport: Transport | null = null;

export function getTransport(): Transport {
  if (!_transport) {
    _transport = isTauri() ? new TauriTransport() : new HttpTransport();
  }
  return _transport;
}

export { isTauri } from './transport';
export type { Transport } from './transport';
