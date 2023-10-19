import { default as seagulls } from './seagulls.js'
import { default as Video    } from './video.js'
import { default as Mouse    } from './mouse.js'
import { Pane } from 'https://cdn.jsdelivr.net/npm/tweakpane@4.0.1/dist/tweakpane.min.js';

const sg     = await seagulls.init(),
      frag   = await seagulls.import( './frag.wgsl' ),
      shader = seagulls.constants.vertex + frag

await Video.init()

const params = { octaves: 3, scale: 1, speed: 0.05 }
const resolution = [ window.innerWidth, window.innerHeight ]
const pane = new Pane();
let frame = 0

Mouse.init()

pane
  .addBinding( params, 'octaves', { min: 0, max: 10, step: 1 })
  .on( 'change',  e => {
    sg.uniforms.octaves = e.value
  })

pane
  .addBinding( params, 'scale', { min: 0.01, step: 0.05 })
  .on( 'change',  e => {
    sg.uniforms.scale = e.value
  })

pane
  .addBinding( params, 'speed', { min: 0, step: 0.01 })
  .on( 'change',  e => {
    sg.uniforms.speed = e.value
  })

sg
  .uniforms({ resolution, frame, mouse:Mouse.values, octaves:3, scale:1.0, speed:0.05 })
  .onframe( ()=> { sg.uniforms.mouse = Mouse.values; sg.uniforms.frame = frame++ } )
  .textures([ Video.element ])
  .render( shader )
  .run()
