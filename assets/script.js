const ws = new WebSocket("ws://" + location.host + "/ws");

ws.onmessage = (event) => {
  const msg = JSON.parse(event.data);

  if (msg.type === "StatusUpdate") {
    document.getElementById("status").textContent = msg.value;
  }

  if (msg.type === "PlaySound") {
    const player = document.getElementById("player");
    player.src = "/sounds/" + msg.name;
    player.play();
  }
};

function sendWs(type) {
  ws.send(JSON.stringify({ type }));
}

function updatePreview() {
  const r = document.getElementById("rRange").value;
  const g = document.getElementById("gRange").value;
  const b = document.getElementById("bRange").value;
  document.getElementById("colorPreview").style.backgroundColor =
    `rgb(${r},${g},${b})`;
  document.getElementById("previewText").textContent =
    `R: ${r} G: ${g} B: ${b}`;
}
function updateStepperPreview() {
  const steps = document.getElementById("steppersteps").value;

  document.getElementById("previewStepper").textContent = `Steps: ${steps}`;
}
function sendStepperStep() {
  const data = {
    type: "StepperStep",
    step: parseInt(document.getElementById("steppersteps").value),
  };

  ws.send(JSON.stringify(data));
}
function sendLEDSettings() {
  const data = {
    type: "UpdateSettings",
    r: parseInt(document.getElementById("rRange").value),
    g: parseInt(document.getElementById("gRange").value),
    b: parseInt(document.getElementById("bRange").value),
    mode: document.getElementById("modeSelect").value.toLowerCase(),
    speed: parseFloat(document.getElementById("speedInput").value),
    repeat: document.getElementById("repeatInput").checked,
  };

  ws.send(JSON.stringify(data));
}
function sendPlayerTable() {
  const data = {
    type: "PlayerTable",
    p1: document.getElementById("p1Select").value,
    p2: document.getElementById("p2Select").value,
    p3: document.getElementById("p3Select").value,
  };

  ws.send(JSON.stringify(data));
}

updateStepperPreview();
updatePreview();
