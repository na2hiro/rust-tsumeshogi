import * as wasm from "tsumeshogi";

document.getElementById("submit").addEventListener("click", (e) => {
    e.preventDefault();

    const sfen = document.getElementById("input").value;
    console.log(sfen)
    var result = wasm.solve_dfpn(sfen);
    console.log(result);
    document.getElementById("result").textContent = JSON.stringify(result, null, 2);
    document.getElementById("answer").textContent = result.is_tsumi ? "詰" + result.moves.join(" ") : "不詰";

})

export function solve(sfen) {
    return wasm.solve_dfpn(sfen);
}
