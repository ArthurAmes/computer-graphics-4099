import { default as seagulls } from '../../seagulls.js'
import {Pane} from 'https://cdn.jsdelivr.net/npm/tweakpane@4.0.1/dist/tweakpane.min.js';

const WORKGROUP_SIZE = 8

let frame = 0

const sg = await seagulls.init(),
      render_shader  = await seagulls.import( './render.wgsl' ),
      compute_shader = await seagulls.import( './compute.wgsl' )

const NUM_PARTICLES = 1024, 
      // must be evenly divisble by 4 to use wgsl structs
      NUM_PROPERTIES = 4, 
      state = new Float32Array( NUM_PARTICLES * NUM_PROPERTIES )

for( let i = 0; i < NUM_PARTICLES * NUM_PROPERTIES; i+= NUM_PROPERTIES ) {
  state[ i ] = -1 + Math.random() * 2
  state[ i + 1 ] = -1 + Math.random() * 2
  0,
  0
}

const PARAMS = {
  ts: 0.01
}

const pane = new Pane();
  pane.addBinding(PARAMS,'ts',{
      min:0.01,
      max:1.00,
      step:0.01
  });

sg.buffers({ state })
  .backbuffer( false )
  .pingpong(100)
  .blend( true )
  .uniforms({ frame, res:[sg.width, sg.height ], ts:PARAMS.ts })
  .compute( compute_shader, NUM_PARTICLES / (WORKGROUP_SIZE*WORKGROUP_SIZE) )
  .render( render_shader )
  .onframe( ()=>  {
    sg.uniforms.frame = frame++
    sg.uniforms.ts = PARAMS.ts
    }  )
  .run( NUM_PARTICLES )

