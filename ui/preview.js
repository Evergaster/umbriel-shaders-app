const VERT = `
attribute vec2 a_pos;
varying vec2 v_uv;
void main() {
  v_uv = a_pos * 0.5 + 0.5;
  gl_Position = vec4(a_pos, 0.0, 1.0);
}`;

// Mirrors Umbriel's shared shader preamble so community effects compile here.
const CORE = `
precision highp float;
#define sin(x) sin(mod((x), 6.283185307179586))
#define cos(x) cos(mod((x), 6.283185307179586))
varying vec2 v_uv;
uniform vec2 umbriel_size;
uniform float umbriel_scale;
uniform float umbriel_time;
uniform vec2 umbriel_pointer;
uniform vec2 umbriel_expand;
uniform sampler2D umbriel_window;
uniform sampler2D umbriel_incoming;
uniform vec4 umbriel_palette[4];
uniform int umbriel_palette_count;
vec4 umbriel_sample(vec2 uv) {
  if (any(lessThan(uv, vec2(0.0))) || any(greaterThan(uv, vec2(1.0)))) return vec4(0.0);
  return texture2D(umbriel_window, uv);
}
vec4 umbriel_sample_incoming(vec2 uv) {
  if (any(lessThan(uv, vec2(0.0))) || any(greaterThan(uv, vec2(1.0)))) return vec4(0.0);
  return texture2D(umbriel_incoming, uv);
}
vec4 umbriel_sample_previous(vec2 uv) { return umbriel_sample(uv); }
vec4 umbriel_palette_at(float t) {
  if (umbriel_palette_count <= 0) return vec4(0.0);
  float span = float(umbriel_palette_count);
  float scaled = fract(t) * span;
  float index = floor(scaled);
  float next = mod(index + 1.0, span);
  vec4 from = umbriel_palette[0];
  vec4 to = umbriel_palette[0];
  for (int i = 0; i < 4; i++) {
    if (i >= umbriel_palette_count) break;
    if (float(i) == index) from = umbriel_palette[i];
    if (float(i) == next) to = umbriel_palette[i];
  }
  return mix(from, to, scaled - index);
}
float umbriel_audio_available() { return 0.0; }
float umbriel_audio_spectrum(float position) { return 0.0; }
`;

const ANIMATION = `
uniform float umbriel_progress;
uniform float umbriel_linear_progress;
uniform float umbriel_direction;
uniform vec4 umbriel_random_seed;
#define umbriel_clamped_progress clamp(umbriel_progress, 0.0, 1.0)
`;

const BORDER = `
uniform vec4 umbriel_border_hole;
uniform vec4 umbriel_border_radius;
float umbriel_border_distance(vec2 uv) {
  vec2 half_size = umbriel_border_hole.zw * umbriel_size * 0.5;
  vec2 p = (uv - umbriel_border_hole.xy) * umbriel_size - half_size;
  float r = p.y < 0.0 ? (p.x < 0.0 ? umbriel_border_radius.x : umbriel_border_radius.y)
                      : (p.x < 0.0 ? umbriel_border_radius.w : umbriel_border_radius.z);
  r = min(r, min(half_size.x, half_size.y));
  vec2 q = abs(p) - half_size + r;
  return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - r;
}
`;

const MASK = `
uniform vec4 umbriel_corner_radius;
float umbriel_mask(vec2 uv) {
  vec2 half_size = umbriel_size * 0.5;
  vec2 p = uv * umbriel_size - half_size;
  float r = p.y < 0.0 ? (p.x < 0.0 ? umbriel_corner_radius.x : umbriel_corner_radius.y)
                      : (p.x < 0.0 ? umbriel_corner_radius.w : umbriel_corner_radius.z);
  r = min(r, min(half_size.x, half_size.y));
  vec2 q = abs(p) - half_size + r;
  float d = length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - r;
  return 1.0 - smoothstep(-0.5, 0.5, d * umbriel_scale);
}
`;

