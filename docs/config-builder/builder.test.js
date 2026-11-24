// Checks the config builder's model: `node docs/config-builder/builder.test.js`.

const assert = require('node:assert/strict');
const B = require('./builder.js');

const near = (a, b, eps = 0.5) => assert.ok(Math.abs(a - b) <= eps, `${a} != ${b}`);

// The sample config, unchanged, writes nothing; a change writes its section.
{
  const state = B.defaults();
  assert.deepEqual(B.configObject(state, true), {});
  state['gaps.inner_gap'] = 12;
  state['window_effects.focused_window.transparency.opacity'] = 80;
  assert.deepEqual(B.configObject(state, true), {
    gaps: { inner_gap: '12px' },
    window_effects: { focused_window: { transparency: { opacity: '80%' } } },
  });
  // Everything: every option, units included, unset optional ones left out.
  const all = B.configObject(state, false);
  assert.equal(all.gaps.outer_gap.top, '20px');
  assert.equal(all.general.default_layout, 'split');
  assert.deepEqual(all.layouts.scrolling.preset_column_widths, ['33.333%', '50%', '66.667%']);
  assert.equal(all.window_effects.focused_window.border.radius, undefined);
  assert.equal(all.gaps.single_window_outer_gap, undefined, 'only when its toggle is on');
  assert.equal(all._single, undefined);
}

// A lone window's gap is written whole once turned on.
{
  const state = B.defaults();
  state._single = true;
  assert.deepEqual(B.configObject(state, true).gaps.single_window_outer_gap, { top: '60px', right: '200px', bottom: '60px', left: '200px' });
}

// Importing reads units back and keeps what it can't read as it was.
{
  const state = B.defaults();
  const read = B.importConfig({
    gaps: { inner_gap: '8px', outer_gap: { top: '4px' }, single_window_outer_gap: { top: '10px' } },
    general: { default_layout: 'scrolling', focus_follows_cursor: 'yes' },
    window_effects: { other_windows: { border: { color: 'accent' } } },
  }, state);
  assert.equal(state['gaps.inner_gap'], 8);
  assert.equal(state['gaps.outer_gap.top'], 4);
  assert.equal(state['general.default_layout'], 'scrolling');
  assert.equal(state['general.focus_follows_cursor'], false, 'not a bool: kept');
  assert.equal(state['window_effects.other_windows.border.color'], 'accent');
  assert.equal(state._single, true);
  assert.ok(read.includes('gaps.single_window_outer_gap.top'));
}

// Easing as GlazeWM reads it: springs last until settled, bad text is null.
{
  const spring = B.parseEasing('spring(1, 800, 0.0001)', 150);
  near(spring.ms, 415, 10);
  near(spring.at(1), 1, 1e-9);
  near(B.parseEasing('spring', 150).ms, spring.ms, 1);
  const bounce = B.parseEasing('spring(0.5, 800)', 150);
  assert.ok([...Array(50).keys()].some((i) => bounce.at(i / 50) > 1), 'damping under 1 overshoots');
  const linear = B.parseEasing('linear', 200);
  assert.equal(linear.ms, 200);
  near(linear.at(0.5), 0.5, 1e-3);
  near(B.parseEasing('cubic_bezier(0.2, 0.0, 0.0, 1.0)', 150).at(1), 1, 1e-3);
  near(B.parseEasing('ease_out_spring', 300).at(1), 1, 1e-9);
  for (const bad of ['bouncy', 'cubic_bezier(2, 0, 0, 1)', 'cubic_bezier(1, 2)', 'spring(1, 2, 3, 4)', 'spring(-1)']) {
    assert.equal(B.parseEasing(bad, 150), null, bad);
  }
  assert.equal(B.parseEasing('spring', 0).ms, 0, 'a 0 duration is instant');
}

// Layouts keep the gaps between windows and along the edges.
{
  const state = B.defaults();
  for (const kind of ['split', 'dwindle']) {
    state['general.default_layout'] = kind;
    const { windows, area } = B.layout(state, 4, 0, 1);
    assert.equal(windows.length, 4);
    near(area.x, 20); near(area.y, 20);
    for (const w of windows) {
      assert.ok(w.x >= area.x - 0.01 && w.y >= area.y - 0.01, kind);
      assert.ok(w.x + w.w <= area.x + area.w + 0.01 && w.y + w.h <= area.y + area.h + 0.01, kind);
    }
    const covered = windows.reduce((sum, w) => sum + w.w * w.h, 0);
    assert.ok(covered < area.w * area.h && covered > area.w * area.h * 0.9, kind);
  }

  // Unscaled gaps keep their physical size, so shrink at 200%.
  near(B.layout(state, 1, 0, 2).area.x, 20);
  state['gaps.scale_with_dpi'] = false;
  near(B.layout(state, 1, 0, 2).area.x, 10);

  // A lone window takes its own outer gap.
  state['gaps.scale_with_dpi'] = true;
  state._single = true;
  near(B.layout(state, 1, 0, 1).area.x, 200);
  near(B.layout(state, 2, 0, 1).area.x, 20);
}

// Scrolling: half-width columns, scrolling to keep the focused one in view,
// centering it when asked; the second column tabbed when set so.
{
  const state = B.defaults();
  state['general.default_layout'] = 'scrolling';
  const first = B.layout(state, 4, 0, 1);
  near(first.windows[0].x, 20);
  near(first.windows[0].w, (1880 + 20) / 2 - 20);
  near(first.windows[1].x, first.windows[2].x, 0.01);
  assert.ok(first.windows[2].y > first.windows[1].y, 'stacked in its column');

  const last = B.layout(state, 4, 3, 1);
  const focused = last.windows[3];
  assert.ok(focused.x + focused.w <= 1900.01 && focused.x >= 19.99, 'scrolled into view');

  state['layouts.scrolling.center_focused_column'] = 'always';
  const centered = B.layout(state, 4, 0, 1).windows[0];
  near(centered.x + centered.w / 2, 960, 1);

  state['layouts.scrolling.default_column_display'] = 'tabbed';
  const tabbed = B.layout(state, 4, 0, 1).windows;
  assert.equal(tabbed[1].tabs, 2);
  assert.ok(tabbed[2].hidden);

  state['layouts.scrolling.center_focused_column'] = 'never';
  state['layouts.scrolling.struts.left'] = 100;
  near(B.layout(state, 4, 0, 1).windows[0].x, 120);
}

console.log('builder: all checks passed');
