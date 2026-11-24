// The config builder's model: every option it sets, where windows go for a
// layout, easing curves as GlazeWM plays them, and the YAML it writes.
// Pure, so `node builder.test.js` checks it without a browser.

const Builder = (() => {
  const CURVES = ['spring', 'linear', 'ease_in', 'ease_out', 'ease_in_out', 'ease_in_cubic', 'ease_out_cubic', 'ease_in_out_cubic', 'ease_out_spring', 'snappy'];
  const OPEN_STYLES = ['zoom', 'slide_right', 'slide_left', 'slide_top', 'slide_bottom', 'none'];

  // Option kinds: bool, enum (`options`), px / pct / num ranges, color
  // ('#rrggbb' or '#rrggbbaa'; `optional` allows unset), text, list
  // (comma-separated, written as a YAML list), easing. `np`: the preview
  // doesn't show it. Keys starting with `_` are the builder's own and never
  // written; `onlyIf` writes an option only while that key is on.
  const tab = (id, label, intro, groups) => ({ id, label, intro, groups });
  const group = (title, items) => ({ title, items });

  function windowLook(prefix, focused) {
    const p = `window_effects.${prefix}`;
    return [
      group(`${focused ? 'Focused window' : 'Other windows'}: border`, [
        { key: `${p}.border.enabled`, kind: 'bool', def: true },
        { key: `${p}.border.color`, kind: 'color', def: focused ? '#8dbcff' : '#a1a1a1', help: 'Or "accent" for the OS accent color, typed in.' },
        { key: `${p}.border.width`, kind: 'px', min: 0, max: 12, def: 2 },
        { key: `${p}.border.radius`, kind: 'px', min: 0, max: 30, def: null, optional: true, help: 'Unset: the window\'s corner radius plus the width.' },
        { key: `${p}.border.highlight`, kind: 'num', min: 0, max: 1, step: 0.05, def: 0, help: 'Lightens the top edge, like lit glass.' },
        { key: `${p}.border.rim`, kind: 'num', min: 0, max: 1, step: 0.05, def: 0, help: 'A bright hairline along the inner edge.' },
        { key: `${p}.border.glow.enabled`, kind: 'bool', def: false },
        { key: `${p}.border.glow.reach`, kind: 'px', min: 0, max: 40, def: 10 },
        { key: `${p}.border.glow.opacity`, kind: 'num', min: 0, max: 1, step: 0.05, def: 0.5 },
      ]),
      group(`${focused ? 'Focused window' : 'Other windows'}: window`, [
        { key: `${p}.hide_title_bar.enabled`, kind: 'bool', def: false, help: 'Can break rendering in some apps.' },
        { key: `${p}.corner_style.enabled`, kind: 'bool', def: true, help: 'Windows 11 only.' },
        { key: `${p}.corner_style.style`, kind: 'enum', options: ['square', 'rounded', 'small_rounded'], def: 'square' },
        { key: `${p}.corner_radius`, kind: 'px', min: 0, max: 30, def: null, optional: true, help: 'Overrides the corner style, for the window, its border and backdrop.' },
        { key: `${p}.transparency.enabled`, kind: 'bool', def: false },
        { key: `${p}.transparency.opacity`, kind: 'pct', min: 10, max: 100, def: focused ? 95 : 90 },
      ]),
      group(`${focused ? 'Focused window' : 'Other windows'}: backdrop`, [
        { key: `${p}.backdrop.enabled`, kind: 'bool', def: false, help: 'Blurred wallpaper behind the window. Windows 10 1803+.' },
        { key: `${p}.backdrop.tint`, kind: 'color', def: focused ? '#00000030' : '#00000060' },
        { key: `${p}.backdrop.blur_amount`, kind: 'num', min: 0, max: 100, step: 1, def: 30 },
        { key: `${p}.backdrop.saturation`, kind: 'num', min: 0, max: 2, step: 0.05, def: 1.2 },
        { key: `${p}.backdrop.exposure`, kind: 'num', min: -2, max: 2, step: 0.1, def: 0, help: 'In stops. Negative darkens.' },
        { key: `${p}.backdrop.vignette`, kind: 'num', min: 0, max: 1, step: 0.05, def: focused ? 0.2 : 0.4 },
        { key: `${p}.backdrop.grain`, kind: 'num', min: 0, max: 1, step: 0.01, def: 0.05, np: true },
      ]),
    ];
  }

  function animation(name, label, def, extra = []) {
    const p = `animations.${name}`;
    return group(label, [
      { key: `${p}.enabled`, kind: 'bool', def: def.enabled ?? true },
      { key: `${p}.duration_ms`, kind: 'num', min: 0, max: 1000, step: 10, def: def.duration_ms, help: 'A spring lasts until it settles, whatever this says.' },
      { key: `${p}.easing`, kind: 'easing', def: def.easing },
      ...extra.map((item) => ({ ...item, key: `${p}.${item.key}` })),
    ]);
  }

  const TABS = [
    tab('general', 'General', 'How GlazeWM behaves. Little of it shows in a picture.', [
      group('Focus', [
        { key: 'general.focus_follows_cursor', kind: 'bool', def: false, np: true, help: 'Focus the window under the cursor as it moves.' },
        { key: 'general.toggle_workspace_on_refocus', kind: 'bool', def: false, np: true, help: 'Focusing the current workspace again goes back to the previous one.' },
        { key: 'general.ignore_focus_steal', kind: 'bool', def: false, np: true, help: 'A window on a hidden workspace grabbing focus is marked urgent instead.' },
        { key: 'general.show_all_in_taskbar', kind: 'bool', def: false, np: true, help: 'Windows of every workspace in the taskbar.' },
        { key: 'general.dynamic_workspaces', kind: 'bool', def: true, np: true, help: 'Create workspaces beyond `workspaces` on demand, removed once empty.' },
      ]),
      group('Cursor jump', [
        { key: 'general.cursor_jump.enabled', kind: 'bool', def: true, np: true, help: 'Move the cursor to what keybindings focus.' },
        { key: 'general.cursor_jump.trigger', kind: 'enum', options: ['monitor_focus', 'window_focus'], def: 'monitor_focus', np: true },
        { key: 'general.cursor_jump.on_workspace_switch', kind: 'bool', def: false, np: true },
      ]),
      group('Grab and move', [
        { key: 'general.grab_and_move.enabled', kind: 'bool', def: false, np: true, help: 'Hold the modifier and drag anywhere on a window: left moves it, right resizes it.' },
        { key: 'general.grab_and_move.modifier', kind: 'enum', options: ['alt', 'win'], def: 'alt', np: true },
        { key: 'general.grab_and_move.resize', kind: 'bool', def: true, np: true },
        { key: 'general.grab_and_move.draw', kind: 'enum', options: ['live', 'peeks'], def: 'live', np: true },
      ]),
      group('New and moved windows', [
        { key: 'window_behavior.initial_state', kind: 'enum', options: ['tiling', 'floating'], def: 'tiling', np: true },
        { key: 'window_behavior.exit_fullscreen_on_new_window', kind: 'bool', def: true, np: true },
        { key: 'window_behavior.drop_indicator.enabled', kind: 'bool', def: true, np: true, help: 'Highlight where a dragged window will land.' },
        { key: 'window_behavior.drop_indicator.color', kind: 'color', def: null, optional: true, np: true, help: 'Unset: the focused window\'s border color.' },
        { key: 'window_behavior.state_defaults.floating.centered', kind: 'bool', def: true, np: true },
        { key: 'window_behavior.state_defaults.floating.shown_on_top', kind: 'bool', def: true, np: true },
        { key: 'window_behavior.state_defaults.fullscreen.maximized', kind: 'bool', def: false, np: true, help: 'Maximize instead, if the window has a maximize button.' },
        { key: 'window_behavior.state_defaults.fullscreen.shown_on_top', kind: 'bool', def: false, np: true },
      ]),
    ]),
    tab('gaps', 'Gaps', 'Space between windows and along the screen\'s edges.', [
      group('Gaps', [
        { key: 'gaps.scale_with_dpi', kind: 'bool', def: true, help: 'Try the preview\'s monitor scale to see it.' },
        { key: 'gaps.inner_gap', kind: 'px', min: 0, max: 80, def: 20 },
        { key: 'gaps.outer_gap.top', kind: 'px', min: 0, max: 200, def: 20 },
        { key: 'gaps.outer_gap.right', kind: 'px', min: 0, max: 200, def: 20 },
        { key: 'gaps.outer_gap.bottom', kind: 'px', min: 0, max: 200, def: 20 },
        { key: 'gaps.outer_gap.left', kind: 'px', min: 0, max: 200, def: 20 },
      ]),
      group('A lone window', [
        { key: '_single', kind: 'bool', def: false, help: 'Its own outer gap while a workspace has one window. Set the preview to 1 window.' },
        { key: 'gaps.single_window_outer_gap.top', kind: 'px', min: 0, max: 400, def: 60, onlyIf: '_single' },
        { key: 'gaps.single_window_outer_gap.right', kind: 'px', min: 0, max: 600, def: 200, onlyIf: '_single' },
        { key: 'gaps.single_window_outer_gap.bottom', kind: 'px', min: 0, max: 400, def: 60, onlyIf: '_single' },
        { key: 'gaps.single_window_outer_gap.left', kind: 'px', min: 0, max: 600, def: 200, onlyIf: '_single' },
      ]),
    ]),
    tab('layouts', 'Layouts', 'How windows tile. The preview follows `default_layout`.', [
      group('Layout', [
        { key: 'general.default_layout', kind: 'enum', options: ['split', 'dwindle', 'scrolling'], def: 'split', help: 'Split tiles beside the focused window, dwindle halves it, scrolling makes columns on a strip.' },
        { key: 'special_workspace.layout', kind: 'enum', options: ['scrolling', 'split', 'dwindle'], def: 'scrolling', np: true },
      ]),
      group('Scrolling', [
        { key: 'layouts.scrolling.default_column_width', kind: 'pct', min: 20, max: 100, def: 50 },
        { key: 'layouts.scrolling.preset_column_widths', kind: 'list', def: ['33.333%', '50%', '66.667%'], np: true, help: 'Widths `cycle-column-width` steps through.' },
        { key: 'layouts.scrolling.center_focused_column', kind: 'enum', options: ['never', 'always', 'on_overflow'], def: 'never', help: 'Step the preview\'s focus to see it.' },
        { key: 'layouts.scrolling.always_center_single_column', kind: 'bool', def: false },
        { key: 'layouts.scrolling.preset_window_heights', kind: 'list', def: ['33.333%', '50%', '66.667%'], np: true },
        { key: 'layouts.scrolling.default_column_display', kind: 'enum', options: ['normal', 'tabbed'], def: 'normal', help: 'The preview\'s second column holds two windows.' },
        { key: 'layouts.scrolling.click_to_scroll', kind: 'bool', def: true, np: true },
        { key: 'layouts.scrolling.struts.left', kind: 'px', min: 0, max: 400, def: 0, help: 'Room kept free at the side, so neighbouring columns peek out.' },
        { key: 'layouts.scrolling.struts.right', kind: 'px', min: 0, max: 400, def: 0 },
      ]),
    ]),
    tab('effects', 'Window effects', 'Borders, corners, transparency and backdrops. The preview\'s first window is focused.', [
      group('Borders', [
        { key: 'window_effects.reserve_border_space', kind: 'bool', def: true, help: 'Borders sit inside each tile, never over a neighbour; off, they reach past the window.' },
        { key: 'window_effects.hide_border_on_fullscreen', kind: 'bool', def: false, np: true },
      ]),
      ...windowLook('focused_window', true),
      ...windowLook('other_windows', false),
    ]),
    tab('stack', 'Stacks', 'Tabbed stacks of windows. The preview\'s first window is a stack of three.', [
      group('Tab bar', [
        { key: 'stack.tab_bar_height', kind: 'px', min: 0, max: 60, def: 28, help: '0 hides the tab bar.' },
        { key: 'stack.tab_bar_position', kind: 'enum', options: ['top', 'bottom'], def: 'top' },
        { key: 'stack.tab_bar_background', kind: 'color', def: '#1f1f1f' },
        { key: 'stack.tab_corner_radius', kind: 'px', min: 0, max: 16, def: 8 },
        { key: 'stack.tab_min_width', kind: 'px', min: 0, max: 300, def: 48, help: 'Below about 60px, tabs only show their icon.' },
        { key: 'stack.tab_max_width', kind: 'px', min: 0, max: 600, def: 0, help: '0 lets tabs share the whole bar.' },
      ]),
      group('Tabs', [
        { key: 'stack.tab_active_background', kind: 'color', def: '#3a3a3a' },
        { key: 'stack.tab_hover_background', kind: 'color', def: '#2c2c2c', help: 'The preview\'s second tab is hovered.' },
        { key: 'stack.tab_inactive_background', kind: 'color', def: '#00000000' },
        { key: 'stack.tab_urgent_background', kind: 'color', def: '#8a5a00', np: true },
        { key: 'stack.tab_text_color', kind: 'color', def: '#ffffff' },
        { key: 'stack.tab_inactive_text_color', kind: 'color', def: '#a0a0a0' },
        { key: 'stack.tab_font_family', kind: 'text', def: 'Segoe UI Variable Text' },
        { key: 'stack.tab_font_size', kind: 'px', min: 8, max: 24, def: 12 },
        { key: 'stack.show_tab_icons', kind: 'bool', def: true },
        { key: 'stack.show_tab_numbers', kind: 'bool', def: false },
        { key: 'stack.tab_close_button', kind: 'enum', options: ['hover', 'always', 'never'], def: 'hover' },
      ]),
      group('Joining', [
        { key: 'stack.new_windows_join_focused_stack', kind: 'bool', def: false, np: true },
        { key: 'stack.new_tab_position', kind: 'enum', options: ['end', 'after_active'], def: 'end', np: true },
        { key: 'stack.float_owned_popups', kind: 'bool', def: true, np: true },
      ]),
    ]),
    tab('animations', 'Animations', 'Press a motion above the preview to play it with these settings.', [
      animation('window_move', 'Moving windows', { duration_ms: 150, easing: 'cubic_bezier(0.2, 0.0, 0.0, 1.0)' }, [
        { key: 'threshold_px', kind: 'num', min: 0, max: 100, step: 1, def: 10, np: true, help: 'Smallest move that animates, in pixels.' },
      ]),
      animation('scroll', 'Scrolling columns', { duration_ms: 220, easing: 'spring(1.0, 800, 0.0001)' }, [
        { key: 'threshold_px', kind: 'num', min: 0, max: 100, step: 1, def: 10, np: true },
      ]),
      animation('window_resize', 'Resizing', { duration_ms: 150, easing: 'cubic_bezier(0.2, 0.0, 0.0, 1.0)' }, [
        { key: 'threshold_px', kind: 'num', min: 0, max: 100, step: 1, def: 10, np: true },
        { key: 'style', kind: 'enum', options: ['fill', 'stretch'], def: 'fill', np: true },
      ]),
      animation('window_fullscreen', 'Fullscreen', { duration_ms: 220, easing: 'cubic_bezier(0.16, 1.0, 0.3, 1.0)' }, [
        { key: 'style', kind: 'enum', options: ['fill', 'stretch'], def: 'stretch', np: true },
      ]),
      animation('window_open', 'Opening', { duration_ms: 250, easing: 'cubic_bezier(0.05, 0.9, 0.1, 1.05)' }, [
        { key: 'style', kind: 'enum', options: OPEN_STYLES, def: 'zoom' },
        { key: 'opacity_from', kind: 'num', min: 0, max: 1, step: 0.05, def: 0 },
        { key: 'zoom_from', kind: 'num', min: 0, max: 1, step: 0.05, def: 0.85 },
      ]),
      animation('window_close', 'Closing', { duration_ms: 180, easing: 'ease_in_cubic' }, [
        { key: 'style', kind: 'enum', options: OPEN_STYLES, def: 'zoom' },
        { key: 'opacity_to', kind: 'num', min: 0, max: 1, step: 0.05, def: 0 },
        { key: 'zoom_to', kind: 'num', min: 0, max: 1, step: 0.05, def: 0.85 },
      ]),
      animation('window_minimize', 'Minimizing', { enabled: false, duration_ms: 150, easing: 'ease_in_cubic' }, [
        { key: 'style', kind: 'enum', options: OPEN_STYLES, def: 'zoom' },
        { key: 'opacity_to', kind: 'num', min: 0, max: 1, step: 0.05, def: 0 },
        { key: 'zoom_to', kind: 'num', min: 0, max: 1, step: 0.05, def: 0.85 },
      ]),
      animation('workspace_switch', 'Switching workspaces', { duration_ms: 350, easing: 'ease_out_cubic' }, [
        { key: 'style', kind: 'enum', options: ['slide', 'fade', 'zoom', 'drift', 'soft_zoom', 'cut'], def: 'slide' },
        { key: 'direction', kind: 'enum', options: ['horizontal', 'vertical'], def: 'horizontal' },
        { key: 'opacity_outgoing', kind: 'num', min: 0, max: 1, step: 0.05, def: 1 },
        { key: 'opacity_incoming', kind: 'num', min: 0, max: 1, step: 0.05, def: 1 },
        { key: 'zoom_factor', kind: 'num', min: 0, max: 1, step: 0.05, def: 0, help: 'How far workspaces shrink while sliding.' },
      ]),
      animation('focus_change', 'Focus changing', { duration_ms: 150, easing: 'ease_out' }),
    ]),
  ];

  const ITEMS = TABS.flatMap((t) => t.groups.flatMap((g) => g.items));
  const BY_KEY = Object.fromEntries(ITEMS.map((item) => [item.key, item]));

  function defaults() {
    return Object.fromEntries(ITEMS.map((item) => [item.key, item.def]));
  }

  // --- Values as the config writes them -----------------------------------

  const isColor = (text) => /^#[0-9a-f]{6}([0-9a-f]{2})?$/i.test(text) || text === 'accent';

  function formatValue(item, value) {
    switch (item.kind) {
      case 'px': return `${value}px`;
      case 'pct': return `${value}%`;
      default: return value;
    }
  }

  // Reads a value from a config back into the builder's form, or
  // `undefined` if it can't.
  function parseValue(item, raw) {
    if (raw === null || raw === undefined) return item.optional ? null : undefined;
    switch (item.kind) {
      case 'bool': return typeof raw === 'boolean' ? raw : undefined;
      case 'enum': return item.options.includes(raw) ? raw : undefined;
      case 'num': return typeof raw === 'number' ? raw : undefined;
      case 'px': case 'pct': {
        const m = /^\s*(-?[\d.]+)\s*(px|%)?\s*$/.exec(String(raw));
        return m ? Number(m[1]) : undefined;
      }
      case 'color': return typeof raw === 'string' && isColor(raw) ? raw : undefined;
      case 'list': return Array.isArray(raw) ? raw.map(String) : undefined;
      default: return typeof raw === 'string' ? raw : undefined;
    }
  }

  const get = (object, path) => path.split('.').reduce((node, part) => (node == null ? undefined : node[part]), object);

  // Loads `config` (a parsed YAML object) over `state`. Returns the keys
  // read.
  function importConfig(config, state) {
    const read = [];
    for (const item of ITEMS) {
      if (item.key.startsWith('_')) continue;
      const value = parseValue(item, get(config, item.key));
      if (value !== undefined && get(config, item.key) !== undefined) {
        state[item.key] = value;
        read.push(item.key);
      }
    }
    if (get(config, 'gaps.single_window_outer_gap') !== undefined) state._single = true;
    return read;
  }

  const isChanged = (state, key) => JSON.stringify(state[key]) !== JSON.stringify(BY_KEY[key].def);

  // The config as an object, of every option or only those changed from
  // the sample config. Unset optional ones and the builder's own are left
  // out.
  function configObject(state, onlyChanged) {
    const out = {};
    for (const item of ITEMS) {
      const value = state[item.key];
      if (item.key.startsWith('_') || value === null || value === undefined) continue;
      if (item.onlyIf && !state[item.onlyIf]) continue;
      if (onlyChanged && !isChanged(state, item.key) && !(item.onlyIf && state[item.onlyIf])) continue;
      const parts = item.key.split('.');
      let node = out;
      for (const part of parts.slice(0, -1)) node = node[part] = node[part] || {};
      node[parts[parts.length - 1]] = formatValue(item, value);
    }
    return out;
  }

  // --- Easing, as GlazeWM plays it -----------------------------------------

  const BEZIERS = { linear: [0, 0, 1, 1], ease_in: [0.42, 0, 1, 1], ease_out: [0, 0, 0.58, 1], ease_in_out: [0.42, 0, 0.58, 1], ease_in_cubic: [0.32, 0, 0.67, 0], ease_out_cubic: [0.33, 1, 0.68, 1], ease_in_out_cubic: [0.65, 0, 0.35, 1], snappy: [0.16, 1, 0.3, 1] };

  function cubicBezier(x1, y1, x2, y2) {
    const bx = (t) => 3 * (1 - t) * (1 - t) * t * x1 + 3 * (1 - t) * t * t * x2 + t * t * t;
    const by = (t) => 3 * (1 - t) * (1 - t) * t * y1 + 3 * (1 - t) * t * t * y2 + t * t * t;
    return (x) => {
      let lo = 0, hi = 1;
      for (let i = 0; i < 30; i++) { const m = (lo + hi) / 2; if (bx(m) < x) lo = m; else hi = m; }
      return by((lo + hi) / 2);
    };
  }

  // `SpringParams::value_at`: from 0 toward 1, `secs` in.
  function springAt(secs, { damping, stiffness }) {
    const w = Math.sqrt(stiffness), b = damping * w, t = Math.max(0, secs);
    if (Math.abs(b - w) <= 1e-6 * Math.max(w, 1)) return 1 - Math.exp(-w * t) * (1 + w * t);
    if (b < w) {
      const w1 = Math.sqrt(w * w - b * b);
      return 1 - Math.exp(-b * t) * (Math.cos(w1 * t) + (b / w1) * Math.sin(w1 * t));
    }
    const w2 = Math.sqrt(b * b - w * w), slow = w2 - b, fast = -w2 - b;
    const c1 = fast / (slow - fast);
    return 1 + c1 * Math.exp(slow * t) - (1 + c1) * Math.exp(fast * t);
  }

  // Seconds until it stays within `epsilon` of 1.
  function springSeconds(params) {
    let last = 0;
    for (let t = 0; t < 10; t += 0.001) if (Math.abs(1 - springAt(t, params)) >= params.epsilon) last = t;
    return last + 0.001;
  }

  // `easing` text into `{ ms, at(progress) }`, or null if GlazeWM wouldn't
  // read it. A spring lasts until it settles, whatever `ms` says.
  function parseEasing(text, ms) {
    const t = String(text || '').trim().toLowerCase();
    if (t === 'ease_out_spring') {
      return { ms, at: (p) => (p <= 0 ? 0 : p >= 1 ? 1 : 1 - 2 ** (-12 * p) * Math.sin((p * 4 - 4.5) * Math.PI)) };
    }
    if (BEZIERS[t]) return { ms, at: cubicBezier(...BEZIERS[t]) };
    let m = /^cubic_bezier\(([^)]*)\)$/.exec(t);
    if (m) {
      const n = m[1].split(',').map(Number);
      if (n.length !== 4 || !n.every(Number.isFinite) || n[0] < 0 || n[0] > 1 || n[2] < 0 || n[2] > 1) return null;
      return { ms, at: cubicBezier(...n) };
    }
    if (t === 'spring' || /^spring\(/.test(t)) {
      const params = { damping: 1, stiffness: 800, epsilon: 1e-4 };
      m = /^spring\(([^)]*)\)$/.exec(t);
      if (t !== 'spring' && !m) return null;
      const names = ['damping', 'stiffness', 'epsilon'];
      const parts = m ? m[1].split(',').map((x) => x.trim()).filter(Boolean) : [];
      if (parts.length > 3) return null;
      for (let i = 0; i < parts.length; i++) {
        const [name, number] = parts[i].includes('=') ? parts[i].split('=').map((x) => x.trim()) : [names[i], parts[i]];
        const value = Number(number);
        if (!Number.isFinite(value) || value < 0) return null;
        params[name === 'damping_ratio' ? 'damping' : name] = value;
      }
      if (!(params.stiffness > 0)) return null;
      const secs = ms > 0 ? springSeconds(params) : 0;
      return { ms: secs * 1000, at: (p) => (p >= 1 ? 1 : springAt(p * secs, params)) };
    }
    return null;
  }

  // --- Where windows go ----------------------------------------------------

  const MONITOR = { w: 1920, h: 1080, taskbar: 48 };

  // The gaps in the preview's logical pixels on a monitor at `scale`:
  // scaled with it, they keep their logical size; else their physical one.
  function gaps(state, scale, count) {
    const px = (value) => (state['gaps.scale_with_dpi'] ? value : value / scale);
    const single = count === 1 && state._single;
    const outer = (side) => px(state[`gaps.${single ? 'single_window_outer_gap' : 'outer_gap'}.${side}`]);
    return { inner: px(state['gaps.inner_gap']), top: outer('top'), right: outer('right'), bottom: outer('bottom'), left: outer('left') };
  }

  function workArea(g) {
    return { x: g.left, y: g.top, w: MONITOR.w - g.left - g.right, h: MONITOR.h - MONITOR.taskbar - g.top - g.bottom };
  }

  // Where each of `count` windows goes, `focused` the focused one. Each is
  // `{ x, y, w, h, column?, tabs? }`; a scrolling strip's columns may lie
  // past the monitor, and `strip` tells how far it scrolled.
  function layout(state, count, focused, scale) {
    const g = gaps(state, scale, count);
    const area = workArea(g);
    const kind = state['general.default_layout'];
    if (count === 0) return { windows: [], area };

    if (kind === 'split') {
      const w = (area.w - g.inner * (count - 1)) / count;
      return { area, windows: Array.from({ length: count }, (_, i) => ({ x: area.x + i * (w + g.inner), y: area.y, w, h: area.h })) };
    }

    if (kind === 'dwindle') {
      const windows = [];
      let rest = { ...area };
      for (let i = 0; i < count; i++) {
        if (i === count - 1) { windows.push(rest); break; }
        if (rest.w >= rest.h) {
          const half = (rest.w - g.inner) / 2;
          windows.push({ x: rest.x, y: rest.y, w: half, h: rest.h });
          rest = { x: rest.x + half + g.inner, y: rest.y, w: half, h: rest.h };
        } else {
          const half = (rest.h - g.inner) / 2;
          windows.push({ x: rest.x, y: rest.y, w: rest.w, h: half });
          rest = { x: rest.x, y: rest.y + half + g.inner, w: rest.w, h: half };
        }
      }
      return { area, windows };
    }

    // Scrolling: columns `default_column_width` of the room between the
    // struts wide, the second holding two windows (stacked or tabbed).
    const left = state['layouts.scrolling.struts.left'] / (state['gaps.scale_with_dpi'] ? 1 : scale);
    const right = state['layouts.scrolling.struts.right'] / (state['gaps.scale_with_dpi'] ? 1 : scale);
    const view = { x: area.x + left, w: Math.max(area.w - left - right, 100) };
    const width = (state['layouts.scrolling.default_column_width'] / 100) * (view.w + g.inner) - g.inner;
    const columns = [];
    for (let i = 0; i < count; i++) {
      if (i === 2 && columns[1] && columns[1].length === 1) columns[1].push(i);
      else columns.push([i]);
    }
    const focusedColumn = columns.findIndex((column) => column.includes(focused));
    const columnX = (c) => c * (width + g.inner);
    const stripWidth = columnX(columns.length) - g.inner;

    // How far the strip scrolls: the focused column at the left of the view
    // unless it fits from the start, or centered.
    const center = state['layouts.scrolling.center_focused_column'];
    const fx = columnX(focusedColumn);
    const centered = fx + width / 2 - view.w / 2;
    let scroll;
    if (columns.length === 1 && state['layouts.scrolling.always_center_single_column']) scroll = centered;
    else if (center === 'always') scroll = centered;
    else if (fx + width <= view.w) scroll = 0;
    else if (center === 'on_overflow') scroll = centered;
    else scroll = fx + width - view.w;
    if (center !== 'always' && !(columns.length === 1 && state['layouts.scrolling.always_center_single_column'])) {
      scroll = Math.max(0, Math.min(scroll, Math.max(0, stripWidth - view.w)));
    }

    const tabbed = state['layouts.scrolling.default_column_display'] === 'tabbed';
    const windows = [];
    columns.forEach((column, c) => {
      const x = view.x + columnX(c) - scroll;
      if (column.length === 2 && tabbed) {
        windows[column[0]] = { x, y: area.y, w: width, h: area.h, column: c, tabs: 2 };
        windows[column[1]] = { x, y: area.y, w: width, h: area.h, column: c, hidden: true };
      } else {
        const h = (area.h - g.inner * (column.length - 1)) / column.length;
        column.forEach((index, row) => { windows[index] = { x, y: area.y + row * (h + g.inner), w: width, h, column: c }; });
      }
    });
    return { area, windows, strip: { scroll, width: stripWidth, view } };
  }

  return { TABS, ITEMS, BY_KEY, CURVES, MONITOR, defaults, configObject, importConfig, isChanged, parseValue, parseEasing, springAt, layout, gaps, isColor };
})();

if (typeof module !== 'undefined') module.exports = Builder;