const CURSOR = `
uniform int umbriel_pointer_count;
uniform vec4 umbriel_pointer_history[8];
uniform vec4 umbriel_pointer_path[64];
`;

function suffix(entry) {
  if (entry === "border")
    return "void main() { vec4 c = border(v_uv); gl_FragColor = c * smoothstep(-0.5, 0.5, umbriel_border_distance(v_uv) * umbriel_scale); }";
  if (entry === "window")
    return "void main() { gl_FragColor = mix(umbriel_sample(v_uv), window(v_uv), umbriel_mask(v_uv)); }";
  if (entry === "cursor")
    return "void main() { gl_FragColor = mix(umbriel_sample(v_uv), cursor(v_uv), umbriel_mask(v_uv)); }";
  if (entry === "screen") return "void main() { gl_FragColor = screen(v_uv); }";
  return `void main() { gl_FragColor = ${entry}(v_uv); }`;
}

function section(kind) {
  if (kind === "animation") return ANIMATION;
  if (kind === "border") return BORDER;
  if (kind === "window") return MASK;
  if (kind === "cursor") return MASK + CURSOR;
  return "";
}

function windowTexture(gl, incoming) {
  const c = document.createElement("canvas");
  c.width = 640;
  c.height = 400;
  const x = c.getContext("2d");
  x.fillStyle = incoming ? "#171320" : "#16161a";
  x.fillRect(0, 0, 640, 400);
  x.fillStyle = incoming ? "#211b2e" : "#1d1d23";
  x.fillRect(0, 0, 640, 34);
  x.fillStyle = "#3a3a44";
  for (let i = 0; i < 3; i++) x.fillRect(16 + i * 14, 14, 7, 7);
  x.fillStyle = incoming ? "#14101c" : "#121216";
  x.fillRect(0, 34, 150, 366);
  x.fillStyle = "#2b2b34";
  x.fillRect(18, 60, 96, 7);
  x.fillRect(18, 78, 72, 7);
  x.fillRect(18, 96, 110, 7);
  x.fillStyle = "#26262e";
  x.fillRect(178, 64, 300, 14);
  x.fillRect(178, 92, 420, 9);
  x.fillRect(178, 112, 380, 9);
  x.fillRect(178, 132, 404, 9);
  x.fillStyle = incoming ? "#ff8a3d" : "#d8ff00";
  x.fillRect(178, 170, 120, 34);
  x.fillStyle = "#26262e";
  x.fillRect(178, 228, 420, 9);
  x.fillRect(178, 248, 390, 9);
  x.fillRect(178, 268, 410, 9);
  x.fillStyle = "#1b1b22";
  x.fillRect(178, 310, 200, 62);
  x.fillRect(398, 310, 200, 62);

  const tex = gl.createTexture();
  gl.bindTexture(gl.TEXTURE_2D, tex);
  gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, true);
  gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, c);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
  return tex;
}

function compile(gl, type, src) {
  const shader = gl.createShader(type);
  gl.shaderSource(shader, src);
  gl.compileShader(shader);
  if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
    const log = gl.getShaderInfoLog(shader) || "compile failed";
    gl.deleteShader(shader);
    return { error: log };
  }
  return { shader };
}

