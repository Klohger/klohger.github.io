// @ts-check
/**
 * @typedef { { tex?: WebGLTexture}} Texture
 */
/**
 * @typedef {{program: WebGLProgram; attribLocations: {vertexPosition: number;}; uniforms: { counter: WebGLUniformLocation, texture: WebGLUniformLocation } }} ProgramInfo
 */

/**
 *
 * @param {string} err
 * @returns { never}
 */
function panic(err) {
  console.error(err);
  document.body.innerHTML = err;
  throw err;
}

/**
 *
 * @param {WebGL2RenderingContext} gl
 */
function initPositionBuffer(gl) {
  // Create a buffer for the square's positions.
  const positionBuffer = gl.createBuffer();

  // Select the positionBuffer as the one to apply buffer
  // operations to from here out.
  gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);

  // Now create an array of positions for the square.
  const positions = new Float32Array([1.0, 1.0, -1.0, 1.0, 1, -1.0, -1, -1.0]);

  // Now pass the list of positions into WebGL to build the
  // shape. We do this by creating a Float32Array from the
  // JavaScript array, then use it to fill the current buffer.
  gl.bufferData(gl.ARRAY_BUFFER, positions, gl.STATIC_DRAW);

  return positionBuffer;
}

/**
 *
 * @param {WebGL2RenderingContext} gl
 * @param {WebGLBuffer} buffer
 * @param {ProgramInfo} programInfo
 */
function setPositionAttribute(gl, buffer, programInfo) {
  const numComponents = 2; // pull out 2 values per iteration
  const type = gl.FLOAT; // the data in the buffer is 32bit floats
  const normalize = false; // don't normalize
  const stride = 0; // how many bytes to get from one set of values to the next
  // 0 = use type and numComponents above
  const offset = 0; // how many bytes inside the buffer to start from
  gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
  gl.vertexAttribPointer(
    programInfo.attribLocations.vertexPosition,
    numComponents,
    type,
    normalize,
    stride,
    offset
  );
  gl.enableVertexAttribArray(programInfo.attribLocations.vertexPosition);
}

/**
 *
 * @param {WebGL2RenderingContext} gl
 * @param {GLenum} type
 * @param {string} source
 * @returns
 */
function loadShader(gl, type, source) {
  const shader = gl.createShader(type);

  if (shader == null) {
    panic('failed to create shader');
  }
  // Send the source to the shader object

  gl.shaderSource(shader, source);

  // Compile the shader program

  gl.compileShader(shader);

  // See if it compiled successfully

  if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
    const err = `An error occurred compiling the shaders: ${gl.getShaderInfoLog(
      shader
    )}`;
    gl.deleteShader(shader);
    panic(err);
  }

  return shader;
}

/**
 *
 * @param {WebGL2RenderingContext} gl
 * @param {string} vsSource
 * @param {string} fsSource
 * @returns
 */
function initShaderProgram(gl, vsSource, fsSource) {
  const vertexShader = loadShader(gl, gl.VERTEX_SHADER, vsSource);
  const fragmentShader = loadShader(gl, gl.FRAGMENT_SHADER, fsSource);

  // Create the shader program

  const shaderProgram = gl.createProgram();
  gl.attachShader(shaderProgram, vertexShader);
  gl.attachShader(shaderProgram, fragmentShader);
  gl.linkProgram(shaderProgram);

  // If creating the shader program failed, alert

  if (!gl.getProgramParameter(shaderProgram, gl.LINK_STATUS)) {
    panic(
      `Unable to initialize the shader program: ${gl.getProgramInfoLog(
        shaderProgram
      )}`
    );
  }

  return shaderProgram;
}

/**
 * @param {WebGL2RenderingContext} gl
 * @param {string} url
 * @param {GLenum} format
 * @param {GLenum} type
 * @returns {Texture}
 */
function createTexture(gl, url, format, type) {
  /**
   * @type {Texture}
   */
  let a = {};
  const image = new Image();
  image.onload = function (_ev) {
    let tex = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.texImage2D(gl.TEXTURE_2D, 0, format, format, type, image);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    a.tex = tex;
  };
  image.src = url;
  return a;
}

