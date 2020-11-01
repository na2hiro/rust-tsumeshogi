if(!window.Worker) {
    alert("Web worker is needed! aborting");
    throw "web worker is needed";
}

let worker = new Worker("./worker-bootstrap.js", { type: 'module', name: "tsume-worker" });


worker.onmessage = (e) => {
    if(e.data===1) {
        ready();
        return;
    }
    var [result, elapsedTime] = e.data;
    console.log(result);
    document.getElementById("result").textContent = JSON.stringify(result, null, 2);
    document.getElementById("answer").textContent = result.is_tsumi ? "詰" + result.moves.join(" ") : "不詰";
    document.getElementById("stopwatch").textContent = `elapsed = ${elapsedTime / 1000}s
nps (effective) = ${result.nodes/elapsedTime*1000}
nps (including temporary) = ${result.nodes_incl_temporary/elapsedTime*1000}`;
}

function ready() {
    document.getElementById("answer").textContent = "";
    document.getElementById("submit").addEventListener("click", (e) => {
        e.preventDefault();

        document.getElementById("result").textContent = "";
        document.getElementById("answer").textContent = "Solving...";
        document.getElementById("stopwatch").textContent = "";

        const sfen = document.getElementById("input").value;
        console.log(sfen)
        worker.postMessage([sfen, new Date().getTime()]);
    })
}

const clock = document.getElementById("clock");
function s () {
    requestAnimationFrame(() => {
        clock.textContent = new Date().toISOString();
        s();
    })
}
s();
