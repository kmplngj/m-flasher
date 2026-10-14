/* M-Flasher window: renders the controller's state (mf.update) and sends the
   user's choices to Python (window.pywebview.api). With ?demo=NAME and no
   pywebview, it loads demo/NAME.js (a saved state) for previews. */
'use strict';

const $ = (id) => document.getElementById(id);
const esc = (s) => String(s == null ? '' : s).replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const REGION_LABEL = { boot: 'BOOT', firmware: 'FIRMWARE', settings: 'SETTINGS', pairings: 'PAIRINGS', calibration: 'CALIBRATION',
  presets: 'PRESETS', sounds: 'SOUNDS', mac: 'MAC', free: 'free' };
const LEGEND_ORDER = ['boot', 'firmware', 'settings', 'pairings', 'calibration', 'presets', 'free', 'sounds', 'mac'];

let S = null;               // the last state
let reviewOpen = false;

function api(name, ...args) {
  if (window.pywebview && window.pywebview.api && window.pywebview.api[name]) return window.pywebview.api[name](...args);
  return Promise.resolve(null);
}

/* ---------- static parts ---------- */

function fit() {
  const b = $('board');
  const s = Math.min(window.innerWidth / 1536, window.innerHeight / 1024);
  b.style.transform = `scale(${s})`;
}

function buildGrid() {
  const ch = $('colhead'), rh = $('rowhead'), g = $('grid');
  for (let c = 0; c < 16; c++) ch.insertAdjacentHTML('beforeend', `<div>${c.toString(16).toUpperCase()}</div>`);
  for (let r = 0; r < 16; r++) rh.insertAdjacentHTML('beforeend', `<div>${(r * 16).toString(16).toUpperCase().padStart(2, '0')}</div>`);
  for (let i = 0; i < 256; i++) {
    const d = document.createElement('div');
    d.className = 'cell';
    d.dataset.i = i;
    d.addEventListener('mouseenter', () => readout(i));
    d.addEventListener('mouseleave', () => readout(null));
    g.appendChild(d);
  }
}

function readout(i) {
  const r = $('readout');
  if (i == null || !S) { r.innerHTML = '&nbsp;'; return; }
  const hex = i.toString(16).toUpperCase().padStart(2, '0');
  const addr = (i * 0x1000).toString(16).toUpperCase().padStart(5, '0');
  const end = (i * 0x1000 + 0xFFF).toString(16).toUpperCase().padStart(5, '0');
  const m = S.marks[i];
  const what = { change: 'will change', written: 'written', verified: 'written and verified', now: 'writing now' }[m] || '';
  r.textContent = `SECTOR ${hex} · 0x${addr}-0x${end} · ${REGION_LABEL[S.map[i]] || ''}${what ? ' · ' + what : ''}`;
}

/* LCD keyboard icon: dot-matrix pixels */
function lcdIcon(kind) {
  const W = 32, H = 12, px = [];
  const on = (x, y) => px.push([x, y]);
  for (let x = 0; x < W; x++) { on(x, 0); on(x, H - 1); }
  for (let y = 0; y < H; y++) { on(0, y); on(W - 1, y); }
  if (kind === 'none') {
    for (let x = 3; x < W - 3; x += 2) on(x, 6);
  } else {
    for (let x = 3; x < 12; x += 2) on(x, 3);                    // knobs / screen
    for (let x = 18; x < 29; x += 3) { on(x, 2); on(x, 3); }
    for (let x = 2; x < W - 2; x += 2) { for (let y = 6; y < H - 2; y++) on(x, y); }   // keys
  }
  return px.map(([x, y]) => `<rect x="${x * 2}" y="${y * 2}" width="1.6" height="1.6"/>`).join('');
}

/* ---------- rendering ---------- */

