import { default as seagulls } from './seagulls.js'

const WORKGROUP_SIZE = 8,
      DISPATCH_COUNT = [2,1,1],
      GRID_SIZE = 2,
      NUM_AGENTS = 128

const W = Math.round( window.innerWidth  / GRID_SIZE ),
      H = Math.round( window.innerHeight / GRID_SIZE )

const render_shader = seagulls.constants.vertex + `
struct VertexInput {
  @location(0) pos: vec2f,
  @builtin(instance_index) instance: u32,
}

struct Vant {
  pos: vec2f,
  dir: f32,
  flag: f32
}

@group(0) @binding(0) var<storage> vants_1: array<Vant>;
@group(0) @binding(1) var<storage> vants_2: array<Vant>;
@group(0) @binding(2) var<storage> vants_3: array<Vant>;
@group(0) @binding(3) var<storage> pheromones: array<f32>;
@group(0) @binding(4) var<storage> render: array<f32>;

@fragment 
fn fs( @builtin(position) pos : vec4f ) -> @location(0) vec4f {
  let grid_pos = floor( pos.xy / ${GRID_SIZE}.);
  
  let pidx = grid_pos.y  * ${W}. + grid_pos.x;
  let p = pheromones[ u32(pidx) ];
  let v = render[ u32(pidx) ];

  let out = select( vec3(p) , vec3(1.,0.,0.), v > 0. );
  
  return vec4f( out, 1. );
}`

const compute_shader =`
struct Vant {
  pos: vec2f,
  dir: f32,
  flag: f32
}

@group(0) @binding(0) var<storage, read_write> vants_1: array<Vant>;
@group(0) @binding(1) var<storage, read_write> vants_2: array<Vant>;
@group(0) @binding(2) var<storage, read_write> vants_3: array<Vant>;
@group(0) @binding(3) var<storage, read_write> pheremones: array<f32>;
@group(0) @binding(4) var<storage, read_write> render: array<f32>;

const pi2   = ${Math.PI*2};

fn vantIndex( cell:vec3u ) -> u32 {
  let size = ${WORKGROUP_SIZE}u;
  return cell.x + (cell.y * size); 
}

fn pheromoneIndex( vant_pos: vec2f ) -> u32 {
  let width = ${W}.;
  return u32( abs( vant_pos.y % ${H}. ) * width + vant_pos.x );
}

fn calc_vant_1(index: u32) {
  var vant:Vant  = vants_1[ index ];

  let pIndex    = pheromoneIndex( vant.pos );
  let pheromone = pheremones[ pIndex ];

  // if pheromones were found
  if( pheromone != 0. ) {
    vant.dir += select(.25,-.25,vant.flag==0.); // turn 90 degrees counter-clockwise
    pheremones[ pIndex ] = 0.;  // set pheromone flag
  }else{
    vant.dir += select(-.25,.25,vant.flag==0.); // turn 90 degrees counter-clockwise
    pheremones[ pIndex ] = 1.;  // unset pheromone flag
  }

  // calculate direction based on vant heading
  let dir = vec2f( sin( vant.dir * pi2 ), cos( vant.dir * pi2 ) );
  
  vant.pos = round( vant.pos + dir ); 

  vants_1[ index ] = vant;
  
  // we'll look at the render buffer in the fragment shader
  // if we see a value of one a vant is there and we can color
  // it accordingly. in our JavaScript we clear the buffer on every
  // frame.
  render[ pIndex ] = 1.;
}

fn calc_vant_2(index: u32) {
  var vant:Vant  = vants_2[ index ];

  let pIndex    = pheromoneIndex( vant.pos );
  let pheromone = pheremones[ pIndex ];

  // if pheromones were found
  if( pheromone != 0. ) {
    vant.dir += select(.125,-.125,vant.flag==0.); // turn 45 degrees counter-clockwise
    pheremones[ pIndex ] = 0.;  // set pheromone flag
  }else{
    vant.dir += select(-.125,.125,vant.flag==0.); // turn 45 degrees counter-clockwise
    pheremones[ pIndex ] = 1.;  // unset pheromone flag
  }

  // calculate direction based on vant heading
  let dir = vec2f( sin( vant.dir * pi2 ), cos( vant.dir * pi2 ) );
  
  vant.pos = round( vant.pos + dir ); 

  vants_2[ index ] = vant;
  
  // we'll look at the render buffer in the fragment shader
  // if we see a value of one a vant is there and we can color
  // it accordingly. in our JavaScript we clear the buffer on every
  // frame.
  render[ pIndex ] = 1.0;
}

fn calc_vant_3(index: u32) {
  var vant:Vant  = vants_3[ index ];

  let pIndex    = pheromoneIndex( vant.pos );
  let pheromone = pheremones[ pIndex ];

  // if pheromones were found
  if( pheromone != 0. ) {
    vant.dir += select(.25,0.0,vant.flag==0.); // turn 90 degrees counter-clockwise
    vant.flag = 10.0;
    pheremones[ pIndex ] = 0.;  // set pheromone flag
  }else{
    vant.flag -= 1.0;
    pheremones[ pIndex ] = 1.;  // unset pheromone flag
  }

  // calculate direction based on vant heading
  let dir = vec2f( sin( vant.dir * pi2 ), cos( vant.dir * pi2 ) );
  
  vant.pos = round( vant.pos + dir ); 

  vants_3[ index ] = vant;
  
  // we'll look at the render buffer in the fragment shader
  // if we see a value of one a vant is there and we can color
  // it accordingly. in our JavaScript we clear the buffer on every
  // frame.
  render[ pIndex ] = 1.0;
}

@compute
@workgroup_size(${WORKGROUP_SIZE}, ${WORKGROUP_SIZE},1)
fn cs(@builtin(global_invocation_id) cell:vec3u)  {
  let index = vantIndex( cell );

  calc_vant_1(index);
  calc_vant_2(index);
  calc_vant_3(index);
}`
 
