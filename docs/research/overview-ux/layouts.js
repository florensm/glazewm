// Ports of window-overview layout algorithms, for the research artifacts.
// Rects are {x, y, w, h}. Each layout returns one rect per window, in order.

const OverviewLayouts = (() => {
  const union = (a, b) => {
    const x = Math.min(a.x, b.x), y = Math.min(a.y, b.y);
    return { x, y, w: Math.max(a.x + a.w, b.x + b.w) - x, h: Math.max(a.y + a.h, b.y + b.h) - y };
  };
  const bounds = (rs) => rs.slice(1).reduce(union, rs[0]);
  const center = (r) => [r.x + r.w / 2, r.y + r.h / 2];

  // GlazeWM `expose.rs` `spread`: one cluster, relative positions kept,
  // pushed apart to `gap`, at most `largest` of the room, never above `maxScale`.
  function cluster(windows, area, { gap = 28, largest = 0.85, maxScale = 1 } = {}) {
    if (!windows.length) return [];
    const rects = windows.map((r) => ({ x: r.x, y: r.y, w: Math.max(r.w, 1), h: Math.max(r.h, 1) }));
    const fit = () => {
      const b = bounds(rects);
      return Math.max(0, Math.min(largest * area.w / b.w, largest * area.h / b.h, maxScale));
    };
    let scale = fit();
    for (let round = 0; round < 6; round++) {
      separate(rects, gap / Math.max(scale, 1e-6));
      const next = fit();
      const settled = Math.abs(next - scale) <= scale * 0.01;
      scale = next;
      if (settled) break;
    }
    const [bx, by] = center(bounds(rects));
    const [ax, ay] = center(area);
    return rects.map((r) => {
      const [cx, cy] = center(r);
      const w = r.w * scale, h = r.h * scale;
      return { x: ax + (cx - bx) * scale - w / 2, y: ay + (cy - by) * scale - h / 2, w, h };
    });
  }

  function separate(rects, pad) {
    for (let it = 0; it < 500; it++) {
      let moved = false;
      for (let a = 0; a < rects.length; a++) {
        for (let b = a + 1; b < rects.length; b++) {
          const [ax, ay] = center(rects[a]), [bx, by] = center(rects[b]);
          const ox = (rects[a].w + rects[b].w) / 2 + pad - Math.abs(bx - ax);
          const oy = (rects[a].h + rects[b].h) / 2 + pad - Math.abs(by - ay);
          if (ox <= 0 || oy <= 0) continue;
          moved = true;
          const away = (f, t) => (t < f ? -1 : 1);
          if (ox <= oy) {
            const p = (ox / 2) * away(ax, bx);
            rects[a].x -= p; rects[b].x += p;
          } else {
            const p = (oy / 2) * away(ay, by);
            rects[a].y -= p; rects[b].y += p;
          }
        }
      }
      if (!moved) break;
    }
  }

  // GNOME Shell `workspace.js` `UnalignedLayoutStrategy` + `_createBestLayout`:
  // rows of windows sorted by position, small windows bumped up to 1.5x,
  // the row count picked by scale and space used, capped at `maxScale`.
  function rows(windows, area, { spacing = 6, maxScale = 0.95, monitorH = 1080 } = {}) {
    if (!windows.length) return [];
    const items = windows.map((r, i) => ({
      i, box: r, cx: r.x + r.w / 2, cy: r.y + r.h / 2,
      s: 1.5 + (1 - 1.5) * Math.min(1, r.h / monitorH),
    }));
    const compute = (numRows) => {
      const total = items.reduce((t, w) => t + w.box.w * w.s, 0);
      const ideal = total / numRows;
      const sorted = items.slice().sort((a, b) => a.cy - b.cy);
      const rs = [];
      let idx = 0;
      for (let r = 0; r < numRows; r++) {
        const row = { windows: [], fullWidth: 0, fullHeight: 0 };
        rs.push(row);
        for (; idx < sorted.length; idx++) {
          const w = sorted[idx];
          const width = w.box.w * w.s, height = w.box.h * w.s;
          row.fullHeight = Math.max(row.fullHeight, height);
          const keep = row.fullWidth + width <= ideal ||
            Math.abs(1 - (row.fullWidth + width) / ideal) < Math.abs(1 - row.fullWidth / ideal);
          if (keep || r === numRows - 1) { row.windows.push(w); row.fullWidth += width; } else break;
        }
      }
      let maxRow = null, gridH = 0;
      for (const row of rs) {
        row.windows.sort((a, b) => a.cx - b.cx);
        if (!maxRow || row.fullWidth > maxRow.fullWidth) maxRow = row;
        gridH += row.fullHeight;
      }
      return { numRows, rows: rs, maxColumns: maxRow.windows.length, gridW: maxRow.fullWidth, gridH };
    };
    const scaleSpace = (L) => {
      const hs = (L.maxColumns - 1) * spacing, vs = (L.numRows - 1) * spacing;
      const scale = Math.min((area.w - hs) / L.gridW, (area.h - vs) / L.gridH, maxScale);
      const space = ((L.gridW * scale + hs) * (L.gridH * scale + vs)) / (area.w * area.h);
      L.scale = scale;
      return [scale, space];
    };
    const better = (os, osp, s, sp) => {
      const spP = (sp - osp) * 0.1, scP = s - os;
      if (s > os && sp > osp) return true;
      if (s > os && sp <= osp) return scP > spP;
      if (s <= os && sp > osp) return spP > scP;
      return false;
    };
    let last = null, lastCols = -1, ls = 0, lsp = 0;
    for (let n = 1; n <= items.length; n++) {
      const cols = Math.ceil(items.length / n);
      if (cols === lastCols) break;
      const L = compute(n);
      const [s, sp] = scaleSpace(L);
      if (last && !better(ls, lsp, s, sp)) break;
      last = L; lastCols = cols; ls = s; lsp = sp;
    }
    // `computeWindowSlots`.
    const L = last, out = new Array(windows.length);
    for (const row of L.rows) {
      row.width = row.fullWidth * L.scale + (row.windows.length - 1) * spacing;
      row.height = row.fullHeight * L.scale;
    }
    const hNoSp = L.rows.reduce((t, r) => t + r.height, 0);
    const vSp = (L.rows.length - 1) * spacing;
    const addV = Math.min(1, (area.h - vSp) / hNoSp);
    let comp = 0, y = 0;
    for (const row of L.rows) {
      const hSp = (row.windows.length - 1) * spacing;
      const wNoSp = row.width - hSp;
      const addH = Math.min(1, (area.w - hSp) / wNoSp);
      if (addH < addV) { row.add = addH; comp += (addV - addH) * row.height; } else row.add = addV;
      row.x = area.x + Math.max(area.w - (wNoSp * row.add + hSp), 0) / 2;
      row.y = area.y + Math.max(area.h - (hNoSp + vSp), 0) / 2 + y;
      y += row.height * row.add + spacing;
    }
    comp /= 2;
    for (const row of L.rows) {
      const rowY = row.y + comp, rowH = row.height * row.add;
      let x = row.x;
      for (const w of row.windows) {
        let s = L.scale * w.s * row.add;
        const cw = w.box.w * s, ch = w.box.h * s;
        s = Math.min(s, maxScale);
        const ww = w.box.w * s, wh = w.box.h * s;
        const wx = x + (cw - ww) / 2;
        const wy = L.rows.length === 1 ? rowY + (rowH - wh) / 2 : rowY + rowH - ch + (ch - wh);
        out[w.i] = { x: wx, y: wy, w: ww, h: wh };
        x += cw + spacing;
      }
    }
    return out;
  }

  // An even grid, as Windows Task View and hyprexpo-style pickers show:
  // equal cells in reading order, each window fit to its cell.
  function grid(windows, area, { gap = 24, maxScale = 1 } = {}) {
    const n = windows.length;
    if (!n) return [];
    const order = windows.map((r, i) => ({ r, i })).sort((a, b) =>
      (Math.round((a.r.y + a.r.h / 2) / 200) - Math.round((b.r.y + b.r.h / 2) / 200)) ||
      (a.r.x - b.r.x));
    let best = null;
    for (let cols = 1; cols <= n; cols++) {
      const rowsN = Math.ceil(n / cols);
      const cw = (area.w - gap * (cols - 1)) / cols, ch = (area.h - gap * (rowsN - 1)) / rowsN;
      const score = order.reduce((t, { r }) => t + Math.min(cw / r.w, ch / r.h, maxScale), 0);
      if (!best || score > best.score) best = { cols, rowsN, cw, ch, score };
    }
    const out = new Array(n);
    const { cols, rowsN, cw, ch } = best;
    const usedH = rowsN * ch + (rowsN - 1) * gap;
    order.forEach(({ r, i }, k) => {
      const row = Math.floor(k / cols), col = k % cols;
      const inRow = Math.min(cols, n - row * cols);
      const rowW = inRow * cw + (inRow - 1) * gap;
      const s = Math.min(cw / r.w, ch / r.h, maxScale);
      const w = r.w * s, h = r.h * s;
      const x0 = area.x + (area.w - rowW) / 2 + col * (cw + gap);
      const y0 = area.y + (area.h - usedH) / 2 + row * (ch + gap);
      out[i] = { x: x0 + (cw - w) / 2, y: y0 + (ch - h) / 2, w, h };
    });
    return out;
  }

  // GlazeWM's exposé room on a `W`x`H` monitor (logical px, 48 px taskbar):
  // `layout.rs` `activities` scene with a compact row and no search line,
  // inset by `wallpaper.padding`. Returns `{ room, slab }`.
  function glazeRoom(W, H, { margin = 48, padding = 24, rowPart = 0.07, rowMin = 44, rowMax = 96, taskbar = 48 } = {}) {
    const top = 18, label = 26, rowH = Math.min(Math.max(H * rowPart, rowMin), rowMax);
    const sceneTop = top + rowH + label + 22;
    const scene = { x: margin, y: sceneTop, w: W - 2 * margin, h: H - margin - sceneTop };
    const room = { x: scene.x + padding, y: scene.y + padding, w: scene.w - 2 * padding, h: scene.h - 2 * padding };
    const work = { w: W, h: H - taskbar };
    const s = Math.min(room.w / work.w, room.h / work.h);
    const slab = { x: room.x + (room.w - work.w * s) / 2 - padding, y: room.y + (room.h - work.h * s) / 2 - padding,
      w: work.w * s + 2 * padding, h: work.h * s + 2 * padding };
    return { room, slab, rowH, sceneTop };
  }

  // Critically damped or underdamped spring from 0 to 1, unit mass.
  function spring(t, { stiffness = 800, damping = 1 } = {}) {
    const w = Math.sqrt(stiffness), b = damping * w;
    if (Math.abs(b - w) < 1e-6) return 1 - Math.exp(-w * t) * (1 + w * t);
    if (b < w) {
      const w1 = Math.sqrt(w * w - b * b);
      return 1 - Math.exp(-b * t) * (Math.cos(w1 * t) + (b / w1) * Math.sin(w1 * t));
    }
    const w2 = Math.sqrt(b * b - w * w), s1 = -b + w2, s2 = -b - w2;
    const c1 = -s2 / (s1 - s2);
    return 1 - (c1 * Math.exp(s1 * t) + (1 - c1) * Math.exp(s2 * t));
  }

  // Seconds a spring takes to stay within `eps` of 1.
  function springSettle(params, eps = 1e-4) {
    let last = 0;
    for (let t = 0; t < 5; t += 0.001) if (Math.abs(1 - spring(t, params)) > eps) last = t;
    return last;
  }

  function cubicBezier(x1, y1, x2, y2) {
    const bx = (t) => 3 * (1 - t) * (1 - t) * t * x1 + 3 * (1 - t) * t * t * x2 + t * t * t;
    const by = (t) => 3 * (1 - t) * (1 - t) * t * y1 + 3 * (1 - t) * t * t * y2 + t * t * t;
    return (x) => {
      let lo = 0, hi = 1;
      for (let i = 0; i < 30; i++) { const m = (lo + hi) / 2; if (bx(m) < x) lo = m; else hi = m; }
      return by((lo + hi) / 2);
    };
  }

  return { cluster, rows, grid, glazeRoom, spring, springSettle, cubicBezier, bounds };
})();

if (typeof module !== 'undefined') module.exports = OverviewLayouts;