export function createPreview(canvas, { onError } = {}) {
  const gl = canvas.getContext("webgl", {
    alpha: true,
    premultipliedAlpha: false,
    antialias: true,
    preserveDrawingBuffer: false,
  });

  let raf = 0;
  let running = false;
  let start = performance.now();
  let duration = 320;
  let reduceMotion = false;
  let program = null;
  let uniforms = {};
  let entry = "animation";
  let hasMain = false;
  let disposed = false;

  if (!gl) {
    onError?.("WebGL no está disponible en esta ventana.");
    return { setShader() {}, restart() {}, setReducedMotion() {}, dispose() {} };
  }

  const buffer = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
  gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 3, -1, -1, 3]), gl.STATIC_DRAW);
  const texture = windowTexture(gl, false);
  const incomingTexture = windowTexture(gl, true);

  function resize() {
    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    const w = Math.max(1, Math.round(canvas.clientWidth * dpr));
    const h = Math.max(1, Math.round(canvas.clientHeight * dpr));
    if (canvas.width !== w || canvas.height !== h) {
      canvas.width = w;
      canvas.height = h;
    }
  }

  const observer = new ResizeObserver(resize);
  observer.observe(canvas);

  function build(source, entryFn, kind) {
    entry = entryFn || "animation";
    hasMain = /void\s+main\s*\(/.test(source);
    const body = hasMain
      ? `${CORE}${section(kind)}\n${source}`
      : `${CORE}${section(kind)}\n${source}\n${suffix(entry)}`;
    const vs = compile(gl, gl.VERTEX_SHADER, VERT);
    if (vs.error) return vs.error;
    const fs = compile(gl, gl.FRAGMENT_SHADER, body);
    if (fs.error) {
      gl.deleteShader(vs.shader);
      return fs.error;
    }
    const next = gl.createProgram();
    gl.attachShader(next, vs.shader);
    gl.attachShader(next, fs.shader);
    gl.bindAttribLocation(next, 0, "a_pos");
    gl.linkProgram(next);
    gl.deleteShader(vs.shader);
    gl.deleteShader(fs.shader);
    if (!gl.getProgramParameter(next, gl.LINK_STATUS)) {
      const log = gl.getProgramInfoLog(next) || "link failed";
      gl.deleteProgram(next);
      return log;
    }
    if (program) gl.deleteProgram(program);
    program = next;
    uniforms = {
      size: gl.getUniformLocation(program, "umbriel_size"),
      scale: gl.getUniformLocation(program, "umbriel_scale"),
      pointer: gl.getUniformLocation(program, "umbriel_pointer"),
      time: gl.getUniformLocation(program, "umbriel_time"),
      expand: gl.getUniformLocation(program, "umbriel_expand"),
      palette: gl.getUniformLocation(program, "umbriel_palette[0]"),
      paletteCount: gl.getUniformLocation(program, "umbriel_palette_count"),
      window: gl.getUniformLocation(program, "umbriel_window"),
      incoming: gl.getUniformLocation(program, "umbriel_incoming"),
      progress: gl.getUniformLocation(program, "umbriel_progress"),
      linear: gl.getUniformLocation(program, "umbriel_linear_progress"),
      direction: gl.getUniformLocation(program, "umbriel_direction"),
      seed: gl.getUniformLocation(program, "umbriel_random_seed"),
      hole: gl.getUniformLocation(program, "umbriel_border_hole"),
      radius: gl.getUniformLocation(program, "umbriel_border_radius"),
      corner: gl.getUniformLocation(program, "umbriel_corner_radius"),
      pointerCount: gl.getUniformLocation(program, "umbriel_pointer_count"),
      history: gl.getUniformLocation(program, "umbriel_pointer_history[0]"),
      path: gl.getUniformLocation(program, "umbriel_pointer_path[0]"),
      workspaceAxis: gl.getUniformLocation(program, "umbriel_workspace_axis"),
    };
    return null;
  }

  const palette = new Float32Array([
    0.847, 1.0, 0.0, 1.0,
    0.945, 0.937, 0.91, 1.0,
    1.0, 0.63, 0.25, 1.0,
    1.0, 0.353, 0.27, 1.0,
  ]);
  const history = new Float32Array(8 * 4);
  const path = new Float32Array(64 * 4);

  function draw(now) {
    if (!running || disposed) return;
    resize();
    const seconds = reduceMotion ? 0.8 : (now - start) / 1000;
    const span = Math.max(duration / 1000, 0.35);
    const cycle = span + 0.85;
    let progress = reduceMotion ? 0.62 : (seconds % cycle) / span;
    progress = Math.min(1, Math.max(0, progress));
    const pointer = [
      0.5 + 0.26 * Math.cos(seconds * 0.9),
      0.5 + 0.22 * Math.sin(seconds * 0.9),
    ];
    for (let i = 0; i < 64; i++) {
      const age = (i / 63) * 1.6;
      const angle = (seconds - age) * 0.9;
      path[i * 4] = 0.5 + 0.26 * Math.cos(angle);
      path[i * 4 + 1] = 0.5 + 0.22 * Math.sin(angle);
      path[i * 4 + 2] = age;
      path[i * 4 + 3] = (i % 7) / 7;
      if (i < 8) {
        history[i * 4] = path[i * 4];
        history[i * 4 + 1] = path[i * 4 + 1];
        history[i * 4 + 2] = age;
        history[i * 4 + 3] = 0;
      }
    }
    gl.viewport(0, 0, canvas.width, canvas.height);
    gl.clearColor(0, 0, 0, 0);
    gl.clear(gl.COLOR_BUFFER_BIT);
    if (program) {
      gl.useProgram(program);
      gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
      gl.enableVertexAttribArray(0);
      gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);
      gl.uniform2f(uniforms.size, canvas.clientWidth || 360, canvas.clientHeight || 220);
      gl.uniform1f(uniforms.scale, 1);
      gl.uniform2f(uniforms.pointer, pointer[0], pointer[1]);
      gl.uniform2f(uniforms.expand, 0, 0);
      gl.uniform1f(uniforms.time, seconds);
      gl.uniform4fv(uniforms.palette, palette);
      gl.uniform1i(uniforms.paletteCount, 4);
      gl.uniform1f(uniforms.progress, progress);
      gl.uniform1f(uniforms.linear, progress);
      gl.uniform1f(uniforms.direction, 1);
      gl.uniform4f(uniforms.seed, 0.37, 0.62, 0.81, 0.19);
      gl.uniform4f(uniforms.hole, 0.06, 0.06, 0.88, 0.88);
      gl.uniform4f(uniforms.radius, 12, 12, 12, 12);
      gl.uniform4f(uniforms.corner, 12, 12, 12, 12);
      gl.uniform1i(uniforms.pointerCount, 64);
      gl.uniform4fv(uniforms.history, history);
      gl.uniform4fv(uniforms.path, path);
      gl.uniform2f(uniforms.workspaceAxis, 1, 0);
      gl.activeTexture(gl.TEXTURE0);
      gl.bindTexture(gl.TEXTURE_2D, texture);
      gl.uniform1i(uniforms.window, 0);
      gl.activeTexture(gl.TEXTURE1);
      gl.bindTexture(gl.TEXTURE_2D, incomingTexture);
      gl.uniform1i(uniforms.incoming, 1);
      gl.drawArrays(gl.TRIANGLES, 0, 3);
    }
    if (!reduceMotion) raf = requestAnimationFrame(draw);
    else running = false;
  }

  function startLoop() {
    if (disposed) return;
    cancelAnimationFrame(raf);
    running = true;
    raf = requestAnimationFrame(draw);
  }

  return {
    setShader(source, entryFn, kind, ms) {
      duration = Math.max(60, ms || 320);
      const error = build(source, entryFn, kind);
      onError?.(error);
      start = performance.now();
      startLoop();
    },
    restart() {
      start = performance.now();
      startLoop();
    },
    setReducedMotion(value) {
      reduceMotion = Boolean(value);
      startLoop();
    },
    dispose() {
      disposed = true;
      running = false;
      cancelAnimationFrame(raf);
      observer.disconnect();
      if (program) gl.deleteProgram(program);
      gl.deleteTexture(texture);
      gl.deleteTexture(incomingTexture);
      gl.deleteBuffer(buffer);
    },
  };
}