function update(state) {
  S = state;
  const dev = state.device, plan = state.plan || {}, phase = state.phase;
  const working = phase === 'working' || phase === 'connecting';
  $('ver').textContent = 'v' + state.version;

  // LCD
  let l1, l2, l3, prompt, alert = false;
  if (!dev) {
    l1 = phase === 'connecting' ? 'READING' : 'NO UNIT';
    l2 = phase === 'connecting' ? 'FLASH READ TWICE · COMPARING' : (state.multiple ? 'MORE THAN ONE UNIT' : 'WAITING FOR DOWNLOAD MODE');
    l3 = phase === 'connecting' ? `${state.pct}%` : 'USB';
    prompt = phase === 'connecting' ? 'DO NOT UNPLUG OR SWITCH OFF' :
      state.multiple ? 'CONNECT ONLY ONE UNIT' : 'HOLD SEQ REC (SMK-37) OR REC (FM-1) WHILE SWITCHING ON';
  } else {
    l1 = dev.model.replace(' (probably)', '').toUpperCase();
    l2 = dev.certain ? 'USB CONNECTED · DOWNLOAD MODE' : 'MODEL NOT CERTAIN · CHECK BELOW';
    l3 = 'FLASH ' + dev.flash_size;
    prompt = lcdPrompt(state);
    alert = !!(state.alert && state.alert.kind !== 'done') || plan.kind === 'refused';
  }
  $('lcd1').textContent = l1; $('lcd2').textContent = l2; $('lcd3').textContent = l3; $('lcdp').textContent = prompt;
  shrink($('lcd1'), 58, 36); shrink($('lcd2'), 30, 22); shrink($('lcdp').parentNode, 29, 18);
  $('lcd').classList.toggle('alert', alert);
  $('lcdIcon').innerHTML = lcdIcon(dev ? 'synth' : 'none');

  // device info
  const set = (id, v, cls) => { const e = $(id); e.textContent = v || '–'; e.className = cls || ''; };
  if (dev) {
    const m = $('i-model');
    if (!dev.certain) {
      m.className = 'warn';
      m.innerHTML = `<select id="modelSel"><option value="">${esc(dev.model)} ?</option>${state.boards.map((b) =>
        `<option value="${b.key}"${b.key === dev.board ? ' selected' : ''}>${esc(b.name)}</option>`).join('')}</select>`;
      $('modelSel').onchange = (e) => e.target.value && api('set_model', e.target.value);
    } else set('i-model', dev.model.toUpperCase());
    set('i-firmware', dev.firmware);
    set('i-app', dev.app, dev.app === 'OK' ? 'ok' : (dev.app.startsWith('none') ? 'warn' : 'bad'));
    set('i-bootloader', dev.bootloader.replace('open bootloader ', 'open '));
    set('i-flash', `${dev.flash} · ${dev.flash_size}`);
    set('i-key', dev['chip key'], dev.chip_key_ok ? '' : 'warn');
  } else ['i-model', 'i-firmware', 'i-app', 'i-bootloader', 'i-flash', 'i-key'].forEach((id) => set(id, '–'));

  // operation keys
  document.querySelectorAll('.op').forEach((k) => {
    k.classList.toggle('on', !!dev && k.dataset.mode === state.mode);
    k.disabled = !dev || state.busy;
  });

  renderPanel(state);

  // primary key
  const pk = $('primary'), pt = $('primaryText');
  let label = state.mode === 'backup' ? 'SAVE FULL BACKUP' : state.mode === 'restore' ? 'REVIEW & RESTORE' : 'REVIEW & FLASH';
  if (phase === 'working') label = state.mode === 'backup' ? 'SAVING…' : 'WRITING…';
  if (phase === 'connecting') label = 'READING…';
  if (phase === 'done') label = 'DONE';
  pt.textContent = label;
  pk.disabled = !dev || state.busy || phase === 'done' || !(plan.ready);

  // transfer
  document.querySelectorAll('#steps div').forEach((d) => { d.className = state.stage[d.dataset.st] || ''; });
  $('barFill').style.width = state.pct + '%';
  $('pct').textContent = state.pct + '%';

  // log
  $('log').innerHTML = state.log.slice(-4).map((l) => `<div class="${/Refused|error|failed|gone/i.test(l) ? 'warn' : ''}">${esc(l)}</div>`).join('');

  renderMap(state);
  renderModal(state);
}

/* largest font size (px) at which the element's text fits its width */
function shrink(el, max, min) {
  let s = max;
  el.style.fontSize = s + 'px';
  while (s > min && el.scrollWidth > el.clientWidth + 1) { s -= 1; el.style.fontSize = s + 'px'; }
}

