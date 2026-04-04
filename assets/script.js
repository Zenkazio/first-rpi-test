let ws;
const canvas = document.getElementById("myCanvas");
const ctx = canvas.getContext("2d");
const points = new Map();

function connect() {
  ws = new WebSocket("ws://" + location.host + "/ws");

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
      drawCanvas();
    }
  };

  ws.onclose = () => {
    console.log("Verbindung verloren. Erneuter Versuch in 5s...");
    setTimeout(connect, 5000);
  };

  ws.onerror = (err) => {
    ws.close();
  };
}

function sendWs(type) {
  if (ws && ws.readyState === WebSocket.OPEN) {
    ws.send(JSON.stringify({ type }));
  } else {
    console.warn("WS nicht bereit. Nachricht verworfen.");
  }
}

connect();

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
function zeichneKegelZuPunkt(x1, y1, x2, y2, oeffnungGrad) {
  const dx = x2 - x1;
  const dy = y2 - y1;

  // Radius (Hypotenuse) berechnen
  const radius = Math.sqrt(dx * dx + dy * dy);

  // Mittelwinkel in Radiant berechnen
  const mittelWinkel = Math.atan2(dy, dx);

  // Halber Öffnungswinkel in Radiant
  const halbOeffnungRad = (oeffnungGrad / 2) * (Math.PI / 180);

  ctx.beginPath();
  ctx.moveTo(x1, y1);
  ctx.arc(
    x1,
    y1,
    radius,
    mittelWinkel - halbOeffnungRad,
    mittelWinkel + halbOeffnungRad,
  );
  ctx.lineTo(x1, y1);
  ctx.fill();
  ctx.stroke();
}
function drawCanvas() {
  // Canvas leeren
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  // Koordinatenursprung (roter Punkt)
  const originX = canvas.width / 2;
  const originY = canvas.height;

  // 1. Die senkrechte Linie (z.B. 40px lang nach oben)
  ctx.lineWidth = 2; // Dicke des Randes in Pixel
  ctx.strokeStyle = "black";
  ctx.fillStyle = "black";
  ctx.beginPath();
  ctx.moveTo(originX, originY);
  ctx.lineTo(originX, 0);
  ctx.stroke();

  for (let i = 1; i <= 6; i++) {
    let currentRadius = i * 125;
    let label = (i * 0.5).toFixed(1) + "m";
    let currentY = originY - currentRadius;

    // 1. Kreis zeichnen (oder Halbkreis)
    ctx.beginPath();
    ctx.arc(originX, originY, currentRadius, 0, Math.PI * 2);
    ctx.stroke();

    // 3. Text rechts neben der Markierung
    ctx.font = "14px Arial";
    ctx.textAlign = "left";
    ctx.textBaseline = "middle";
    ctx.fillText(label, originX + 5, currentY - 10);
  }

  ctx.fillStyle = "red";
  ctx.beginPath();
  ctx.arc(originX, originY, 8, 0, Math.PI * 2);
  ctx.fill();
  // Punkte zeichnen
  for (const [id, points1] of Object.entries(points)) {
    points1.forEach((point, i) => {
      if (point.points[0][0] !== 0 && point.points[0][1] !== 0) {
        const c3 = point.is_open_door ? "green" : id == 3 ? "blue" : "cyan";
        ctx.fillStyle = c3;
        const coords = toCanvasCoords(point.points[0][0], point.points[0][1]);

        ctx.beginPath();
        ctx.arc(coords.x, coords.y, 20, 0, Math.PI * 2);
        ctx.fill();
        ctx.font = "bold 14px Arial"; // Fett, 14px, Schriftart Arial
        ctx.fillStyle = "black";
        ctx.fillText(
          `${i + 1} (${point.points[0][0].toFixed(1)}, ${point.points[0][1].toFixed(1)}, ${point.speeds[0].toFixed(1)}, ${point.distances[0].toFixed(1)})`,
          coords.x + 20,
          coords.y,
        );

        for (let i = 0; i < 5; i++) {
          ctx.fillStyle = "rgba(255, 0, 0, 0.5)";
          ctx.strokeStyle = "rgba(255, 0, 0, 0.5)"; // Farbe des Pfeils
          ctx.lineWidth = 2;

          zeichneKegelZuPunkt(
            coords.x,
            coords.y,
            coords.x + point.vecs[i][0] * 1000,
            coords.y - point.vecs[i][1] * 1000,
            10,
          );
        }
      }
    });
  }
}

updatePreview();