const NUM_PROPERTIES = 4 // must be evenly divisble by 4!
const pheromones   = new Float32Array( W*H ) // hold pheromone data
const vants_render = new Float32Array( W*H ) // hold info to help draw vants
const vants_1        = new Float32Array( NUM_AGENTS * NUM_PROPERTIES ) // hold vant info
const vants_2        = new Float32Array( NUM_AGENTS * NUM_PROPERTIES ) // hold vant info
const vants_3        = new Float32Array( NUM_AGENTS * NUM_PROPERTIES ) // hold vant info

for( let i = 0; i < NUM_AGENTS * NUM_PROPERTIES; i+= NUM_PROPERTIES ) {
  vants_1[ i ]   = Math.floor( (.45+Math.random()/10) * W ) // x
  vants_1[ i+1 ] = Math.floor( (.45+Math.random()/10) * H ) // y
  vants_1[ i+2 ] = 0 // direction 
  vants_1[ i+3 ] = Math.round( Math.random()  ) // vant behavior type 

  vants_2[ i ]   = Math.floor( (.45+Math.random()/10) * W ) // x
  vants_2[ i+1 ] = Math.floor( (.45+Math.random()/10) * H ) // y
  vants_2[ i+2 ] = 0 // direction 
  vants_2[ i+3 ] = Math.round( Math.random()  ) // vant behavior type 

  vants_3[ i ]   = Math.floor( (.45+Math.random()/10) * W ) // x
  vants_3[ i+1 ] = Math.floor( (.45+Math.random()/10) * H ) // y
  vants_3[ i+2 ] = 0 // direction 
  vants_3[ i+3 ] = Math.round( Math.random()  ) // vant behavior type 
}

const sg = await seagulls.init()

sg.buffers({
    vants_1,
    vants_2,
    vants_3,
    pheromones,
    vants_render
  })
  .backbuffer( false )
  .compute( compute_shader, DISPATCH_COUNT )
  .render( render_shader )
  .onframe( ()=> sg.buffers.vants_render.clear() )
  .run(1)
