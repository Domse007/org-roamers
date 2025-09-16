import { type NodeDisplayData, type PartialButFor } from "sigma/types";
import { type Settings } from "sigma/settings";

const background = "--overlay";
const getColor = (v: string) =>
  getComputedStyle(document.documentElement).getPropertyValue(v).trim();

// this is stolen from sigma, because they refuse to expose context.fillStyle....
// the only thing changed are the colors computed by getColor
export default function drawHover(
  context: CanvasRenderingContext2D,
  data: PartialButFor<NodeDisplayData, "x" | "y" | "size" | "label" | "color">,
  settings: Settings,
): void {
  const size = settings.labelSize,
    font = settings.labelFont,
    weight = settings.labelWeight;

  context.font = `${weight} ${size}px ${font}`;

  // Then we draw the label background with modern styling
  context.fillStyle = getColor(background);
  context.shadowOffsetX = 0;
  context.shadowOffsetY = 2;
  context.shadowBlur = 6;
  context.shadowColor = "rgba(0, 0, 0, 0.15)";

  const PADDING = 2;

  if (typeof data.label === "string") {
    const textWidth = context.measureText(data.label).width,
      boxWidth = Math.round(textWidth + 5),
      boxHeight = Math.round(size + 2 * PADDING),
      radius = Math.max(data.size, size / 2) + PADDING;

    const angleRadian = Math.asin(boxHeight / 2 / radius);
    const xDeltaCoord = Math.sqrt(
      Math.abs(Math.pow(radius, 2) - Math.pow(boxHeight / 2, 2)),
    );

    context.beginPath();
    context.moveTo(data.x + xDeltaCoord, data.y + boxHeight / 2);
    context.lineTo(data.x + radius + boxWidth, data.y + boxHeight / 2);
    context.lineTo(data.x + radius + boxWidth, data.y - boxHeight / 2);
    context.lineTo(data.x + xDeltaCoord, data.y - boxHeight / 2);
    context.arc(data.x, data.y, radius, angleRadian, -angleRadian);
    context.closePath();
    context.fill();
  } else {
    context.beginPath();
    context.arc(data.x, data.y, data.size + PADDING, 0, Math.PI * 2);
    context.closePath();
    context.fill();
  }

  context.shadowOffsetX = 0;
  context.shadowOffsetY = 0;
  context.shadowBlur = 0;

  // And finally we draw the label
  drawLabel(context, data, settings);
}

function drawLabel(
  context: CanvasRenderingContext2D,
  data: PartialButFor<NodeDisplayData, "x" | "y" | "size" | "label" | "color">,
  settings: Settings,
): void {
  if (!data.label) return;

  const size = settings.labelSize,
    font = settings.labelFont,
    weight = settings.labelWeight;

  context.font = `${weight} ${size}px ${font}`;
  const width = context.measureText(data.label).width + 8;

  // Modern label background
  const labelX = data.x + data.size + 4;
  const labelY = data.y + size / 3 - 15;
  const labelHeight = 20;

  context.fillStyle = getColor(background);
  context.shadowOffsetX = 0;
  context.shadowOffsetY = 1;
  context.shadowBlur = 3;
  context.shadowColor = "rgba(0, 0, 0, 0.1)";

  // Create rectangle background
  context.fillRect(labelX, labelY, width, labelHeight);

  context.shadowBlur = 0;
  context.fillStyle = getColor("--text");
  context.fillText(data.label, labelX + 4, data.y + size / 3);
}
