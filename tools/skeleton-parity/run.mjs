// The skeleton shaders on a real browser GPU against the CPU (examples/skeleton_parity.rs writes the input):
//
//   1. the text shader before this change (its source given as a file) and after, without its int precision line
//      and with it, each chunk drawn into its own target with no blending: the RGBA bytes of "before" and "after
//      without int precision" are compared byte for byte (the refactor), and "before" against "after" (the effect
//      of the int precision on this GPU)
//   2. skeletonDistance written into a float target at every pixel centre of a GRID x GRID grid per glyph, against
//      the CPU twin's value at the same point (the points are exact in f32 on both sides)
//
//   node run.mjs <parity.json> <before.frag.glsl> <chromium|webkit|firefox> [launch args...]
//
// Needs `playwright` resolvable from the working directory (NODE_PATH or a local node_modules). Prints one JSON
// object; exits 1 when the page could not be driven or a comparison compared nothing.
//
// Author: Moroya Sakamoto

import fs from "node:fs";
import { chromium, webkit, firefox } from "playwright";

const [jsonPath, beforePath, engineName, ...args] = process.argv.slice(2);
const data = JSON.parse(fs.readFileSync(jsonPath, "utf8"));
const before = fs.readFileSync(beforePath, "utf8");
const engine = { chromium, webkit, firefox }[engineName];
if (!engine) throw new Error(`unknown engine ${engineName}`);