function lcdPrompt(state) {
  const plan = state.plan || {}, f = state.files, phase = state.phase;
  const hex = (a) => '0x' + a.toString(16).toUpperCase().padStart(5, '0');
  if (phase === 'working') {
    if (state.mode === 'backup') return 'SAVING FULL BACKUP · 1 MiB';
    const st = state.stage;
    const n = Object.values(state.marks).filter((m) => m === 'written' || m === 'verified').length;
    if (st.write === 'idle' && st.backup === 'active') return 'SAVING A BACKUP FIRST · DO NOT SWITCH OFF';
    const verb = st.verify === 'active' ? 'VERIFYING' : 'WRITING';
    return `${verb} ${state.cur != null ? hex(state.cur) : ''} · ${n} / ${plan.count} · DO NOT SWITCH OFF`;
  }
  if (phase === 'done') return state.mode === 'backup' ? 'BACKUP SAVED · ' + (state.backup_saved || '') :
    `DONE · ${plan.count} SECTORS VERIFIED · SWITCH OFF AND ON`;
  if (plan.kind === 'refused') return 'REFUSED · ' + plan.refused.toUpperCase();
  if (plan.kind === 'need') return plan.title.toUpperCase();
  if (plan.kind === 'backup') return 'SAVE FULL BACKUP · 1 MiB · WRITES NOTHING';
  if (plan.kind === 'write') {
    if (!plan.count) return 'NOTHING TO WRITE · ALREADY ON THE UNIT';
    let what = '';
    if (state.mode === 'firmware') what = 'INSTALL ' + (f.firmware || '').replace(/\.fwsc$/i, '');
    else if (state.mode === 'restore') what = 'RESTORE ' + (f.restore || '').replace(/\.bin$/i, '');
    else what = { open: 'OPEN BOOTLOADER ' + state.open_version, stock: 'STOCK BOOTLOADER', file: 'BOOTLOADER ' + (f.bootfile || '') }[state.options.boot_source];
    const line = `${what} · ${plan.count} SECTORS (${plan.kib} KiB) · ${plan.short || plan.ranges}`;
    return line.length <= 60 ? line : `${what} · ${plan.count} SECTORS · ${plan.short || plan.ranges}`;
  }
  return '';
}

function fileRow(label, name, kind, keyText) {
  return `<div class="row"><div class="flabel">${label}</div>
    <div class="field${name ? '' : ' empty'}">${esc(name || 'no file chosen')}</div>
    <button class="key small" data-file="${kind}">${keyText || 'CHOOSE FILE'}</button></div>`;
}

function check(name, text, on) {
  return `<div class="check${on ? ' on' : ''}" data-opt="${name}"><i></i>${text}</div>`;
}

function renderPanel(state) {
  const p = $('panel'), lab = $('panelLabel'), f = state.files, o = state.options, dev = state.device;
  const auto = state.backup_saved ? `AUTO BACKUP SAVED · <b>${esc(state.backup_saved)}</b>` : 'AUTO BACKUP BEFORE EVERY WRITE';
  let html = '';
  if (!dev) {
    lab.textContent = 'HOW TO CONNECT';
    html = `<div class="howto"><em>SMK-37 with the open bootloader</em>hold <b>SEQ REC</b> while switching on<br>
      <em>FM-1 with the open bootloader</em>hold <b>REC</b> while switching on<br>
      <em>Felucca, SLOOP, Snoop37 running</em>hold <b>OCT-</b> and <b>OCT+</b> for 5 s<br>
      <em>Then</em>connect USB: a data cable, no hub<br>
      <em>Windows offers to format a disk?</em>choose <b>Cancel</b>; the unit is found by itself</div>`;
  } else if (state.mode === 'backup') {
    lab.textContent = 'BACKUP DESTINATION';
    html = `<div class="row"><div class="flabel">FOLDER</div><div class="field">${esc(state.backup_dir)}</div>
      <button class="key small" data-act="open_backups">OPEN FOLDER</button></div>
      <div class="note">Saves the whole flash (1 MiB, read twice when the unit connected) with a <b>.json</b> describing it.
      Nothing is written to the unit.</div>`;
  } else if (state.mode === 'restore') {
    lab.textContent = 'BACKUP FILE';
    html = fileRow('BACKUP FILE', f.restore, 'restore') + '<div class="recent">' +
      state.backups.slice(0, 2).map((b) => `<div class="${b.name === f.restore ? 'sel' : ''}" data-backup="${esc(b.path)}">
        <span>${esc(b.name)}</span><span>${esc(b.time)}</span><span>${esc(b.firmware)}</span></div>`).join('') +
      '</div>' + `<div class="status">${auto}</div>`;
  } else if (state.mode === 'firmware') {
    lab.textContent = 'FIRMWARE FILE';
    html = fileRow('FIRMWARE FILE', f.firmware, 'firmware') +
      check('wipe', 'Also erase settings and Bluetooth pairings', o.wipe) +
      check('with_boot', "Also put back the package's stock bootloader", o.with_boot) +
      `<div class="status">${auto}</div>`;
  } else {
    lab.textContent = 'BOOTLOADER SOURCE';
    const src = o.boot_source;
    html = `<div class="row"><div class="flabel">SOURCE</div><div class="seg">
      <button class="key${src === 'open' ? ' on' : ''}" data-src="open">OPEN ${esc(state.open_version)}</button>
      <button class="key${src === 'stock' ? ' on' : ''}" data-src="stock">STOCK (FROM .FWSC)</button>
      <button class="key${src === 'file' ? ' on' : ''}" data-src="file">FILE (.BOOT)</button></div></div>`;
    if (src === 'open') html += `<div class="note">The bundled open bootloader <b>${esc(state.open_version)}</b> for the identified model.
      Writes the boot area only (16 KiB).</div>`;
    else html += `<div style="height:10px"></div>` + (src === 'stock' ? fileRow('PACKAGE', f.stock, 'stock') : fileRow('BOOTLOADER', f.bootfile, 'bootfile'));
    html += `<div class="status">${auto}</div>`;
  }
  if (p.dataset.html !== html) { p.innerHTML = html; p.dataset.html = html; }
}

