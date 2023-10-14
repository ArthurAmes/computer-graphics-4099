struct Vant {
  pos: vec2f,
  dir: f32,
  flag: f32
}

@group(0) @binding(0) var<storage, read_write> vants: array<Vant>;
@group(0) @binding(1) var<storage, read_write> pheremones: array<f32>;
@group(0) @binding(2) var<storage, read_write> render: array<f32>;

fn vantIndex( cell:vec3u ) -> u32 {
  let size = 8u;
  return cell.x + (cell.y * size); 
}

fn pheromoneIndex( vant_pos: vec2f ) -> u32 {
  let width = ${W}.;
  return u32( abs( vant_pos.y % ${H}. ) * width + vant_pos.x );
}

@compute
@workgroup_size(8, 8,1)

fn cs(@builtin(global_invocation_id) cell:vec3u)  {
  let pi2   = 6.282;
  let index = vantIndex( cell );
  var vant:Vant  = vants[ index ];

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

  vants[ index ] = vant;
  
  // we'll look at the render buffer in the fragment shader
  // if we see a value of one a vant is there and we can color
  // it accordingly. in our JavaScript we clear the buffer on every
  // frame.
  render[ pIndex ] = 1.;