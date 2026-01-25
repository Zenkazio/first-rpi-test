const ws = new WebSocket("ws://" + location.host + "/ws");

ws.onmessage = (event) => {
  const msg = JSON.parse(event.data);

  if (msg.type === "CounterUpdate") {
    document.getElementById("counter").textContent = msg.value;
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

function sendSettings() {
  const data = {
    r: parseInt(document.getElementById("rRange").value),
    g: parseInt(document.getElementById("gRange").value),
    b: parseInt(document.getElementById("bRange").value),
    mode: document.getElementById("modeSelect").value,
    speed: parseFloat(document.getElementById("speedInput").value),
    repeat: document.getElementById("repeatInput").checked,
  };

  fetch("/led/settings", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(data),
  });
}
