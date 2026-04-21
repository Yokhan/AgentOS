/**
 * Pixel Office Engine — Minimal canvas renderer for agent visualization
 * Inspired by pixel-agents (MIT). Renders characters in an isometric office.
 *
 * API:
 *   const office = new PixelOffice(canvas, sprites)
 *   office.setAgents([{id, name, status, tool}])  // update from webhook
 *   office.start()  // begin render loop
 */

const TILE = 16;
const ZOOM = 2;
const T = TILE * ZOOM;
const FPS = 20;
const WALK_SPEED = 1.5; // tiles per second
const WANDER_MIN = 3, WANDER_MAX = 8; // seconds between wanders
const DESK_SPACING_X = 3; // tiles between desks horizontally
const DESK_SPACING_Y = 3; // tiles between desk rows
const DESKS_PER_ROW = 6; // desks per row

const STATES = { IDLE: 0, WALK: 1, TYPE: 2, SLEEP: 3, ERROR: 4 };
const DIRECTIONS = { DOWN: 0, UP: 1, RIGHT: 2 };

// Character sprite layout: 7 columns x 3 rows (DOWN/UP/RIGHT)
// Col 0-3: walk frames, Col 4-5: type frames, Col 6: idle
const FRAME_W = 16, FRAME_H = 16;
const WALK_FRAMES = [0,1,2,3], TYPE_FRAMES = [4,5], IDLE_FRAME = 6;

class PixelOffice {
  constructor(canvas, sprites) {
    this.canvas = canvas;
    this.ctx = canvas.getContext('2d');
    this.sprites = sprites; // {characters, floors, furniture}
    this.characters = [];
    this.gridW = 20;
    this.gridH = 14;
    this.running = false;
    this.lastTime = 0;

    // Pre-render sprite canvases from hex grids
    this.spriteCache = new Map();

    canvas.width = this.gridW * T;
    canvas.height = this.gridH * T;
    canvas.style.imageRendering = 'pixelated';
  }

  // Convert hex color grid to canvas
  renderSprite(pixels, ox, oy, w, h) {
    const key = `${ox},${oy},${w},${h}`;
    if (this.spriteCache.has(key + JSON.stringify(pixels).substring(0,20))) {
      return this.spriteCache.get(key + JSON.stringify(pixels).substring(0,20));
    }
    const c = document.createElement('canvas');
    c.width = w * ZOOM;
    c.height = h * ZOOM;
    const x = c.getContext('2d');
    for (let r = oy; r < oy + h && r < pixels.length; r++) {
      for (let col = ox; col < ox + w && col < pixels[r].length; col++) {
        if (pixels[r][col]) {
          x.fillStyle = pixels[r][col];
          x.fillRect((col - ox) * ZOOM, (r - oy) * ZOOM, ZOOM, ZOOM);
        }
      }
    }
    return c;
  }

  // Get character sprite frame
  getCharFrame(charIdx, state, dir, frame) {
    const sp = this.sprites.characters[charIdx % this.sprites.characters.length];
    if (!sp) return null;
    const row = dir * FRAME_H;
    let col;
    if (state === STATES.TYPE) col = TYPE_FRAMES[frame % 2] * FRAME_W;
    else if (state === STATES.WALK) col = WALK_FRAMES[frame % 4] * FRAME_W;
    else col = IDLE_FRAME * FRAME_W;
    return this.renderSprite(sp.pixels, col, row, FRAME_W, FRAME_H);
  }

  setAgents(agents) {
    // Map agent data to characters
    const existing = new Map(this.characters.map(c => [c.id, c]));

    this.characters = agents.map((a, i) => {
      const prev = existing.get(a.id || a.name);
      const status = a.status || 'idle';
      const state = status === 'working' ? STATES.TYPE
                  : status === 'error' ? STATES.ERROR
                  : status === 'sleeping' ? STATES.SLEEP
                  : STATES.IDLE;

      if (prev) {
        // Update existing
        const wasState = prev.targetState;
        prev.targetState = state;
        prev.tool = a.tool || '';
        prev.name = a.name;
        if (wasState !== state && state === STATES.TYPE && prev.state === STATES.IDLE) {
          // Walk to desk then type
          prev.state = STATES.WALK;
          prev.targetX = prev.deskX;
          prev.targetY = prev.deskY;
        }
        return prev;
      }

      // New character — assign desk position (grid layout)
      const col = i % DESKS_PER_ROW;
      const row = Math.floor(i / DESKS_PER_ROW);
      const deskX = 1 + col * DESK_SPACING_X;
      const deskY = 2 + row * DESK_SPACING_Y;
      return {
        id: a.id || a.name,
        name: a.name,
        palette: i % 6,
        x: deskX, y: deskY + 1, // start below desk
        deskX: deskX, deskY: deskY + 1,
        state: state === STATES.TYPE ? STATES.TYPE : STATES.IDLE,
        targetState: state,
        dir: DIRECTIONS.DOWN,
        frame: 0,
        frameTimer: 0,
        wanderTimer: WANDER_MIN + Math.random() * (WANDER_MAX - WANDER_MIN),
        tool: a.tool || '',
        targetX: deskX, targetY: deskY + 1,
      };
    });
  }

