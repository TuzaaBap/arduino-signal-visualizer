import { deflateSync } from "node:zlib";
import { writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const size = 256;
const scale = 3;
const canvasSize = size * scale;
const pixels = new Uint8Array(canvasSize * canvasSize * 4);

const color = (hex) => [
  Number.parseInt(hex.slice(1, 3), 16),
  Number.parseInt(hex.slice(3, 5), 16),
  Number.parseInt(hex.slice(5, 7), 16),
  255,
];

const backgroundTop = color("#123943");
const backgroundBottom = color("#07161c");
const border = color("#2b6570");
const teal = color("#61ddd3");
const green = color("#4be38a");
const white = color("#eefdfa");

function mix(first, second, amount) {
  return first.map((value, index) =>
    Math.round(value + (second[index] - value) * amount),
  );
}

function blendPixel(x, y, source, alpha = 1) {
  if (x < 0 || y < 0 || x >= canvasSize || y >= canvasSize) {
    return;
  }
  const index = (y * canvasSize + x) * 4;
  for (let channel = 0; channel < 3; channel += 1) {
    pixels[index + channel] = Math.round(
      pixels[index + channel] * (1 - alpha) + source[channel] * alpha,
    );
  }
  pixels[index + 3] = 255;
}

function insideRoundedSquare(x, y, inset, radius) {
  const left = inset;
  const top = inset;
  const right = canvasSize - inset;
  const bottom = canvasSize - inset;
  const nearestX = Math.max(left + radius, Math.min(x, right - radius));
  const nearestY = Math.max(top + radius, Math.min(y, bottom - radius));
  const dx = x - nearestX;
  const dy = y - nearestY;
  return (
    (x >= left + radius && x <= right - radius && y >= top && y <= bottom) ||
    (y >= top + radius && y <= bottom - radius && x >= left && x <= right) ||
    dx * dx + dy * dy <= radius * radius
  );
}

for (let y = 0; y < canvasSize; y += 1) {
  for (let x = 0; x < canvasSize; x += 1) {
    if (!insideRoundedSquare(x, y, 0, 56 * scale)) {
      continue;
    }
    const diagonal = (x + y) / (canvasSize * 2);
    blendPixel(x, y, mix(backgroundTop, backgroundBottom, diagonal));
    const inInner = insideRoundedSquare(x, y, 17 * scale, 43 * scale);
    const inBorderOuter = insideRoundedSquare(x, y, 14 * scale, 45 * scale);
    if (inBorderOuter && !inInner) {
      blendPixel(x, y, border, 0.9);
    }
  }
}

function drawDisc(centerX, centerY, radius, fill) {
  const minX = Math.floor(centerX - radius);
  const maxX = Math.ceil(centerX + radius);
  const minY = Math.floor(centerY - radius);
  const maxY = Math.ceil(centerY + radius);
  for (let y = minY; y <= maxY; y += 1) {
    for (let x = minX; x <= maxX; x += 1) {
      const distance = Math.hypot(x - centerX, y - centerY);
      if (distance <= radius) {
        blendPixel(x, y, fill, Math.min(1, radius - distance + 0.5));
      }
    }
  }
}

function drawPolyline(points, radius, firstColor, secondColor = firstColor) {
  for (let segment = 0; segment < points.length - 1; segment += 1) {
    const [startX, startY] = points[segment];
    const [endX, endY] = points[segment + 1];
    const steps = Math.max(Math.abs(endX - startX), Math.abs(endY - startY));
    for (let step = 0; step <= steps; step += 1) {
      const amount = step / Math.max(1, steps);
      drawDisc(
        startX + (endX - startX) * amount,
        startY + (endY - startY) * amount,
        radius,
        mix(firstColor, secondColor, (segment + amount) / (points.length - 1)),
      );
    }
  }
}

const s = scale;
drawPolyline(
  [
    [50 * s, 128 * s],
    [67 * s, 91 * s],
    [104 * s, 89 * s],
    [132 * s, 128 * s],
    [160 * s, 167 * s],
    [197 * s, 165 * s],
    [214 * s, 128 * s],
    [197 * s, 91 * s],
    [160 * s, 89 * s],
    [132 * s, 128 * s],
    [104 * s, 167 * s],
    [67 * s, 165 * s],
    [50 * s, 128 * s],
  ],
  9 * s,
  teal,
  green,
);
drawPolyline(
  [
    [93 * s, 128 * s],
    [112 * s, 128 * s],
    [121 * s, 105 * s],
    [135 * s, 151 * s],
    [144 * s, 128 * s],
    [163 * s, 128 * s],
  ],
  3.5 * s,
  white,
);

const downsampled = new Uint8Array(size * size * 4);
for (let y = 0; y < size; y += 1) {
  for (let x = 0; x < size; x += 1) {
    const sums = [0, 0, 0, 0];
    for (let sampleY = 0; sampleY < scale; sampleY += 1) {
      for (let sampleX = 0; sampleX < scale; sampleX += 1) {
        const source =
          (((y * scale + sampleY) * canvasSize + x * scale + sampleX) * 4);
        for (let channel = 0; channel < 4; channel += 1) {
          sums[channel] += pixels[source + channel];
        }
      }
    }
    const target = (y * size + x) * 4;
    for (let channel = 0; channel < 4; channel += 1) {
      downsampled[target + channel] = Math.round(
        sums[channel] / (scale * scale),
      );
    }
  }
}

function crc32(data) {
  let crc = 0xffffffff;
  for (const byte of data) {
    crc ^= byte;
    for (let bit = 0; bit < 8; bit += 1) {
      crc = (crc >>> 1) ^ (crc & 1 ? 0xedb88320 : 0);
    }
  }
  return (crc ^ 0xffffffff) >>> 0;
}

function pngChunk(name, data) {
  const type = Buffer.from(name, "ascii");
  const output = Buffer.alloc(12 + data.length);
  output.writeUInt32BE(data.length, 0);
  type.copy(output, 4);
  data.copy(output, 8);
  output.writeUInt32BE(crc32(Buffer.concat([type, data])), 8 + data.length);
  return output;
}

const rawRows = Buffer.alloc((size * 4 + 1) * size);
for (let y = 0; y < size; y += 1) {
  const row = y * (size * 4 + 1);
  rawRows[row] = 0;
  Buffer.from(downsampled.buffer, y * size * 4, size * 4).copy(rawRows, row + 1);
}

const ihdr = Buffer.alloc(13);
ihdr.writeUInt32BE(size, 0);
ihdr.writeUInt32BE(size, 4);
ihdr[8] = 8;
ihdr[9] = 6;
const png = Buffer.concat([
  Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]),
  pngChunk("IHDR", ihdr),
  pngChunk("IDAT", deflateSync(rawRows, { level: 9 })),
  pngChunk("IEND", Buffer.alloc(0)),
]);

const icoHeader = Buffer.alloc(22);
icoHeader.writeUInt16LE(0, 0);
icoHeader.writeUInt16LE(1, 2);
icoHeader.writeUInt16LE(1, 4);
icoHeader[6] = 0;
icoHeader[7] = 0;
icoHeader[8] = 0;
icoHeader[9] = 0;
icoHeader.writeUInt16LE(1, 10);
icoHeader.writeUInt16LE(32, 12);
icoHeader.writeUInt32LE(png.length, 14);
icoHeader.writeUInt32LE(22, 18);

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const iconDirectory = resolve(scriptDirectory, "../desktop/src-tauri/icons");
writeFileSync(resolve(iconDirectory, "icon.png"), png);
writeFileSync(resolve(iconDirectory, "icon.ico"), Buffer.concat([icoHeader, png]));
console.log("Generated icon.png and icon.ico");