function renderMap(state) {
  const g = $('grid'), cells = g.children, marks = state.marks, dev = state.device;
  const hasMarks = Object.keys(marks).length > 0;
  const writing = state.phase === 'working' && state.mode !== 'backup';
  g.classList.toggle('waiting', !dev);
  g.classList.toggle('dimmed', hasMarks);
  for (let i = 0; i < 256; i++) {
    const c = cells[i], region = state.map[i];
    let m = marks[i];
    if (writing && m === 'change') m = 'pending';
    const cls = 'cell ' + region + (m ? ' m m-' + m : '');
    if (c.className !== cls) {
      c.className = cls;
      c.style.setProperty('--c', `var(--${region})`);
    }
  }
  // legend: regions present on this unit, with their sectors
  const ranges = state.ranges || {};
  const keys = LEGEND_ORDER.filter((r) => ranges[r] || !dev);
  const leg = keys.map((r) => `<div><i class="${r === 'free' ? 'free' : ''}" style="--c:var(--${r})"></i>${REGION_LABEL[r]}
    <span>${esc(ranges[r] || '')}</span></div>`).join('');
  if ($('legend').dataset.html !== leg) { $('legend').innerHTML = leg; $('legend').dataset.html = leg; }
  // marks legend
  let mk = '';
  const sw = (style, text) => `<div><i style="${style}"></i>${text}</div>`;
  if (writing) {
    mk = sw('background:#b7fff0', 'WRITTEN') + sw('background:var(--now)', 'NOW') + sw('background:#26302d;box-shadow:inset 0 0 0 1px #4f6a62', 'PENDING') +
      sw('background:#8ff5df;box-shadow:inset 0 0 0 2px #2c8f78', 'VERIFIED');
  } else if (state.phase === 'done' && hasMarks) {
    mk = sw('background:#8ff5df;box-shadow:inset 0 0 0 2px #2c8f78', `WRITTEN AND VERIFIED · ${Object.keys(marks).length}`);
  } else if (hasMarks) {
    mk = sw('outline:2px solid #f4efe4;outline-offset:-2px', `WILL CHANGE · ${Object.keys(marks).length}`) +
      sw('background:#3b4a45;opacity:.6', 'UNTOUCHED');
  } else if (dev) {
    mk = `<div style="color:var(--dim)">${state.mode === 'backup' ? 'A BACKUP READS ALL 256 SECTORS' : 'NOTHING MARKED: CHOOSE A FILE TO SEE WHAT CHANGES'}</div>`;
  }
  if ($('marks').dataset.html !== mk) { $('marks').innerHTML = mk; $('marks').dataset.html = mk; }
}

