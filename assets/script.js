const ws = new WebSocket("ws://" + location.host + "/ws");
const canvas = document.getElementById("myCanvas");
const ctx = canvas.getContext("2d");
const points = new Map();

ws.addEventListener("close", (event) => {
  window.location.reload();
});
ws.addEventListener("error", (event) => {
  window.location.reload();
});

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
  if (msg.type === "Targets") {
    points[msg.id] = msg.targets;
    // console.log(points[3]);
    drawCanvas();
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

function toCanvasCoords(x, y) {
  const maxCoord = 4000;
  const scale = Math.min(canvas.width, canvas.height) / maxCoord;
  const originX = canvas.width / 2;
  const originY = canvas.height;
  return {
    x: originX + x * scale,
    y: originY - y * scale, // Y-Achse nach unten
  };
}

function drawArrow(ctx, x, y, vx, vy, color, scale = 20) {
  // Skaliere den Vektor für die Darstellung
  const length = Math.sqrt(vx * vx + vy * vy);
  const nx = (vx / length) * scale; // Normierter Vektor (x)
  const ny = (vy / length) * scale; // Normierter Vektor (y)

  // Linie zeichnen (Pfeilschaft)
  ctx.beginPath();
  ctx.moveTo(x, y);
  ctx.lineTo(x + nx, y - ny); // Minus ny, weil Canvas-Y nach unten zeigt
  ctx.strokeStyle = color; // Farbe des Pfeils
  ctx.lineWidth = 2;
  ctx.stroke();

  // Pfeilspitze zeichnen (Dreieck)
  const angle = Math.atan2(ny, nx);
  const arrowSize = 6;
  ctx.beginPath();
  ctx.moveTo(x + nx, y - ny);
  ctx.lineTo(
    x + nx - arrowSize * Math.cos(angle - Math.PI / 6),
    y - ny + arrowSize * Math.sin(angle - Math.PI / 6),
  );
  ctx.lineTo(
    x + nx - arrowSize * Math.cos(angle + Math.PI / 6),
    y - ny + arrowSize * Math.sin(angle + Math.PI / 6),
  );
  ctx.closePath();
  ctx.fillStyle = color;
  ctx.fill();
}
function berechneWinkel(vec1_x, vec1_y, vec2_x, vec2_y) {
  // Skalarprodukt berechnen
  const skalarprodukt = vec1_x * vec2_x + vec1_y * vec2_y;

  // Magnituden berechnen
  const magnitude1 = Math.sqrt(vec1_x * vec1_x + vec1_y * vec1_y);
  const magnitude2 = Math.sqrt(vec2_x * vec2_x + vec2_y * vec2_y);

  // Kosinus des Winkels berechnen
  const cosTheta = skalarprodukt / (magnitude1 * magnitude2);

  // Winkel in Radiant und dann in Grad umrechnen
  const winkelRadiant = Math.acos(cosTheta);
  const winkelGrad = winkelRadiant * (180 / Math.PI);

  return winkelGrad;
}
function drawCanvas() {
  // Canvas leeren
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  // Koordinatenursprung (roter Punkt)
  const originX = canvas.width / 2;
  const originY = canvas.height;
  ctx.lineWidth = 4; // Dicke des Randes in Pixel
  ctx.strokeStyle = "black";
  ctx.beginPath();
  ctx.arc(originX, originY, 375, 0, Math.PI * 2);
  ctx.stroke();
  ctx.fillStyle = "red";
  ctx.beginPath();
  ctx.arc(originX, originY, 8, 0, Math.PI * 2);
  ctx.fill();
  // Punkte zeichnen
  for (const [id, points1] of Object.entries(points)) {
    points1.forEach((point) => {
      if (point.points[0] !== (0, 0) && point.speed !== 0) {
        const c3 = point.done ? "green" : id == 3 ? "blue" : "cyan";
        ctx.fillStyle = c3;
        const coords = toCanvasCoords(point.points[0][0], point.points[0][1]);

        ctx.beginPath();
        ctx.arc(coords.x, coords.y, 20, 0, Math.PI * 2);
        ctx.fill();
        ctx.font = "bold 14px Arial"; // Fett, 14px, Schriftart Arial
        ctx.fillStyle = "black";
        // ctx.fillText(
        //   `${point.label} (${point.x.toFixed(1)}, ${point.y.toFixed(1)},${angle.toFixed(2)}, ${length1.toFixed(2)})`,
        //   coords.x + 40,
        //   coords.y,
        // );

        // const c1 = "red";
        // drawArrow(
        //   ctx,
        //   originX,
        //   originY,
        //   point.points[0][0],
        //   point.points[0][1],
        //   (color = c1),
        //   50,
        // );
        // const c2 = "red";
        // drawArrow(
        //   ctx,
        //   coords.x,
        //   coords.y,
        //   point.vec_x,
        //   point.vec_y,
        //   (color = c2),
        //   50,
        // );
      }
    });
  }
}

updatePreview();
