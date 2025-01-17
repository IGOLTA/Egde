// Vertex shader

struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local_position: vec4<f32>,
    @location(1) world_position: vec4<f32>
}

struct ChunkUniform {
    size: vec3<u32>,
    transform: mat4x4<f32>,
    invert_rotation: mat4x4<f32>
}

@group(0) @binding(0) 
var<uniform> chunk: ChunkUniform;

struct CameraUniform {
    position: vec3<f32>,
    transform: mat4x4<f32>,
}

@group(1) @binding(0) 
var<uniform> camera: CameraUniform;

@vertex
fn vs_main( in: VertexInput,) -> VertexOutput {
    var out: VertexOutput;
    out.local_position = vec4<f32>(in.position, 1.0);
    out.world_position = chunk.transform * out.local_position;
    out.clip_position = camera.transform * out.world_position;
    return out;
}

@group(0) @binding(1)
var t_albedo: texture_3d<f32>;
@group(0) @binding(2)
var c_sampler: sampler;

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let ray = chunk.invert_rotation * (in.world_position - vec4<f32>(camera.position, 1));
    let ray_dir = normalize(ray);
    let ray_pos =  vec3<f32>(in.local_position.x * f32(chunk.size.x), in.local_position.y * f32(chunk.size.y), in.local_position.z * f32(chunk.size.z));

    var map_pos = vec3<i32>(i32(ray_pos.x), i32(ray_pos.y), i32(ray_pos.z));
    
    let delta_dist = abs(vec3<f32>(1.0/ray_dir.x, 1.0/ray_dir.y, 1.0/ray_dir.z)); 

    let ray_step = vec3<i32>(sign(ray_dir.xyz));

	var side_dist = vec3<f32>(
        ((f32(map_pos.x) - ray_pos.x) * f32(ray_step.x) + f32(ray_step.x + 1) / 2.0) * delta_dist.x,
        ((f32(map_pos.y) - ray_pos.y) * f32(ray_step.y) + f32(ray_step.y + 1) / 2.0) * delta_dist.y,
        ((f32(map_pos.z) - ray_pos.z) * f32(ray_step.z) + f32(ray_step.z + 1) / 2.0) * delta_dist.z
    );

     
    var mask = vec3<i32>(0, 0, 0);
    var color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    for (var i = 0; i < i32(chunk.size.x + chunk.size.y + chunk.size.z); i++) {
        if (map_pos.x < 0 || map_pos.y < 0 || map_pos.z < 0 || map_pos.x > i32(chunk.size.x) || map_pos.y > i32(chunk.size.y) || map_pos.z > i32(chunk.size.z)) {
            discard;
        }

        color = textureSample(t_albedo, c_sampler, vec3<f32>(f32(map_pos.x) / f32(chunk.size.x), f32(map_pos.y) / f32(chunk.size.y), f32(map_pos.z) / f32(chunk.size.z)));
        if (color.x != 0.0) {
            break;
        }

        mask = vec3<i32>(
            i32(side_dist.x <= min(side_dist.y, side_dist.z)),
            i32(side_dist.y <= min(side_dist.z, side_dist.x)),
            i32(side_dist.z <= min(side_dist.x, side_dist.y))
        );

        side_dist +=  vec3<f32>(
            f32(mask.x) * delta_dist.x,
            f32(mask.y) * delta_dist.y,
            f32(mask.z) * delta_dist.z
        );

        map_pos += vec3<i32>(
            mask.x * ray_step.x,
            mask.y * ray_step.y,
            mask.z * ray_step.z
        );
    }
    return color;
}