function renderModal(state) {
  const m = $('modal');
  const al = state.alert;
  if (al) {
    $('mTitle').firstChild.textContent = al.kind === 'done' ? 'DONE' : al.kind === 'refused' ? 'REFUSED' : 'ERROR';
    const text = al.text.charAt(0).toUpperCase() + al.text.slice(1);
    $('mBody').innerHTML = `<p class="${al.kind === 'done' ? 'safe' : 'bad'}" style="white-space:pre-line">${esc(text)}</p>`;
    $('mKeys').innerHTML = `<button class="key" data-act="dismiss">OK</button>`;
    m.hidden = false;
    return;
  }
  if (reviewOpen && state.plan && state.plan.ready && state.phase === 'ready') {
    const p = state.plan;
    $('mTitle').firstChild.textContent = 'REVIEW';
    $('mBody').innerHTML = `<div class="big">${esc(lcdPrompt(state))}</div>
      <p>${esc(p.title)}: writes <b>${p.count} sectors (${p.kib} KiB)</b>: ${esc(p.ranges)}</p>
      ${p.notes.filter((n) => !/^Installs /.test(n)).map((n) => `<p>${esc(n)}</p>`).join('')}
      ${p.warnings.map((w) => `<p class="w">⚠ ${esc(w)}</p>`).join('')}
      <p class="safe">${state.backup_saved ? 'Backup of this session: ' + esc(state.backup_saved) :
        'A full backup of the unit is saved first'}; everything written is read back and verified.</p>
      <p class="bad">Do not unplug or switch off the unit while it writes.</p>`;
    $('mKeys').innerHTML = `<button class="key" data-act="cancel">CANCEL</button>
      <button class="key go" data-act="confirm">WRITE ${p.count} SECTORS</button>`;
    m.hidden = false;
    return;
  }
  if (aboutOpen) return;
  reviewOpen = false;
  m.hidden = true;
}

let aboutOpen = false;
function about() {
  aboutOpen = true;
  $('mTitle').firstChild.textContent = 'ABOUT';
  $('mBody').innerHTML = `<p>M-Flasher ${esc(S ? S.version : '')}: backup, restore and reinstall the firmware of the M-VAVE FM-1 and
    SMK-37 Pro / Elite through the chip's USB download mode, and install the open bootloader.</p>
    <p>Protocol after jl-uboot-tool by kagaimiq (MIT). Bundles JieLi's WL82 flash loader and the open bootloader builds
    (fm1-bootloader, Apache 2.0). Fonts: Barlow Condensed, Share Tech Mono, IBM Plex Mono, VT323 (OFL).</p>
    <p style="color:var(--dim)">Unofficial: not affiliated with M-VAVE or JieLi.</p>`;
  $('mKeys').innerHTML = `<button class="key" data-act="close">CLOSE</button>`;
  $('modal').hidden = false;
}

/* ---------- input ---------- */

document.addEventListener('click', (e) => {
  const t = e.target.closest('[data-mode],[data-act],[data-file],[data-opt],[data-src],[data-backup],#primary');
  if (!t || t.disabled) return;
  if (t.id === 'primary') {
    if (S.mode === 'backup') api('start_action');
    else { reviewOpen = true; renderModal(S); }
  } else if (t.dataset.mode) api('select_mode', t.dataset.mode);
  else if (t.dataset.file) api('choose_file', t.dataset.file);
  else if (t.dataset.opt) api('set_option', t.dataset.opt, !t.classList.contains('on'));
  else if (t.dataset.src) api('set_boot_source', t.dataset.src);
  else if (t.dataset.backup) api('pick_backup', t.dataset.backup);
  else {
    const a = t.dataset.act;
    if (a === 'about') about();
    else if (a === 'close') { aboutOpen = false; $('modal').hidden = true; }
    else if (a === 'cancel') { reviewOpen = false; renderModal(S); }
    else if (a === 'confirm') { reviewOpen = false; $('modal').hidden = true; api('confirm'); }
    else api(a);
  }
});
document.addEventListener('keydown', (e) => {
  if (e.key === 'Escape' && !$('modal').hidden && !(S && S.alert)) { reviewOpen = false; aboutOpen = false; $('modal').hidden = true; }
});

window.mf = { update };
window.addEventListener('resize', fit);
buildGrid();
fit();
window.addEventListener('pywebviewready', () => api('ready').then((s) => s && update(s)));

const demo = new URLSearchParams(location.search).get('demo');
if (demo && !window.pywebview) {
  const sc = document.createElement('script');
  sc.src = `demo/${demo}.js`;
  sc.onload = () => { update(window.MF_DEMO); if (window.MF_DEMO_REVIEW) { reviewOpen = true; renderModal(S); } };
  document.body.appendChild(sc);
}
