/* tslint:disable */
/* eslint-disable */

export class SamplerEngine {
    free(): void;
    [Symbol.dispose](): void;
    create_audio_worklet_node(ctx: AudioContext): AudioWorkletNode;
    constructor();
    set_amplitude(value: number): void;
    set_frequency_hz(value: number): void;
    readonly amplitude: number;
    readonly frequency_hz: number;
}

export class SineWorkletNode {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    disconnect(): void;
    static new(): SineWorkletNode;
    pack(): number;
    process(buf: Float32Array): boolean;
    set_amplitude_inner(value: number): void;
    set_frequency_hz_inner(value: number): void;
    static unpack(val: number): SineWorkletNode;
}

export function create_worklet_debug_info(): any;

export function create_worklet_module_url(): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly __wbg_samplerengine_free: (a: number, b: number) => void;
    readonly __wbg_sineworkletnode_free: (a: number, b: number) => void;
    readonly create_worklet_module_url: () => [number, number];
    readonly samplerengine_amplitude: (a: number) => number;
    readonly samplerengine_create_audio_worklet_node: (a: number, b: any) => [number, number, number];
    readonly samplerengine_frequency_hz: (a: number) => number;
    readonly samplerengine_new: () => number;
    readonly samplerengine_set_amplitude: (a: number, b: number) => [number, number];
    readonly samplerengine_set_frequency_hz: (a: number, b: number) => [number, number];
    readonly sineworkletnode_disconnect: (a: number) => [number, number];
    readonly sineworkletnode_new: () => number;
    readonly sineworkletnode_pack: (a: number) => number;
    readonly sineworkletnode_process: (a: number, b: number, c: number, d: any) => number;
    readonly sineworkletnode_set_amplitude_inner: (a: number, b: number) => void;
    readonly sineworkletnode_set_frequency_hz_inner: (a: number, b: number) => void;
    readonly sineworkletnode_unpack: (a: number) => number;
    readonly create_worklet_debug_info: () => any;
    readonly memory: WebAssembly.Memory;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_thread_destroy: (a?: number, b?: number, c?: number) => void;
    readonly __wbindgen_start: (a: number) => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput, memory?: WebAssembly.Memory, thread_stack_size?: number }} module - Passing `SyncInitInput` directly is deprecated.
 * @param {WebAssembly.Memory} memory - Deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput, memory?: WebAssembly.Memory, thread_stack_size?: number } | SyncInitInput, memory?: WebAssembly.Memory): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput>, memory?: WebAssembly.Memory, thread_stack_size?: number }} module_or_path - Passing `InitInput` directly is deprecated.
 * @param {WebAssembly.Memory} memory - Deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput>, memory?: WebAssembly.Memory, thread_stack_size?: number } | InitInput | Promise<InitInput>, memory?: WebAssembly.Memory): Promise<InitOutput>;
