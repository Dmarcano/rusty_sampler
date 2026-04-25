
function buildWorkletDebugInfo() {
    // This inline module is at: snippets/<crate>-<hash>/inline0.js
    // Main module is at: sampler_web_wasm.js (2 levels up)
    const bindgenUrl = new URL('../../sampler_web_wasm.js', import.meta.url).href;
    const polyfillUrl = URL.createObjectURL(new Blob([`
        if (!globalThis.TextDecoder) {
            globalThis.TextDecoder = class TextDecoder {
                decode(arg) {
                    if (typeof arg !== 'undefined') {
                        throw Error('TextDecoder stub called');
                    } else {
                        return '';
                    }
                }
            };
        }

        if (!globalThis.TextEncoder) {
            globalThis.TextEncoder = class TextEncoder {
                encode(arg) {
                    if (typeof arg !== 'undefined') {
                        throw Error('TextEncoder stub called');
                    } else {
                        return new Uint8Array(0);
                    }
                }
            };
        }

        export function nop() {}
    `], { type: 'text/javascript' }));

    const workletSource = `
        import '${polyfillUrl}';
        import * as bindgen from '${bindgenUrl}';

        registerProcessor('SineWorkletNode', class SineWorkletNode extends AudioWorkletProcessor {
            constructor(options) {
                super();
                const processorOptions = options?.processorOptions;
                if (!processorOptions) {
                    throw new Error('Missing processorOptions in AudioWorkletProcessor constructor');
                }
                let [module, memory, handle] = processorOptions;
                bindgen.initSync({ module, memory });
                this.processor = bindgen.SineWorkletNode.unpack(handle);

                this.port.onmessage = (event) => {
                    const message = event.data ?? {};

                    switch (message.type) {
                        case 'setFrequency':
                            this.processor.set_frequency_hz_inner(message.value);
                            break;
                        case 'setAmplitude':
                            this.processor.set_amplitude_inner(message.value);
                            break;
                    }
                };
            }
            process(inputs, outputs) {
                return this.processor.process(outputs[0][0]);
            }
        });
    `;

    return {
        bindgenUrl,
        polyfillUrl,
        workletSource,
    };
}

export function createWorkletModuleUrl() {
    const info = buildWorkletDebugInfo();
    return URL.createObjectURL(new Blob([info.workletSource], { type: 'text/javascript' }));
}

export function createWorkletDebugInfo() {
    return buildWorkletDebugInfo();
}

export function createWorkletNode(ctx, module, memory, handle) {
    return new AudioWorkletNode(ctx, 'SineWorkletNode', {
        processorOptions: [module, memory, handle],
    });
}
