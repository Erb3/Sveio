const targetElement = document.querySelector("nav > p");
const progressElement = document.querySelector(".progress > div");
const mapElement = document.querySelector("#map");
const leaderboardElement = document.querySelector("#leaderboard");
const spinner = document.querySelector("#spinner");
const socket = io();
let targetAnnounced = false;

function createMarkerIcon(color) {
  return L.icon({
    iconUrl: `https://raw.githubusercontent.com/pointhi/leaflet-color-markers/master/img/marker-icon-${color}.png`,
    shadowUrl:
      "https://cdnjs.cloudflare.com/ajax/libs/leaflet/0.7.7/images/marker-shadow.png",
    iconSize: [25, 41],
    iconAnchor: [12, 41],
    popupAnchor: [1, -34],
    tooltipAnchor: [16, -28],
    shadowSize: [41, 41],
    shadowAnchor: [12, 41],
  });
}

const greenMarker = createMarkerIcon("green");
const blueMarker = createMarkerIcon("blue");

const map = L.map("map", {
  center: [43.5, 10],
  minZoom: 3,
  maxZoom: 5,
  scrollWheelZoom: true,
  maxBounds: [
    [-90, -180],
    [90, 180],
  ],
  zoomControl: false,
  noWrap: true,
  zoomAnimation: true,
  markerZoomAnimation: true,
  doubleClickZoom: false,
}).setView([51.505, -0.09], 13);

L.tileLayer(
  "https://a.basemaps.cartocdn.com/light_nolabels/{z}/{x}/{y}@2x.png",
  {
    attribution:
      '&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a>, &copy; <a href="https://carto.com/attributions">CARTO</a>',
  }
).addTo(map);
map.attributionControl.setPosition("bottomleft");
map.setZoom(3);

const mySelectionMarker = L.marker([0, 0]);
const goalMarker = L.marker([0, 0], { icon: greenMarker });
const distanceLine = L.polyline([], { color: "red" });
const otherPlayerMarkers = [];
let canMoveMarker = false;
let guessingTime = 5;

map.on("click", (e) => {
  if (!canMoveMarker) return;
  mySelectionMarker.setLatLng(e.latlng).addTo(map);
  console.log("Emitting", e.latlng);
  socket.emit("guess", {
    lat: e.latlng.lat,
    long: e.latlng.lng,
  });
});

socket.on("newTarget", (data) => {
  targetAnnounced = true;
  distanceLine.remove();
  mySelectionMarker.remove();
  goalMarker.remove();

  otherPlayerMarkers.forEach((marker) => {
    marker.remove();
  });

  targetElement.innerHTML = `${data.name}, ${data.country}`;
  progressElement.style.width = "100%";
  progressElement.style.transitionDuration = guessingTime + "s";
  canMoveMarker = true;
  mapElement.classList.remove("cursor-grab");
  map.setZoom(3);
});

socket.on("solution", (data) => {
  if (!targetAnnounced) return;

  const goalCoords = [data.location.latitude, data.location.longitude];
  goalMarker.setLatLng(goalCoords).addTo(map);
  targetElement.innerHTML = "The target location will appear here";
  progressElement.style.width = "0%";
  progressElement.style.transitionDuration = "1s";

  if (map.hasLayer(mySelectionMarker)) {
    const coords = [mySelectionMarker.getLatLng(), goalCoords];
    distanceLine.setLatLngs(coords).addTo(map);
    map.fitBounds(distanceLine.getBounds());

    const distance = Math.round(
      map.distance(mySelectionMarker.getLatLng(), goalCoords) / 1000
    );
    goalMarker
      .bindPopup(`Location: ${data.location.name}<br>Distance: ${distance} km`)
      .openPopup();
  }

  for (const [sid, guess] of Object.entries(data.guesses)) {
    if (sid === socket.id) continue;

    console.log("Adding someone elses marker", guess, data.guesses, sid);
    const marker = L.marker([guess.lat, guess.long], { icon: blueMarker });
    marker.addTo(map);
    console.log(data.leaderboard);
    marker.bindPopup(data.leaderboard[sid].username).openPopup();
    otherPlayerMarkers.push(marker);
  }

  const leaderboard = [];
  Object.entries(data.leaderboard)
    .sort((a, b) => b[1][1] - a[1][1])
    .slice(0, 10)
    .forEach(([, { username, score }], index) => {
      leaderboard.push(`${index + 1}. ${username} - ${score}`);
    });
  leaderboardElement.innerHTML = leaderboard.join("<br>");

  canMoveMarker = false;
  mapElement.classList.add("cursor-grab");
});

socket.on("game-metadata", (data) => {
  spinner.style.opacity = 0;
  setTimeout(() => {
    spinner.style.display = "none";
  }, 750);

  console.log("Connected to game!", data);
  guessingTime = data.guess_time;
});

socket.on("kick", (data) => {
  console.log("Kicked!", data);
  location.href = "/?message=" + data.message;
});

socket.on("connect", () => {
  socket.emit("join", {
    username: localStorage.getItem("username"),
    game: "PRIMARY",
  });
});
