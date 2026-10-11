/** Classic vertical cross placements (col, row) in a 3×4 cell grid. */
const FACE_CELLS: ReadonlyArray<{ face: number; col: number; row: number }> = [
  { face: 2, col: 1, row: 0 }, // +Y
  { face: 1, col: 0, row: 1 }, // −X
  { face: 4, col: 1, row: 1 }, // +Z
  { face: 0, col: 2, row: 1 }, // +X
  { face: 3, col: 1, row: 2 }, // −Y
  { face: 5, col: 1, row: 3 }, // −Z
];

const GRID_COLS = 3;
const GRID_ROWS = 4;

function loadImage(url: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.onload = () => {
      resolve(img);
    };
    img.onerror = () => {
      reject(new Error("failed to load face image"));
    };
    img.src = url;
  });
}

async function loadFaceImages(
  faceUrls: ReadonlyArray<string | null>,
): Promise<Array<HTMLImageElement | null>> {
  return Promise.all(
    Array.from({ length: 6 }, async (_, i) => {
      const url = faceUrls[i] ?? null;
      if (url === null) {
        return null;
      }
      try {
        return await loadImage(url);
      } catch {
        return null;
      }
    }),
  );
}

function cellSizeFromImages(images: ReadonlyArray<HTMLImageElement | null>): number {
  let cell = 1;
  for (const img of images) {
    if (img !== null) {
      cell = Math.max(cell, img.naturalWidth, img.naturalHeight);
    }
  }
  return cell;
}

function fillCheckerCell(
  ctx: CanvasRenderingContext2D,
  x0: number,
  y0: number,
  cell: number,
  check: number,
): void {
  for (let y = 0; y < cell; y += check) {
    for (let x = 0; x < cell; x += check) {
      const on = (x / check + y / check) % 2 === 0;
      ctx.fillStyle = on ? "#2a2a2e" : "#1e1e22";
      ctx.fillRect(x0 + x, y0 + y, check, check);
    }
  }
}

function paintCrossBackground(ctx: CanvasRenderingContext2D, cell: number): void {
  const check = Math.max(4, Math.floor(cell / 16));
  for (const { col, row } of FACE_CELLS) {
    fillCheckerCell(ctx, col * cell, row * cell, cell, check);
  }
}

function drawFaceCover(
  ctx: CanvasRenderingContext2D,
  img: HTMLImageElement,
  x0: number,
  y0: number,
  cell: number,
): void {
  const scale = Math.max(cell / img.naturalWidth, cell / img.naturalHeight);
  const dw = img.naturalWidth * scale;
  const dh = img.naturalHeight * scale;
  const dx = x0 + (cell - dw) / 2;
  const dy = y0 + (cell - dh) / 2;
  ctx.drawImage(img, dx, dy, dw, dh);
}

function paintFaces(
  ctx: CanvasRenderingContext2D,
  images: ReadonlyArray<HTMLImageElement | null>,
  cell: number,
): void {
  for (const { face, col, row } of FACE_CELLS) {
    const img = images[face] ?? null;
    if (img !== null) {
      drawFaceCover(ctx, img, col * cell, row * cell, cell);
    }
  }
}

function canvasToPngBlob(canvas: HTMLCanvasElement): Promise<Blob> {
  return new Promise((resolve, reject) => {
    canvas.toBlob((b) => {
      if (b === null) {
        reject(new Error("cubemap cross encode failed"));
        return;
      }
      resolve(b);
    }, "image/png");
  });
}

/**
 * Composite up to six face object-URLs into a classic cubemap cross PNG blob URL.
 * Missing faces leave empty (transparent) cells. Caller must revoke the returned URL.
 */
export async function composeCubemapCross(
  faceUrls: ReadonlyArray<string | null>,
): Promise<{ url: string; width: number; height: number }> {
  const images = await loadFaceImages(faceUrls);
  const cell = cellSizeFromImages(images);
  const width = GRID_COLS * cell;
  const height = GRID_ROWS * cell;
  const canvas = document.createElement("canvas");
  canvas.width = width;
  canvas.height = height;
  const ctx = canvas.getContext("2d");
  if (ctx === null) {
    throw new Error("2d canvas unavailable");
  }

  paintCrossBackground(ctx, cell);
  paintFaces(ctx, images, cell);

  const blob = await canvasToPngBlob(canvas);
  return {
    url: URL.createObjectURL(blob),
    width,
    height,
  };
}
