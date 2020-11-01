import * as wasm from "tsumeshogi";

postMessage(1);

onmessage = (e) => {
    const [sfen, time] = e.data;
    const result = wasm.solve_dfpn(sfen);
    postMessage([result, new Date().getTime() - time]);
}