const vsSource = `#version 300 es
    in vec4 aVertexPosition;
    void main() {
      gl_Position = aVertexPosition;
    }
  `;
const fsSource = `#version 300 es
  precision lowp float;
  precision lowp sampler2D;
  out vec4 color;
  uniform sampler2D uSampler;
  uniform uint counter;
  float arr[64] = float[64](0.0, 0.5, 0.125, 0.625, 0.03125, 0.53125, 0.15625, 0.65625, 0.75, 0.25, 0.875, 0.375, 0.78125, 0.28125, 0.90625, 0.40625, 0.1875, 0.6875, 0.0625, 0.5625, 0.21875, 0.71875, 0.09375, 0.59375, 0.9375, 0.4375, 0.8125, 0.3125, 0.96875, 0.46875, 0.84375, 0.34375, 0.046875, 0.546875, 0.171875, 0.671875, 0.015625, 0.515625, 0.140625, 0.640625, 0.796875, 0.296875, 0.921875, 0.421875, 0.765625, 0.265625, 0.890625, 0.390625, 0.234375, 0.734375, 0.109375, 0.609375, 0.203125, 0.703125, 0.078125, 0.578125, 0.984375, 0.484375, 0.859375, 0.359375, 0.953125, 0.453125, 0.828125, 0.328125);
  
  vec3 dither(vec3 a) {
    return floor(vec3(15.0,15.0,15.0)*a + arr[uint(gl_FragCoord.x) & 7u | (((uint(gl_FragCoord.y) & 7u) << 3))]) / vec3(15.0,15.0,15.0);
  }
  

  vec3 hsv2rgb(vec3 c)
  {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
  }
  void main() {
    uint c = counter;
    c = 0u;
    
    vec2 barbo = texelFetch(uSampler, ivec2(gl_FragCoord)+ivec2(-35,-78), 0).ra;
    
    color = vec4(mix(vec3(0.0),dither( barbo.x * hsv2rgb(vec3(gl_FragCoord.x*0.00130718954248366 + float(counter) * 0.0234375,1.0,1.0))), barbo.y),1.0);
  }
  `;

function main() {
  const canvas = document.createElement('canvas');
  canvas.width = 256;
  canvas.height = 128;
  document.body.appendChild(canvas);
  const gl = canvas.getContext('webgl2', {
    alpha: false,
    antialias: false,
    stencil: false,
    desynchronized: true,
    preserveDrawingBuffer: false,
  });
  if (gl == null) {
    panic("can't get te gl2");
  }
  const pos_buf = initPositionBuffer(gl);
  const shaderProgram = initShaderProgram(gl, vsSource, fsSource);
  const programInfo = {
    program: shaderProgram,
    attribLocations: {
      vertexPosition: gl.getAttribLocation(shaderProgram, 'aVertexPosition'),
    },
    uniforms: {
      counter: (() => {
        const counter = gl.getUniformLocation(shaderProgram, 'counter');
        if (counter === null) {
          panic("uniform 'counter' missing");
        }
        return counter;
      })(),
      texture: (() => {
        const counter = gl.getUniformLocation(shaderProgram, 'uSampler');
        if (counter === null) {
          panic("uniform 'uSampler' missing");
        }
        return counter;
      })(),
    },
  };
  gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, true);
  let image = createTexture(
    gl,
    './assets/textures/body.png',
    gl.LUMINANCE_ALPHA,
    gl.UNSIGNED_BYTE
  );
  let counter = 0;
  setInterval(function () {
    gl.clearColor(0.0, 0.0, 0.0, 1.0);
    //gl.viewport(0, 0, 1, 1);
    gl.clearDepth(1.0);
    gl.enable(gl.DEPTH_TEST);
    gl.depthFunc(gl.LEQUAL);

    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);

    setPositionAttribute(gl, pos_buf, programInfo);
    gl.useProgram(programInfo.program);
    if (image.tex) {
      gl.activeTexture(gl.TEXTURE0);
      gl.bindTexture(gl.TEXTURE_2D, image.tex);
      gl.uniform1i(programInfo.uniforms.texture, 0);
      gl.uniform1ui(programInfo.uniforms.counter, counter);
      gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
      counter++;
    }
  }, 50);
}

main();