const browser = await engine.launch({ args });
const page = await browser.newPage();
await page.setContent("<!doctype html><canvas id=c width=8 height=8></canvas>");
const result = await page.evaluate(({ data, before }) => {
  const gl = document.getElementById("c").getContext("webgl2", { antialias: false, preserveDrawingBuffer: true });
  if (!gl) return { error: "no webgl2" };
  const info = gl.getExtension("WEBGL_debug_renderer_info");
  const renderer = info ? gl.getParameter(info.UNMASKED_RENDERER_WEBGL) : gl.getParameter(gl.RENDERER);
  const floatTarget = !!gl.getExtension("EXT_color_buffer_float");
  function program(vs, fs) {
    const make = (kind, src) => {
      const s = gl.createShader(kind);
      gl.shaderSource(s, src);
      gl.compileShader(s);
      if (!gl.getShaderParameter(s, gl.COMPILE_STATUS)) throw new Error(gl.getShaderInfoLog(s));
      return s;
    };
    const p = gl.createProgram();
    gl.attachShader(p, make(gl.VERTEX_SHADER, vs));
    gl.attachShader(p, make(gl.FRAGMENT_SHADER, fs));
    gl.linkProgram(p);
    if (!gl.getProgramParameter(p, gl.LINK_STATUS)) throw new Error(gl.getProgramInfoLog(p));
    const block = gl.getUniformBlockIndex(p, "MultiSkeletonBlock");
    if (block === gl.INVALID_INDEX) throw new Error("no MultiSkeletonBlock");
    gl.uniformBlockBinding(p, block, 0);
    return p;
  }
  const ubo = gl.createBuffer();
  gl.bindBuffer(gl.UNIFORM_BUFFER, ubo);
  gl.bufferData(gl.UNIFORM_BUFFER, data.chunks[0].length, gl.DYNAMIC_DRAW);
  gl.bindBufferBase(gl.UNIFORM_BUFFER, 0, ubo);
  const quad = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, quad);
  gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 1, -1, -1, 1, 1, 1]), gl.STATIC_DRAW);
  function target(w, h, float) {
    const t = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, t);
    gl.texStorage2D(gl.TEXTURE_2D, 1, float ? gl.RGBA32F : gl.RGBA8, w, h);
    const f = gl.createFramebuffer();
    gl.bindFramebuffer(gl.FRAMEBUFFER, f);
    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, t, 0);
    if (gl.checkFramebufferStatus(gl.FRAMEBUFFER) !== gl.FRAMEBUFFER_COMPLETE) throw new Error("incomplete target");
    gl.viewport(0, 0, w, h);
    return f;
  }
  // 1. the text shader, three ways, chunk by chunk
  const W = 1024, H = 256;
  const variants = {
    before: program(data.vert, before),
    after_without_int: program(data.vert, data.frag_without_int),
    after: program(data.vert, data.frag),
  };
  const pixels = {};
  for (const [name, p] of Object.entries(variants)) {
    gl.useProgram(p);
    const pos = gl.getAttribLocation(p, "aQuadPos");
    gl.enableVertexAttribArray(pos);
    gl.vertexAttribPointer(pos, 2, gl.FLOAT, false, 0, 0);
    gl.uniform4f(gl.getUniformLocation(p, "uFgColor"), 0.925, 0.933, 0.945, 1.0);
    gl.uniform4f(gl.getUniformLocation(p, "uBgColor"), 0.059, 0.09, 0.165, 0.0);
    gl.uniform1f(gl.getUniformLocation(p, "uAntiAlias"), 0.006);
    gl.uniform2f(gl.getUniformLocation(p, "uCanvasSize"), W, H);
    gl.uniform1f(gl.getUniformLocation(p, "uYPadding"), 0.15);
    pixels[name] = data.chunks.map((chunk, ci) => {
      target(W, H, false);
      gl.disable(gl.BLEND);
      gl.clearColor(0, 0, 0, 0);
      gl.clear(gl.COLOR_BUFFER_BIT);
      gl.bindBuffer(gl.UNIFORM_BUFFER, ubo);
      gl.bufferSubData(gl.UNIFORM_BUFFER, 0, new Uint8Array(chunk));
      gl.drawArraysInstanced(gl.TRIANGLE_STRIP, 0, 4, data.instance_counts[ci]);
      const out = new Uint8Array(W * H * 4);
      gl.readPixels(0, 0, W, H, gl.RGBA, gl.UNSIGNED_BYTE, out);
      return out;
    });
  }
  const compare = (a, b) => {
    let bytes = 0, differ = 0, maxDiff = 0, lit = 0;
    for (let ci = 0; ci < a.length; ci++) {
      for (let i = 0; i < a[ci].length; i++) {
        bytes++;
        const d = Math.abs(a[ci][i] - b[ci][i]);
        if (d) differ++;
        if (d > maxDiff) maxDiff = d;
        if (i % 4 === 3 && a[ci][i] > 0) lit++;
      }
    }
    return { bytes, differ, max_diff_of_255: maxDiff, lit_pixels: lit };
  };
  const coverage = {
    refactor: compare(pixels.before, pixels.after_without_int),
    int_precision: compare(pixels.before, pixels.after),
  };
  // 2. the distance, glyph by glyph, against the CPU
  let distance = { error: "no float render target (EXT_color_buffer_float)" };
  if (floatTarget) {
    const p = program("#version 300 es\nin vec2 aQuadPos;\nvoid main(){gl_Position=vec4(aQuadPos,0.0,1.0);}\n", data.distance_frag);
    gl.useProgram(p);
    const pos = gl.getAttribLocation(p, "aQuadPos");
    gl.enableVertexAttribArray(pos);
    gl.vertexAttribPointer(pos, 2, gl.FLOAT, false, 0, 0);
    const G = data.grid;
    let compared = 0, maxAbs = 0, worst = null, signDiffer = 0, sumAbs = 0;
    for (const g of data.cpu) {
      target(G, G, true);
      gl.bindBuffer(gl.UNIFORM_BUFFER, ubo);
      gl.bufferSubData(gl.UNIFORM_BUFFER, 0, new Uint8Array(data.chunks[g.chunk]));
      gl.uniform1i(gl.getUniformLocation(p, "uGlyph"), g.slot);
      gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
      const out = new Float32Array(G * G * 4);
      gl.readPixels(0, 0, G, G, gl.RGBA, gl.FLOAT, out);
      const bits = new Uint32Array(1), f = new Float32Array(bits.buffer);
      for (let i = 0; i < G * G; i++) {
        bits[0] = g.bits[i];
        const cpu = f[0], gpu = out[i * 4];
        const d = Math.abs(gpu - cpu);
        compared++;
        sumAbs += d;
        if (Math.sign(gpu) !== Math.sign(cpu)) signDiffer++;
        if (d > maxAbs) { maxAbs = d; worst = { glyph: data.cpu.indexOf(g), i, cpu, gpu }; }
      }
    }
    distance = { compared, max_abs_em: maxAbs, mean_abs_em: sumAbs / Math.max(compared, 1), sign_differ: signDiffer, worst };
  }
  return { renderer, text: data.text, coverage, distance };
}, { data, before });
await browser.close();
console.log(JSON.stringify({ engine: engineName, ...result }));
const ok = !result.error && result.coverage?.refactor.bytes > 0 && result.coverage.refactor.lit_pixels > 0
  && result.distance?.compared > 0;
process.exit(ok ? 0 : 1);
