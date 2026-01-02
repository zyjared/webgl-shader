const config = {
  el: '#canvas',
  scale: 0.65,
  duration: 2 * Math.PI
};

function createShader(gl, type, source) {
  const shader = gl.createShader(type);
  gl.shaderSource(shader, source);
  gl.compileShader(shader);
  if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
    console.error('着色器编译错误:', gl.getShaderInfoLog(shader));
    return null;
  }
  return shader;
}

function createProgram(gl, vs, fs) {
  const p = gl.createProgram();
  gl.attachShader(p, vs);
  gl.attachShader(p, fs);
  gl.linkProgram(p);
  if (!gl.getProgramParameter(p, gl.LINK_STATUS)) {
    console.error('程序链接错误:', gl.getProgramInfoLog(p));
    return null;
  }
  return p;
}

const canvas = document.querySelector(config.el);
const gl = canvas.getContext('webgl') || canvas.getContext('experimental-webgl');
if (!gl) throw new Error('不支持 WebGL');

const fragment = `
precision highp float;
uniform vec2 u_resolution;
uniform float u_time;
float tanh_impl(float x) {
  if (x > 10.0) return 1.0;
  if (x < -10.0) return -1.0;
  float e = exp(2.0 * x);
  return (e - 1.0) / (e + 1.0);
}
void main() {
  float scale = u_resolution.y * ${config.scale};
  vec2 p = (gl_FragCoord.xy * 2.0 - u_resolution) / scale;
  float l = abs(0.7 - dot(p, p));
  vec2 v = p * (1.0 - l) / 0.2;
  vec4 o = vec4(0.0);
  for(float i = 1.0; i <= 8.0; i++) {
    vec4 s = vec4(sin(v.x), sin(v.y), sin(v.y), sin(v.x));
    o += (s + 1.0) * abs(v.x - v.y) * 0.2;
    v += cos(vec2(v.y, v.x) * i + vec2(0.0, i) + u_time) / i + 0.7;
  }
  vec4 ep = vec4(exp(p.y), exp(-p.y), exp(-p.y * 2.0), 1.0);
  vec4 os = max(abs(o), vec4(1e-10));
  o = vec4(tanh_impl(ep.x * exp(-4.0 * l) / os.x),
           tanh_impl(ep.y * exp(-4.0 * l) / os.y),
           tanh_impl(ep.z * exp(-4.0 * l) / os.z),
           tanh_impl(ep.w * exp(-4.0 * l) / os.w));
  gl_FragColor = o;
}`;

const vertex = `attribute vec2 a_position;void main(){gl_Position=vec4(a_position,0.0,1.0);}`;
const vs = createShader(gl, gl.VERTEX_SHADER, vertex);
const fs = createShader(gl, gl.FRAGMENT_SHADER, fragment);
const program = createProgram(gl, vs, fs);
if (!program) throw new Error('着色器程序创建失败');

const buf = gl.createBuffer();
gl.bindBuffer(gl.ARRAY_BUFFER, buf);
gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 1, -1, -1, 1, -1, 1, 1, -1, 1, 1]), gl.STATIC_DRAW);

const loc = {
  pos: gl.getAttribLocation(program, 'a_position'),
  res: gl.getUniformLocation(program, 'u_resolution'),
  time: gl.getUniformLocation(program, 'u_time')
};

let resizeTimeout;
function resize() {
  const s = Math.floor(Math.min(parseFloat(getComputedStyle(canvas).width), parseFloat(getComputedStyle(canvas).height))) || 600;
  if (canvas.width !== s || canvas.height !== s) {
    canvas.width = canvas.height = s;
    gl.viewport(0, 0, s, s);
  }
}
window.addEventListener('resize', () => { clearTimeout(resizeTimeout); resizeTimeout = setTimeout(resize, 100); });

let startTime = performance.now();
let animId = null;

function render() {
  const t = ((performance.now() - startTime) / 1000) % config.duration;
  gl.clearColor(0, 0, 0, 1);
  gl.clear(gl.COLOR_BUFFER_BIT);
  gl.useProgram(program);
  gl.enableVertexAttribArray(loc.pos);
  gl.bindBuffer(gl.ARRAY_BUFFER, buf);
  gl.vertexAttribPointer(loc.pos, 2, gl.FLOAT, false, 0, 0);
  gl.uniform2f(loc.res, canvas.width, canvas.height);
  gl.uniform1f(loc.time, t);
  gl.drawArrays(gl.TRIANGLES, 0, 6);
  animId = requestAnimationFrame(render);
}

document.addEventListener('visibilitychange', () => {
  if (document.hidden) {
    if (animId) cancelAnimationFrame(animId), animId = null;
  } else {
    if (!animId) startTime = performance.now() - ((performance.now() - startTime) % (config.duration * 1000)), render();
  }
});

resize();
render();