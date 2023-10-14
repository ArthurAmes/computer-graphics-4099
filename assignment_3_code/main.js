import { default as seagulls } from '../../seagulls.js'
import { default as Mouse    } from '../../mouse.js'
import { Pane } from 'https://cdn.jsdelivr.net/npm/tweakpane@4.0.1/dist/tweakpane.min.js';

const w = window.innerWidth;
const h = window.innerHeight;

const sg      = await seagulls.init(),
      frag    = await seagulls.import( './frag.wgsl' ),
      compute = await seagulls.import( './compute.wgsl' ),
      render  = seagulls.constants.vertex + frag,
      size    = w * h,
      stateA1   = new Float32Array( size ),
      stateA2   = new Float32Array( size ),
      stateB1   = new Float32Array( size ),
      stateB2   = new Float32Array( size ),
      delta1    = new Float32Array( size ),
      delta2    = new Float32Array( size )

Mouse.init()

for( let i = 0; i < size; i++ ) {
  stateA1[ i ] = Math.random();
  stateB1[ i ] = 0.0;
  stateA2[ i ] = 0.0;
  stateB2[ i ] = 0.0;
  delta1[ i ] = 0.0;
  delta2[ i ] = 0.0;
}

for( let j = h/2 - 100; j < h/2 + 100; j++) {
  for (let i = w-100; i < w+100; i++) {
    stateB1[j * w + i] = 1.0;
  }
}

const pane = new Pane();

const params = { DA: 1.0, DB: 0.2, F: 0.0367, K: 0.0649, DT: 0.75, Scale: 1.0, Multiplier: 0.0 }

pane
  .addBinding( params, 'DA', { min: 0, max: 1 })
  .on( 'change',  e => {
    sg.uniforms.DA = e.value
  })

pane
  .addBinding( params, 'DB', { min: 0, max: 1 })
  .on( 'change',  e => {
    sg.uniforms.DB = e.value
  })

pane
  .addBinding( params, 'F', { min: 0, max: 0.1 })
  .on( 'change',  e => {
    sg.uniforms.F = e.value
  })

pane
  .addBinding( params, 'K', { min: 0, max: 0.1 })
  .on( 'change',  e => {
    sg.uniforms.K = e.value
  })

pane
  .addBinding( params, 'DT', { min: 0, max: 1 })
  .on( 'change',  e => {
    sg.uniforms.DT = e.value
  })

pane
  .addBinding( params, 'Scale', { min: 0, max: 5.0 })
  .on( 'change',  e => {
    sg.uniforms.scale = e.value
  })
pane
  .addBinding( params, 'Multiplier', { min: -0.07, max: 0.07 })
  .on( 'change',  e => {
    sg.uniforms.multiplier = e.value
  })



sg.buffers({ stateA1:stateA1, stateA2:stateA2, stateB1:stateB1, stateB2:stateB2, delta1:delta1, delta2:delta2 })
  .uniforms({ resolution:[ window.innerWidth, window.innerHeight ], mouse:Mouse.values, DA:params.DA, DB: params.DB, F:params.F, K:params.K, DT:params.DT, scale:params.Scale, multiplier:params.Multiplier })
  .onframe(() => { sg.uniforms.mouse = Mouse.values; })
  .backbuffer( false )
  .pingpong( 5 )
  .compute( 
    compute, 
    [Math.round(window.innerWidth / 8), Math.round(window.innerHeight/8), 1], 
    { pingpong: ['stateA1', 'stateB1', 'delta1'] }
  )
  .render( render )
  .run()