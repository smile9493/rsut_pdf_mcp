/** Worker communication protocol — Transferable enforcement */

export interface WorkerMessage<T = unknown> {
  type: 'request'
  id: number
  payload: T
  buffer?: ArrayBuffer
}

export interface WorkerResponse<T = unknown> {
  type: 'response'
  id: number
  payload: T
  buffer?: ArrayBuffer
}

/** Main thread → Worker: zero-copy transfer */
export function postToWorker<T>(
  worker: Worker,
  message: WorkerMessage<T>,
  transferables?: ArrayBuffer[]
): void {
  worker.postMessage(message, transferables ?? [])
}

/** Worker → Main thread: zero-copy transfer */
export function postToMain<T>(
  self: Worker,
  message: WorkerResponse<T>,
  transferables?: ArrayBuffer[]
): void {
  self.postMessage(message, transferables ?? [])
}
