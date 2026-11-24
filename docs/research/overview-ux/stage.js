// A mock 1920x1080 desktop that opens into an overview and closes again.
// Needs `layouts.js` loaded first. Used by every page in this folder.

const OverviewStage = (() => {
  const L = OverviewLayouts;
  const W = 1920, H = 1080, TASKBAR = 48;
  const WORK = { x: 0, y: 0, w: W, h: H - TASKBAR };

  const APPS = [
    { name: 'Code', title: 'layout.rs — glazewm', kind: 'editor', tint: '#3b82f6' },
    { name: 'Firefox', title: 'Mission Control — Apple Support', kind: 'browser', tint: '#f97316' },
    { name: 'Terminal', title: 'cargo test --workspace', kind: 'terminal', tint: '#22c55e' },
    { name: 'Chat', title: '#window-managers', kind: 'chat', tint: '#a855f7' },
    { name: 'Music', title: 'Now playing', kind: 'music', tint: '#ec4899' },
    { name: 'Files', title: 'Downloads', kind: 'files', tint: '#eab308' },
    { name: 'Notes', title: 'overview ideas.md', kind: 'notes', tint: '#14b8a6' },
    { name: 'Mail', title: 'Inbox (3)', kind: 'browser', tint: '#0ea5e9' },
  ];

  // Presets: what each overview does, as closely as the research allows.
  const PRESETS = {
    glazewm: {
      label: 'GlazeWM today', layout: 'cluster', largest: 0.85, margin: 48, padding: 24, gap: 28,
      strip: 'compact', wallpaper: 'slab', radius: 14, maxScale: 1,
      open: { kind: 'spring', stiffness: 800, damping: 1 }, close: { kind: 'spring', stiffness: 800, damping: 1 },
      staggerMs: 0, chromeFrom: 0.8, lift: 0.025, dim: 0, zoom: 0.5,
    },
    proposed: {
      label: 'GlazeWM proposed', layout: 'cluster', largest: 0.94, margin: 32, padding: 16, gap: 24,
      strip: 'compact', wallpaper: 'slab', radius: 14, maxScale: 1,
      open: { kind: 'spring', stiffness: 520, damping: 0.85 }, close: { kind: 'spring', stiffness: 800, damping: 1 },
      staggerMs: 10, chromeFrom: 0.6, lift: 0.04, dim: 0.12, zoom: 0.5,
    },
    macos: {
      label: 'macOS (observed)', layout: 'cluster', largest: 0.92, margin: 40, padding: 0, gap: 40,
      strip: 'names', wallpaper: 'dim', radius: 10, maxScale: 1,
      open: { kind: 'bezier', ms: 300, curve: [0.25, 0.8, 0.25, 1] }, close: { kind: 'bezier', ms: 280, curve: [0.25, 0.8, 0.25, 1] },
      staggerMs: 0, chromeFrom: 0.7, lift: 0.0, dim: 0, zoom: 0.5,
    },
    gnome: {
      label: 'GNOME', layout: 'rows', largest: 1, margin: 24, padding: 0, gap: 18, rowsMax: 0.95,
      strip: 'thumbs', wallpaper: 'slab', radius: 30, maxScale: 0.95,
      open: { kind: 'bezier', ms: 250, curve: [0.61, 1, 0.88, 1] }, close: { kind: 'bezier', ms: 250, curve: [0.5, 1, 0.89, 1] },
      staggerMs: 0, chromeFrom: 0.9, lift: 0.0125, dim: 0, zoom: 0.5,
    },
    kde: {
      label: 'KDE Plasma', layout: 'rows', largest: 1, margin: 32, padding: 0, gap: 52, rowsMax: 1,
      strip: 'thumbs', wallpaper: 'blur', radius: 8, maxScale: 1,
      open: { kind: 'bezier', ms: 300, curve: [0.33, 1, 0.68, 1] }, close: { kind: 'bezier', ms: 300, curve: [0.33, 1, 0.68, 1] },
      staggerMs: 0, chromeFrom: 0.8, lift: 0, dim: 0, zoom: 0.5,
    },
    niri: {
      label: 'niri', layout: 'rigid', largest: 1, margin: 0, padding: 0, gap: 0,
      strip: 'none', wallpaper: 'slab', radius: 0, maxScale: 1, surround: '#262626',
      open: { kind: 'spring', stiffness: 800, damping: 1 }, close: { kind: 'spring', stiffness: 800, damping: 1 },
      staggerMs: 0, chromeFrom: 0.8, lift: 0, dim: 0, zoom: 0.5,
    },
    windows: {
      label: 'Windows (observed)', layout: 'grid', largest: 1, margin: 96, padding: 0, gap: 48,
      strip: 'bottom', wallpaper: 'blur', radius: 8, maxScale: 0.9,
      open: { kind: 'bezier', ms: 250, curve: [0.1, 0.9, 0.2, 1] }, close: { kind: 'bezier', ms: 200, curve: [0.1, 0.9, 0.2, 1] },
      staggerMs: 0, chromeFrom: 0.5, lift: 0, dim: 0, zoom: 0.5,
    },
  };

  // Window rects on the working area for `count` windows `arrangement`d.
  function scenario(count, arrangement) {
    const n = Math.max(1, Math.min(8, count));
    const g = 8;
    if (arrangement === 'tiled') {
      const out = [];
      let area = { x: g, y: g, w: WORK.w - 2 * g, h: WORK.h - 2 * g };
      for (let i = 0; i < n; i++) {
        if (i === n - 1) { out.push(area); break; }
        const vertical = i % 2 === 0;
        if (vertical) {
          const w = (area.w - g) / 2;
          out.push({ x: area.x, y: area.y, w, h: area.h });
          area = { x: area.x + w + g, y: area.y, w, h: area.h };
        } else {
          const h = (area.h - g) / 2;
          out.push({ x: area.x, y: area.y, w: area.w, h });
          area = { x: area.x, y: area.y + h + g, w: area.w, h };
        }
      }
      return out;
    }
    const floating = [
      { x: 140, y: 90, w: 1040, h: 680 }, { x: 980, y: 220, w: 820, h: 600 },
      { x: 420, y: 470, w: 760, h: 480 }, { x: 1320, y: 60, w: 520, h: 420 },
      { x: 60, y: 560, w: 560, h: 400 }, { x: 1260, y: 600, w: 600, h: 380 },
      { x: 760, y: 120, w: 420, h: 300 }, { x: 640, y: 720, w: 380, h: 260 },
    ];
    if (arrangement === 'onebig') {
      const big = { x: 0, y: 0, w: WORK.w, h: WORK.h };
      const small = [
        { x: 1180, y: 520, w: 640, h: 420 }, { x: 160, y: 140, w: 480, h: 340 },
        { x: 820, y: 180, w: 420, h: 520 }, { x: 260, y: 600, w: 560, h: 360 },
        { x: 1400, y: 90, w: 420, h: 300 }, { x: 680, y: 700, w: 360, h: 260 },
        { x: 40, y: 40, w: 360, h: 260 },
      ];
      return [big, ...small.slice(0, n - 1)];
    }
    return floating.slice(0, n);
  }

  // Where windows, the slab and the strip settle for `p`, on `windows`.
  function targets(p, windows) {
    const stripH = { compact: Math.min(Math.max(H * 0.07, 44), 96), names: 36, thumbs: H * 0.09, bottom: 0, none: 0 }[p.strip] ?? 0;
    const stripBlock = p.strip === 'compact' ? 18 + stripH + 26 + 22 : p.strip === 'none' || p.strip === 'bottom' ? p.margin : 16 + stripH + 24;
    const bottomBlock = p.strip === 'bottom' ? H * 0.16 : p.margin;
    const scene = { x: p.margin, y: Math.max(stripBlock, p.margin), w: W - 2 * p.margin, h: 0 };
    scene.h = H - bottomBlock - scene.y;
    const pad = p.wallpaper === 'slab' ? p.padding : 0;
    const room = { x: scene.x + pad, y: scene.y + pad, w: scene.w - 2 * pad, h: scene.h - 2 * pad };
    const fit = Math.min(room.w / WORK.w, room.h / WORK.h);
    let slab = {
      x: room.x + (room.w - WORK.w * fit) / 2 - pad, y: room.y + (room.h - WORK.h * fit) / 2 - pad,
      w: WORK.w * fit + 2 * pad, h: WORK.h * fit + 2 * pad,
    };
    let rects;
    if (p.layout === 'rigid') {
      const z = p.zoom;
      const ws = { x: (W - WORK.w * z) / 2, y: (H - WORK.h * z) / 2, w: WORK.w * z, h: WORK.h * z };
      rects = windows.map((r) => ({ x: ws.x + r.x * z, y: ws.y + r.y * z, w: r.w * z, h: r.h * z }));
      slab = ws;
    } else if (p.layout === 'rows') {
      rects = L.rows(windows, room, { spacing: p.gap, maxScale: p.rowsMax ?? p.maxScale, monitorH: WORK.h });
    } else if (p.layout === 'grid') {
      rects = L.grid(windows, room, { gap: p.gap, maxScale: p.maxScale });
    } else {
      rects = L.cluster(windows, room, { gap: p.gap, largest: p.largest, maxScale: p.maxScale });
    }
    if (p.wallpaper !== 'slab') slab = { x: 0, y: 0, w: W, h: H };
    return { rects, slab, room, stripH, stripY: p.strip === 'bottom' ? H - bottomBlock + 24 : 18 };
  }

  // One animated value, on a spring or a bezier, retargetable mid-flight.
  class Motion {
    constructor(value = 0) { this.x = value; this.v = 0; this.to = value; this.delay = 0; this.anim = null; }
    aim(to, anim, delay = 0) {
      this.to = to; this.delay = delay; this.anim = anim;
      if (anim.kind === 'bezier') { this.from = this.x; this.t = 0; this.curve = L.cubicBezier(...anim.curve); }
    }
    step(dt) {
      if (this.delay > 0) { this.delay -= dt; if (this.delay > 0) return true; dt = -this.delay; this.delay = 0; }
      const a = this.anim;
      if (!a) return false;
      if (a.kind === 'bezier') {
        this.t += dt;
        const u = a.ms <= 0 ? 1 : Math.min(1, this.t / (a.ms / 1000));
        this.x = this.from + (this.to - this.from) * this.curve(u);
        if (u >= 1) { this.x = this.to; this.anim = null; return false; }
        return true;
      }
      const k = a.stiffness, c = 2 * a.damping * Math.sqrt(k);
      const steps = Math.max(1, Math.ceil(dt / 0.001)), h = dt / steps;
      for (let i = 0; i < steps; i++) {
        this.v += (-k * (this.x - this.to) - c * this.v) * h;
        this.x += this.v * h;
      }
      if (Math.abs(this.x - this.to) < 1e-4 && Math.abs(this.v) < 1e-3) { this.x = this.to; this.v = 0; this.anim = null; return false; }
      return true;
    }
  }

  const lerp = (a, b, t) => a + (b - a) * t;
  const lerpRect = (a, b, t) => ({ x: lerp(a.x, b.x, t), y: lerp(a.y, b.y, t), w: lerp(a.w, b.w, t), h: lerp(a.h, b.h, t) });
  const smooth = (e) => e * e * (3 - 2 * e);

  function windowBody(app) {
    const body = document.createElement('div');
    body.className = 'ovs-body ovs-' + app.kind;
    const bar = (w, c, extra = '') => `<i style="width:${w}%;background:${c};${extra}"></i>`;
    if (app.kind === 'editor') {
      const cols = ['#7aa2f7', '#9ece6a', '#bb9af7', '#e0af68', '#7dcfff', '#565f89'];
      let s = '<div class="ovs-gutter"></div><div class="ovs-code">';
      for (let i = 0; i < 26; i++) s += `<p style="padding-left:${(i * 7) % 4 * 6}%">${bar(18 + (i * 37) % 50, cols[i % 6])}${bar(8 + (i * 13) % 20, cols[(i + 2) % 6])}</p>`;
      body.innerHTML = s + '</div>';
    } else if (app.kind === 'browser') {
      body.innerHTML = `<div class="ovs-url">${bar(46, 'rgba(0,0,0,.12)', 'height:100%;border-radius:99px')}</div>
        <div class="ovs-page"><div class="ovs-hero" style="background:${app.tint}"></div>
        <p>${bar(70, '#c9ccd6')}</p><p>${bar(85, '#d7d9e0')}</p><p>${bar(60, '#d7d9e0')}</p>
        <div class="ovs-cards"><b></b><b></b><b></b></div></div>`;
    } else if (app.kind === 'terminal') {
      let s = '';
      for (let i = 0; i < 22; i++) s += `<p>${bar(4, '#22c55e')}${bar(10 + (i * 29) % 60, i % 5 === 4 ? '#f7768e' : '#a9b1d6')}</p>`;
      body.innerHTML = s;
    } else if (app.kind === 'chat') {
      let s = '<div class="ovs-side"></div><div class="ovs-msgs">';
      for (let i = 0; i < 9; i++) s += `<p class="${i % 3 === 2 ? 'me' : ''}"><b style="width:${30 + (i * 23) % 45}%"></b></p>`;
      body.innerHTML = s + '</div>';
    } else if (app.kind === 'music') {
      body.innerHTML = `<div class="ovs-art" style="background:linear-gradient(135deg,${app.tint},#1e1b4b)"></div>
        <div class="ovs-track">${bar(60, '#e5e7eb')}${bar(40, '#9ca3af')}<span>${bar(100, '#374151', 'height:100%')}</span></div>`;
    } else if (app.kind === 'files') {
      let s = '';
      for (let i = 0; i < 18; i++) s += `<b style="background:${['#eab308', '#60a5fa', '#a3a3a3', '#f87171'][i % 4]}"></b>`;
      body.innerHTML = `<div class="ovs-grid">${s}</div>`;
    } else {
      let s = '';
      for (let i = 0; i < 16; i++) s += `<p>${bar(i % 5 === 0 ? 40 : 60 + (i * 17) % 35, i % 5 === 0 ? '#14b8a6' : '#cbd5e1')}</p>`;
      body.innerHTML = s;
    }
    return body;
  }

  const CSS = `
  .ovs{position:relative;overflow:hidden;aspect-ratio:16/9;width:100%;max-width:100%;border-radius:10px;background:var(--ovs-surround,#141416);user-select:none;-webkit-user-select:none;contain:layout paint;isolation:isolate;cursor:default}
  .ovs *{box-sizing:border-box}
  .ovs-layer{position:absolute;left:0;top:0;transform-origin:0 0;will-change:transform}
  .ovs-wall{position:absolute;inset:0;background:
    radial-gradient(60% 80% at 18% 22%,#3c5d8f 0%,transparent 60%),
    radial-gradient(55% 70% at 85% 75%,#c2643f 0%,transparent 62%),
    radial-gradient(40% 50% at 60% 30%,#6f8fb8 0%,transparent 70%),
    linear-gradient(160deg,#1d2b45 0%,#29324a 45%,#4a3340 100%)}
  .ovs-wall.blurred{filter:blur(18px) saturate(1.15);transform:scale(1.08)}
  .ovs-slab{position:absolute;overflow:hidden;will-change:transform,border-radius}
  .ovs-shadow{position:absolute;will-change:transform,opacity;box-shadow:0 18px 60px rgba(0,0,0,.55)}
  .ovs-dimmer{position:absolute;inset:0;background:#000;pointer-events:none}
  .ovs-task{position:absolute;left:0;right:0;bottom:0;background:rgba(20,22,30,.82);display:flex;align-items:center;justify-content:center;gap:1.1%}
  .ovs-task b{width:1.4%;aspect-ratio:1;border-radius:20%;opacity:.9}
  .ovs-win{position:absolute;left:0;top:0;transform-origin:0 0;will-change:transform,opacity;border-radius:var(--ovs-wr,6px);overflow:visible}
  .ovs-frame{position:absolute;inset:0;border-radius:inherit;overflow:hidden;background:#f6f7fa;box-shadow:0 6px 26px rgba(0,0,0,.35),0 0 0 1px rgba(255,255,255,.06)}
  .ovs-title{height:34px;display:flex;align-items:center;gap:10px;padding:0 12px;background:#e9ebf0;color:#222;font:500 14px/1 system-ui,sans-serif;white-space:nowrap;overflow:hidden}
  .ovs-dark .ovs-title{background:#1f2230;color:#c0c4d6}
  .ovs-title .ico{width:16px;height:16px;border-radius:4px;flex:none}
  .ovs-title .ctl{margin-left:auto;letter-spacing:18px;opacity:.55}
  .ovs-body{position:absolute;top:34px;left:0;right:0;bottom:0;overflow:hidden}
  .ovs-body i{display:inline-block;height:9px;border-radius:5px;margin-right:2%}
  .ovs-body p{margin:0 0 11px}
  .ovs-editor{background:#1a1b26;display:flex}.ovs-gutter{width:5%;background:#16161e}.ovs-code{padding:18px 3%;flex:1}
  .ovs-url{height:30px;margin:10px 12px;padding:0 0}
  .ovs-page{padding:0 6%}.ovs-hero{height:30%;border-radius:10px;margin:8px 0 22px;opacity:.85}
  .ovs-cards{display:flex;gap:4%;margin-top:20px}.ovs-cards b{flex:1;height:120px;background:#e4e6ec;border-radius:10px}
  .ovs-terminal{background:#0f111a;padding:18px 3%}
  .ovs-chat{display:flex;background:#f3f1f8}.ovs-side{width:22%;background:#2b2440}.ovs-msgs{flex:1;padding:20px 4%}
  .ovs-msgs p{display:flex}.ovs-msgs p b{height:36px;border-radius:14px;background:#ddd6f0}.ovs-msgs p.me{justify-content:flex-end}.ovs-msgs p.me b{background:#a855f7;opacity:.75}
  .ovs-music{background:#111827;display:flex;gap:5%;padding:6%}.ovs-art{width:38%;aspect-ratio:1;border-radius:12px;align-self:flex-start}.ovs-track{flex:1;padding-top:4%}.ovs-track i{display:block;height:16px;margin-bottom:16px}.ovs-track span{display:block;height:6px;margin-top:40px;border-radius:3px;overflow:hidden}
  .ovs-files .ovs-grid{display:grid;grid-template-columns:repeat(6,1fr);gap:22px;padding:28px}.ovs-grid b{aspect-ratio:1.1;border-radius:8px;opacity:.85}
  .ovs-notes{background:#fbfaf7;padding:26px 6%}
  .ovs-ring{position:absolute;inset:calc(-1 * var(--ovs-ring-gap,0px));border-radius:inherit;box-shadow:0 0 0 var(--ovs-ring-w,5px) var(--ovs-accent,#5aa9ff);opacity:0;pointer-events:none}
  .ovs-icon{position:absolute;left:50%;top:50%;width:84px;height:84px;margin:-42px 0 0 -42px;border-radius:22px;opacity:0;box-shadow:0 8px 24px rgba(0,0,0,.35);pointer-events:none}
  .ovs-tag{position:absolute;left:50%;top:100%;transform:translate(-50%,0);margin-top:14px;padding:7px 14px;border-radius:9px;background:rgba(18,18,24,.86);color:#f2f2f5;font:500 26px/1.2 system-ui,sans-serif;white-space:nowrap;opacity:0;pointer-events:none}
  .ovs-win.notag .ovs-tag{display:none}
  .ovs-deco{position:absolute;inset:0;pointer-events:none;z-index:3}
  .ovs-strip{position:absolute;left:0;right:0;display:flex;justify-content:center;gap:16px;pointer-events:none}
  .ovs-mini{position:relative;border-radius:8px;overflow:hidden;background:rgba(255,255,255,.08);box-shadow:0 0 0 2px rgba(255,255,255,.08)}
  .ovs-mini.on{box-shadow:0 0 0 3px var(--ovs-accent,#5aa9ff)}
  .ovs-mini u{position:absolute;background:rgba(255,255,255,.55);border-radius:2px}
  .ovs-mini em{position:absolute;left:0;right:0;top:calc(100% + 6px);text-align:center;color:#e7e7ee;font:500 18px/1 system-ui,sans-serif;font-style:normal}
  .ovs-name{padding:8px 18px;border-radius:10px;color:#eee;font:500 20px/1 system-ui,sans-serif;background:rgba(255,255,255,.08)}
  .ovs-name.on{background:rgba(255,255,255,.22)}
  .ovs-hint{position:absolute;left:50%;bottom:4%;transform:translateX(-50%);padding:10px 20px;border-radius:12px;background:rgba(0,0,0,.45);color:#fff;font:500 24px/1 system-ui,sans-serif;opacity:0;pointer-events:none;white-space:nowrap}
  .ovs-cursor{position:absolute;left:0;top:0;width:34px;height:46px;z-index:100;pointer-events:none;transition:transform var(--ovs-cur-ms,600ms) cubic-bezier(.3,.7,.2,1),opacity .2s;opacity:0;filter:drop-shadow(0 3px 4px rgba(0,0,0,.45))}
  .ovs-cursor.down{scale:.86}
  .ovs-ripple{position:absolute;width:40px;height:40px;margin:-20px 0 0 -20px;border-radius:50%;border:5px solid var(--ovs-accent,#5aa9ff);pointer-events:none;z-index:90;animation:ovs-ripple .7s cubic-bezier(.2,.7,.3,1) forwards}
  @keyframes ovs-ripple{from{transform:scale(.4);opacity:.9}to{transform:scale(9);opacity:0}}
  .ovs-search{position:absolute;left:50%;top:var(--ovs-search-top,40px);transform:translateX(-50%);min-width:420px;padding:16px 24px;border-radius:18px;background:rgba(16,17,24,.88);color:#f2f2f5;font:500 30px/1 system-ui,sans-serif;box-shadow:0 12px 40px rgba(0,0,0,.45),0 0 0 2px rgba(255,255,255,.08);z-index:80;opacity:0;pointer-events:none;display:flex;gap:14px;align-items:center;white-space:nowrap}
  .ovs-search .q::after{content:"";display:inline-block;width:3px;height:30px;background:var(--ovs-accent,#5aa9ff);margin-left:3px;vertical-align:-5px;animation:ovs-blink 1s steps(1) infinite}
  .ovs-search .n{margin-left:auto;font-size:22px;opacity:.6}
  @keyframes ovs-blink{50%{opacity:0}}
  @media (prefers-reduced-motion: reduce){.ovs-win,.ovs-layer{transition:none}}
  `;
  let cssInjected = false;

  class Stage {
    constructor(el, opts = {}) {
      if (!cssInjected) { const s = document.createElement('style'); s.textContent = CSS; document.head.appendChild(s); cssInjected = true; }
      this.el = el; this.el.classList.add('ovs');
      this.params = { ...PRESETS.glazewm, ...(opts.params || {}) };
      this.windows = opts.windows || scenario(4, 'tiled');
      this.apps = opts.apps || null;
      this.minimized = new Set(opts.minimized || []);
      this.titles = opts.titles || null;
      this.layoutHook = opts.layout || null;
      this.onChange = opts.onChange || (() => {});
      this.interactive = opts.interactive !== false;
      this.openness = 0; this.isOpen = false; this.picked = -1; this.hover = -1;
      this.world = document.createElement('div'); this.world.className = 'ovs-layer';
      this.world.style.width = W + 'px'; this.world.style.height = H + 'px';
      this.el.appendChild(this.world);
      this.slabMotion = new Motion(0);
      this.build();
      new ResizeObserver(() => this.fit()).observe(this.el);
      this.fit();
      if (this.interactive) this.bindInput();
      this.last = 0; this.running = false;
      if (opts.open) { this.isOpen = true; this.retarget(true); }
      this.render();
    }

    setParams(params) { this.params = { ...this.params, ...params }; this.build(); this.retarget(true); this.render(); }
    setWindows(windows) { this.windows = windows; this.build(); this.retarget(true); this.render(); }

    fit() {
      const k = this.el.clientWidth / W;
      this.k = k;
      this.world.style.transform = `scale(${k})`;
    }

    build() {
      const p = this.params;
      this.el.style.setProperty('--ovs-surround', p.surround || (p.wallpaper === 'slab' ? '#141416' : '#000'));
      this.world.innerHTML = '';
      this.wallBox = document.createElement('div'); this.wallBox.className = 'ovs-layer'; this.wallBox.style.width = W + 'px'; this.wallBox.style.height = H + 'px';
      this.shadow = document.createElement('div'); this.shadow.className = 'ovs-shadow';
      this.slab = document.createElement('div'); this.slab.className = 'ovs-slab';
      this.slab.style.width = W + 'px'; this.slab.style.height = H + 'px'; this.slab.style.left = '0'; this.slab.style.top = '0'; this.slab.style.transformOrigin = '0 0';
      this.wall = document.createElement('div'); this.wall.className = 'ovs-wall';
      this.blurWall = document.createElement('div'); this.blurWall.className = 'ovs-wall blurred'; this.blurWall.style.opacity = 0;
      this.dimmer = document.createElement('div'); this.dimmer.className = 'ovs-dimmer'; this.dimmer.style.opacity = 0;
      this.task = document.createElement('div'); this.task.className = 'ovs-task'; this.task.style.height = TASKBAR + 'px';
      this.task.innerHTML = APPS.slice(0, 7).map((a) => `<b style="background:${a.tint}"></b>`).join('');
      this.slab.append(this.wall, this.blurWall, this.dimmer, this.task);
      if (p.wallpaper === 'slab') this.world.append(this.shadow);
      this.world.append(this.slab);

      // The strip.
      this.strip = document.createElement('div'); this.strip.className = 'ovs-strip';
      this.world.append(this.strip);
      const t = targets(p, this.windows);
      this.t = t;
      this.strip.style.top = t.stripY + 'px';
      if (p.strip === 'compact' || p.strip === 'thumbs' || p.strip === 'bottom') {
        const h = p.strip === 'bottom' ? 120 : t.stripH, w = h * 16 / 9;
        for (let i = 0; i < 4; i++) {
          const m = document.createElement('div'); m.className = 'ovs-mini' + (i === 0 ? ' on' : '');
          m.style.width = w + 'px'; m.style.height = h + 'px';
          const rects = i === 0 ? this.windows : scenario(1 + ((i * 3) % 4), i % 2 ? 'tiled' : 'floating');
          if (i < 3) for (const r of rects) {
            const u = document.createElement('u');
            Object.assign(u.style, { left: r.x / W * 100 + '%', top: r.y / H * 100 + '%', width: r.w / W * 100 + '%', height: r.h / H * 100 + '%' });
            m.append(u);
          }
          if (p.strip !== 'thumbs') { const em = document.createElement('em'); em.textContent = i < 3 ? String(i + 1) : '+'; m.append(em); }
          this.strip.append(m);
        }
      } else if (p.strip === 'names') {
        ['Desktop 1', 'Desktop 2', 'Desktop 3', '+'].forEach((n, i) => {
          const d = document.createElement('div'); d.className = 'ovs-name' + (i === 0 ? ' on' : ''); d.textContent = n; this.strip.append(d);
        });
      }

      // Windows.
      this.wins = this.windows.map((r, i) => {
        const app = APPS[(this.apps ? this.apps[i] : i) % APPS.length];
        const el = document.createElement('div'); el.className = 'ovs-win' + (['editor', 'terminal', 'music'].includes(app.kind) ? ' ovs-dark' : '');
        el.style.width = r.w + 'px'; el.style.height = r.h + 'px';
        el.style.setProperty('--ovs-wr', (8 / Math.max(0.3, (t.rects[i]?.w ?? r.w) / r.w)) + 'px');
        const frame = document.createElement('div'); frame.className = 'ovs-frame';
        frame.innerHTML = `<div class="ovs-title"><span class="ico" style="background:${app.tint}"></span>${this.titles?.[i] ?? app.title}<span class="ctl">—▢✕</span></div>`;
        frame.append(windowBody(app));
        const ring = document.createElement('div'); ring.className = 'ovs-ring';
        const icon = document.createElement('div'); icon.className = 'ovs-icon'; icon.style.background = app.tint;
        const tag = document.createElement('div'); tag.className = 'ovs-tag'; tag.textContent = this.titles?.[i] ?? app.title;
        const deco = document.createElement('div'); deco.className = 'ovs-deco';
        el.append(frame, ring, icon, tag, deco);
        el.dataset.index = i;
        this.world.append(el);
        return { el, ring, icon, tag, deco, app, real: r, motion: new Motion(this.openness), hover: new Motion(0), fade: new Motion(1), scale: 1 };
      });
      this.hint = document.createElement('div'); this.hint.className = 'ovs-hint'; this.world.append(this.hint);
      this.cursor = document.createElement('div'); this.cursor.className = 'ovs-cursor';
      this.cursor.innerHTML = '<svg viewBox="0 0 24 32" width="34" height="46"><path d="M2 2v24l6.5-6 4.5 10 4-1.8-4.4-9.8H22z" fill="#fff" stroke="#111" stroke-width="1.6" stroke-linejoin="round"/></svg>';
      this.search = document.createElement('div'); this.search.className = 'ovs-search';
      this.search.innerHTML = '<span style="opacity:.55">⌕</span><span class="q"></span><span class="n"></span>';
      this.world.append(this.cursor, this.search);
      this.slabMotion = new Motion(this.openness);
      this.chrome = 0;
    }

    retarget(snap) {
      this.t = this.computeTargets();
      this.slabFrom = null; this.wins.forEach((w) => { w.morphFrom = null; });
      if (snap) {
        const v = this.isOpen ? 1 : 0;
        this.slabMotion = new Motion(v);
        this.wins.forEach((w) => { w.motion = new Motion(v); });
      }
    }

    // Delays for the open: nearest the middle (or the focused window) first.
    staggerDelays() {
      const p = this.params, n = this.wins.length;
      if (!p.staggerMs || n < 2) return this.wins.map(() => 0);
      const origin = this.windows[0];
      const oc = p.staggerFrom || [origin.x + origin.w / 2, origin.y + origin.h / 2];
      const order = this.windows.map((r, i) => [Math.hypot(r.x + r.w / 2 - oc[0], r.y + r.h / 2 - oc[1]), i]).sort((a, b) => a[0] - b[0]);
      const within = p.staggerWithinMs ?? 60;
      const step = Math.min(p.staggerMs, within / (n - 1));
      const d = new Array(n);
      order.forEach(([, i], rank) => { d[i] = (rank * step) / 1000; });
      return d;
    }

    open() {
      if (this.isOpen) return;
      this.isOpen = true; this.picked = -1;
      this.t = this.computeTargets();
      const delays = this.staggerDelays();
      this.slabMotion.aim(1, this.params.open);
      this.wins.forEach((w, i) => w.motion.aim(1, this.params.open, delays[i]));
      this.startedAt = performance.now(); this.kick(); this.onChange('open');
    }

    close(pick = -1) {
      if (!this.isOpen) return;
      this.isOpen = false; this.picked = pick; this.setHover(-1);
      this.wins.forEach((w) => { if (w.fade.x < 1 || w.fade.to < 1) w.fade.aim(1, this.params.close); });
      if (this.wins.some((w) => w.morphFrom)) {
        const base = this.computeTargets().rects;
        this.wins.forEach((w, i) => { w.morphFrom = this.targetOf(w, i); w.morphM = new Motion(0); w.morphM.aim(1, this.params.close); });
        this.t = { ...this.t, rects: base };
      }
      this.slabMotion.aim(0, this.params.close);
      this.wins.forEach((w) => w.motion.aim(0, this.params.close));
      this.startedAt = performance.now(); this.kick(); this.onChange('close');
    }

    toggle() { this.isOpen ? this.close() : this.open(); }

    // Where everything settles: the preset's layout, or the page's own.
    computeTargets() {
      const t = targets(this.params, this.windows);
      if (this.layoutHook) t.rects = this.layoutHook(t, this);
      return t;
    }

    // Re-runs the layout and moves to it from wherever things are.
    relayout(anim, staggerMs = 0) { this.morphRects(this.computeTargets().rects, anim, staggerMs); }

    // Moves to `params`' layout from wherever things are now, on `anim`,
    // each window `staggerMs` after the one before.
    morph(params, anim = { kind: 'spring', stiffness: 420, damping: 0.82 }, staggerMs = 18) {
      const before = this.wins.map((w, i) => this.targetOf(w, i));
      const slabBefore = this.slabTarget();
      this.params = { ...this.params, ...params };
      this.t = this.computeTargets();
      this.wins.forEach((w, i) => {
        w.morphFrom = before[i]; w.morphM = new Motion(0); w.morphM.aim(1, anim, (i * staggerMs) / 1000);
      });
      this.slabFrom = slabBefore; this.slabM = new Motion(0); this.slabM.aim(1, anim);
      this.kick();
    }

    targetOf(w, i) {
      const to = this.t.rects[i] || w.real;
      return w.morphFrom && w.morphM ? lerpRect(w.morphFrom, to, w.morphM.x) : to;
    }

    slabTarget() {
      return this.slabFrom && this.slabM ? lerpRect(this.slabFrom, this.t.slab, this.slabM.x) : this.t.slab;
    }

    setHover(i) {
      if (i === this.hover) return;
      this.hover = i;
      const anim = { kind: 'spring', stiffness: 700, damping: 0.8 };
      this.wins.forEach((w, j) => w.hover.aim(j === i ? 1 : 0, anim));
      this.kick();
    }

    bindInput() {
      const toWorld = (e) => { const b = this.el.getBoundingClientRect(); return [(e.clientX - b.left) / this.k, (e.clientY - b.top) / this.k]; };
      const hit = (x, y) => {
        for (let i = this.wins.length - 1; i >= 0; i--) {
          const r = this.t.rects[i];
          if (r && x >= r.x && x <= r.x + r.w && y >= r.y && y <= r.y + r.h) return i;
        }
        return -1;
      };
      this.el.addEventListener('pointermove', (e) => {
        if (!this.isOpen || this.openness < 0.6) return;
        const [x, y] = toWorld(e); this.setHover(hit(x, y));
      });
      this.el.addEventListener('pointerleave', () => this.setHover(-1));
      this.el.addEventListener('click', (e) => {
        if (!this.isOpen) { this.open(); return; }
        const [x, y] = toWorld(e); this.close(hit(x, y));
      });
    }

    // Sends each window to `rects[i]` from wherever it shows now.
    morphRects(rects, anim = { kind: 'spring', stiffness: 420, damping: 0.82 }, staggerMs = 0) {
      const before = this.wins.map((w, i) => this.targetOf(w, i));
      this.t = { ...this.t, rects: rects.map((r) => ({ ...r })) };
      this.wins.forEach((w, i) => {
        w.morphFrom = before[i]; w.morphM = new Motion(0); w.morphM.aim(1, anim, (i * staggerMs) / 1000);
      });
      this.kick();
    }

    fadeWindow(i, to, anim = { kind: 'spring', stiffness: 380, damping: 1 }) { this.wins[i].fade.aim(to, anim); this.kick(); }

    showCursor(x, y, ms = 600) {
      this.cursor.style.setProperty('--ovs-cur-ms', ms + 'ms');
      this.cursor.style.opacity = 1; this.cursor.style.transform = `translate(${x}px,${y}px)`;
    }
    hideCursor() { this.cursor.style.opacity = 0; }
    press() { this.cursor.classList.add('down'); setTimeout(() => this.cursor.classList.remove('down'), 140); }

    ripple(x, y) {
      const r = document.createElement('div'); r.className = 'ovs-ripple'; r.style.left = x + 'px'; r.style.top = y + 'px';
      this.world.append(r); setTimeout(() => r.remove(), 800);
    }

    setSearch(query, count) {
      this.search.style.transition = 'opacity .18s, transform .25s cubic-bezier(.3,.7,.2,1)';
      this.search.style.opacity = query == null ? 0 : 1;
      this.search.style.transform = `translateX(-50%) translateY(${query == null ? -16 : 0}px)`;
      this.search.querySelector('.q').textContent = query || '';
      this.search.querySelector('.n').textContent = count == null ? '' : count;
    }

    // Middle of window `i` where it rests now, in world pixels.
    centerOf(i) { const r = this.targetOf(this.wins[i], i); return [r.x + r.w / 2, r.y + r.h / 2]; }

    kick() {
      if (this.running) return;
      this.running = true; this.last = performance.now();
      const frame = (now) => {
        const dt = Math.min(0.05, (now - this.last) / 1000); this.last = now;
        let busy = this.slabMotion.step(dt);
        if (this.slabM) busy = this.slabM.step(dt) || busy;
        for (const w of this.wins) {
          busy = w.motion.step(dt) || busy; busy = w.hover.step(dt) || busy;
          if (w.morphM) busy = w.morphM.step(dt) || busy;
          busy = w.fade.step(dt) || busy;
        }
        this.render();
        if (busy) requestAnimationFrame(frame); else { this.running = false; this.onChange(this.isOpen ? 'opened' : 'closed'); }
      };
      requestAnimationFrame(frame);
    }

    render() {
      const p = this.params, t = this.t;
      const s = this.slabMotion.x;
      this.openness = s;
      const full = { x: 0, y: 0, w: W, h: H };
      const slab = lerpRect(full, this.slabTarget(), s);
      const sk = slab.w / W;
      this.slab.style.transform = `translate(${slab.x}px,${slab.y}px) scale(${sk},${slab.h / H})`;
      const radius = p.wallpaper === 'slab' ? lerp(0, p.radius, Math.min(1, Math.max(0, s))) : 0;
      this.slab.style.borderRadius = radius / Math.max(sk, 0.01) + 'px';
      if (p.wallpaper === 'slab') {
        Object.assign(this.shadow.style, { left: slab.x + 'px', top: slab.y + 'px', width: slab.w + 'px', height: slab.h + 'px', borderRadius: radius + 'px', opacity: Math.min(1, Math.max(0, s)) });
      }
      const sc = Math.min(1, Math.max(0, s));
      this.blurWall.style.opacity = p.wallpaper === 'blur' ? sc : 0;
      this.dimmer.style.opacity = p.wallpaper === 'blur' ? sc * 0.45 : p.wallpaper === 'dim' ? sc * 0.28 : 0;
      this.task.style.opacity = 1 - sc;
      const from = p.chromeFrom ?? 0.8;
      const chrome = smooth(Math.min(1, Math.max(0, (s - from) / (1 - from))));
      this.chrome = chrome;
      this.strip.style.opacity = chrome;
      this.strip.style.transform = `translateY(${(1 - chrome) * -14}px)`;

      const anyHover = this.hover >= 0;
      this.wins.forEach((w, i) => {
        const target = this.targetOf(w, i);
        const m = w.motion.x;
        let r = lerpRect(w.real, target, m);
        let appear = this.minimized.has(i) ? Math.max(0, Math.min(1, m)) : 1;
        if (p.motion === 'fade') {
          if (m < 0.5) { r = w.real; appear *= this.minimized.has(i) ? 0 : 1 - 2 * Math.max(m, 0); }
          else { const g = 1 + 0.06 * (1 - m) * 2; r = { x: target.x - target.w * (g - 1) / 2, y: target.y - target.h * (g - 1) / 2, w: target.w * g, h: target.h * g }; appear *= Math.min(1, 2 * m - 1); }
        }
        const lift = 1 + (p.lift || 0) * w.hover.x;
        const k = r.w / w.real.w * lift;
        const cx = r.x + r.w / 2, cy = r.y + r.h / 2;
        const x = cx - (w.real.w * k) / 2, y = cy - (w.real.h * k) / 2;
        w.el.style.transform = `translate(${x}px,${y}px) scale(${k})`;
        w.el.style.zIndex = i === this.picked ? 50 : i === this.hover ? 40 : 10 + i;
        const dimmed = anyHover && i !== this.hover ? (p.dim || 0) * chrome : 0;
        w.el.style.opacity = appear * (1 - dimmed) * Math.max(0, Math.min(1, w.fade.x));
        w.el.style.setProperty('--inv', 1 / Math.max(k, 0.05));
        w.el.style.setProperty('--chrome', chrome);
        w.el.style.setProperty('--hov', Math.max(0, Math.min(1, w.hover.x)));
        const ringW = (p.ringWidth ?? 2.5) / Math.max(k * this.k, 0.01);
        w.ring.style.setProperty('--ovs-ring-w', ringW + 'px');
        w.ring.style.opacity = Math.max(w.hover.x, 0) * chrome;
        w.tag.style.opacity = (p.titles === 'always' ? 1 : p.titles === 'never' ? 0 : Math.max(w.hover.x, 0)) * chrome;
        w.tag.style.transform = `translate(-50%,0) scale(${1 / Math.max(k, 0.05)})`;
        w.tag.style.transformOrigin = '50% 0';
        w.icon.style.opacity = p.icons ? chrome : 0;
        w.icon.style.transform = `scale(${1 / Math.max(k, 0.05)})`;
      });
    }

    // Final scale of each window, and how much of the slab they fill.
    readout() {
      const t = this.computeTargets();
      const scales = t.rects.map((r, i) => r.w / this.windows[i].w);
      const b = L.bounds(t.rects);
      return { scales, slab: t.slab.w / W, cluster: b, fill: Math.max(b.w / t.slab.w, b.h / t.slab.h) };
    }
  }

  return { Stage, PRESETS, scenario, targets, Motion, APPS, W, H, WORK };
})();

if (typeof module !== 'undefined') module.exports = OverviewStage;
