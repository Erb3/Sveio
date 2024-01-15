const targetElement = document.querySelector('h2');
const locateElement = document.querySelector('h3');
const progressElement = document.querySelector('progress');
const mapElement = document.querySelector('.map');
const socket = io();

function createMarkerIcon(color) {
  return L.icon({
    iconUrl: 'marker-icon-' + color + '.png',
    shadowUrl: 'marker-shadow.png',
    iconSize: [25, 41],
    iconAnchor: [12, 41],
    popupAnchor: [1, -34],
    tooltipAnchor: [16, -28],
    shadowSize: [41, 41],
    shadowAnchor: [12, 41],
  });
}

const greenMarker = createMarkerIcon('green');

const map = L.map('map', {
  center: [43.5, 10],
  minZoom: 3,
  maxZoom: 5,
  scrollWheelZoom: true,
  maxBounds: [
    [89.9, 160.9],
    [-89.9, -160.9],
  ],
  zoomControl: false,
  noWrap: true,
  zoomAnimation: true,
  markerZoomAnimation: true,
  doubleClickZoom: false,
}).setView([51.505, -0.09], 13);

L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
  attribution: '&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a>',
}).addTo(map);

map.setZoom(3);

const mySelectionMarker = L.marker([0, 0]);
const goalMarker = L.marker([0, 0], { icon: greenMarker });
const distanceLine = L.polyline([], { color: 'red' });
const distancePopup = L.popup();
let canMoveMarker = false;

map.on('click', (e) => {
  if (!canMoveMarker) return;
  mySelectionMarker.setLatLng(e.latlng).addTo(map);
});

socket.on('newTarget', (data) => {
  distanceLine.remove();
  mySelectionMarker.remove();
  goalMarker.remove();
  distancePopup.remove();
  locateElement.style.display = 'unset';
  targetElement.innerHTML = `${data.capital}, ${data.country}`;
  canMoveMarker = true;
  mapElement.classList.remove('cant');
  map.setZoom(3);
});

socket.on('solution', (data) => {
  console.log('received solution :) ', data);
  goalMarker.setLatLng([data.Latitude, data.Longitude]).addTo(map);
  locateElement.style.display = 'none';
  targetElement.innerHTML = 'Get ready...';

  if (map.hasLayer(mySelectionMarker)) {
    const coords = [mySelectionMarker.getLatLng(), [data.Latitude, data.Longitude]];
    distanceLine.setLatLngs(coords).addTo(map);
    map.fitBounds(distanceLine.getBounds());

    const distance = Math.round(map.distance(mySelectionMarker.getLatLng(), [data.Latitude, data.Longitude]) / 1000);
    distancePopup.setLatLng([data.Latitude, data.Longitude]).setContent(`Distance: ${distance} km`);
    goalMarker.bindPopup(distancePopup).openPopup();
  }

  canMoveMarker = false;
  mapElement.classList.add('cant');
});