  update(dt) {
    for (const ch of this.characters) {
      ch.frameTimer += dt;

      if (ch.state === STATES.WALK) {
        // Move toward target
        const dx = ch.targetX - ch.x;
        const dy = ch.targetY - ch.y;
        const dist = Math.sqrt(dx*dx + dy*dy);
        if (dist < 0.1) {
          ch.x = ch.targetX;
          ch.y = ch.targetY;
          ch.state = ch.targetState;
          ch.frame = 0;
        } else {
          const speed = WALK_SPEED * dt;
          ch.x += (dx / dist) * Math.min(speed, dist);
          ch.y += (dy / dist) * Math.min(speed, dist);
          ch.dir = Math.abs(dx) > Math.abs(dy)
            ? DIRECTIONS.RIGHT
            : (dy > 0 ? DIRECTIONS.DOWN : DIRECTIONS.UP);
          if (ch.frameTimer > 0.15) { ch.frame = (ch.frame + 1) % 4; ch.frameTimer = 0; }
        }
      } else if (ch.state === STATES.TYPE) {
        if (ch.frameTimer > 0.3) { ch.frame = (ch.frame + 1) % 2; ch.frameTimer = 0; }
        ch.dir = DIRECTIONS.UP; // face desk
      } else if (ch.state === STATES.IDLE) {
        ch.wanderTimer -= dt;
        if (ch.wanderTimer <= 0) {
          // Random wander
          ch.targetX = 1 + Math.floor(Math.random() * (this.gridW - 2));
          ch.targetY = 1 + Math.floor(Math.random() * (this.gridH - 2));
          ch.state = STATES.WALK;
          ch.wanderTimer = WANDER_MIN + Math.random() * (WANDER_MAX - WANDER_MIN);
        }
        // If should be typing, walk to desk
        if (ch.targetState === STATES.TYPE) {
          ch.state = STATES.WALK;
          ch.targetX = ch.deskX;
          ch.targetY = ch.deskY;
        }
      } else if (ch.state === STATES.SLEEP) {
        // Occasional zzz frame
        if (ch.frameTimer > 1) { ch.frame = (ch.frame + 1) % 2; ch.frameTimer = 0; }
      }
    }
  }

  render() {
    const ctx = this.ctx;
    ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);

    // Floor
    const floor = this.sprites.floors[0];
    if (floor) {
      for (let y = 0; y < this.gridH; y++) {
        for (let x = 0; x < this.gridW; x++) {
          const tile = this.renderSprite(floor.pixels, 0, 0, TILE, TILE);
          ctx.drawImage(tile, x * T, y * T);
        }
      }
    } else {
      ctx.fillStyle = '#2a2a3e';
      ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
    }

    // Desks (one per character)
    const desk = this.sprites.furniture?.DESK?.DESK_FRONT;
    for (const ch of this.characters) {
      if (desk) {
        const dTile = this.renderSprite(desk.pixels, 0, 0, desk.width, desk.height);
        ctx.drawImage(dTile, ch.deskX * T, (ch.deskY - 1) * T);
      } else {
        ctx.fillStyle = '#4a3a2a';
        ctx.fillRect(ch.deskX * T + 4, (ch.deskY - 1) * T + T/2, T - 8, T/2);
      }
    }

    // Characters (z-sorted by Y)
    const sorted = [...this.characters].sort((a, b) => a.y - b.y);
    for (const ch of sorted) {
      const sprite = this.getCharFrame(ch.palette, ch.state, ch.dir, ch.frame);
      if (sprite) {
        ctx.drawImage(sprite, Math.round(ch.x * T), Math.round(ch.y * T) - T/2);
      } else {
        // Fallback colored square
        const colors = ['#e74c3c','#3498db','#2ecc71','#f39c12','#9b59b6','#1abc9c'];
        ctx.fillStyle = colors[ch.palette % colors.length];
        ctx.fillRect(Math.round(ch.x * T) + T/4, Math.round(ch.y * T), T/2, T/2);
      }

      // Name label
      ctx.font = `${Math.round(8 * ZOOM/2)}px monospace`;
      ctx.textAlign = 'center';
      ctx.fillStyle = '#aaa';
      const shortName = ch.name.length > 8 ? ch.name.substring(0, 7) + '.' : ch.name;
      ctx.fillText(shortName, Math.round(ch.x * T) + T/2, Math.round(ch.y * T) + T + 8);

      // Status bubble
      if (ch.state === STATES.TYPE && ch.tool) {
        this.drawBubble(ctx, ch.x * T + T/2, ch.y * T - T/2, ch.tool);
      } else if (ch.state === STATES.SLEEP) {
        this.drawBubble(ctx, ch.x * T + T/2, ch.y * T - T/2, 'zzz');
      } else if (ch.state === STATES.ERROR) {
        this.drawBubble(ctx, ch.x * T + T/2, ch.y * T - T/2, '!', '#ef4444');
      }
    }
  }

  drawBubble(ctx, x, y, text, bg) {
    const w = ctx.measureText(text).width + 8;
    const h = 14;
    ctx.fillStyle = bg || '#1e1e35';
    ctx.strokeStyle = '#555';
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.roundRect(x - w/2, y - h - 2, w, h, 3);
    ctx.fill();
    ctx.stroke();
    ctx.fillStyle = bg ? '#fff' : '#aaa';
    ctx.font = '10px monospace';
    ctx.textAlign = 'center';
    ctx.fillText(text, x, y - 4);
  }

  start() {
    if (this.running) return;
    this.running = true;
    this.lastTime = performance.now();
    const loop = (now) => {
      if (!this.running) return;
      const dt = Math.min((now - this.lastTime) / 1000, 0.1);
      this.lastTime = now;
      this.update(dt);
      this.render();
      requestAnimationFrame(loop);
    };
    requestAnimationFrame(loop);
  }

  stop() { this.running = false; }
}
