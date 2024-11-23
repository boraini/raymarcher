#version 330
precision mediump float;

varying vec3 ray_direction;

layout(std140) uniform uni {
    vec3 origin;
    vec3 light_dir;
    vec3 light_color;
    float stop_distance;
};

struct HitInfo {
    vec3 position;
    vec3 color;
    vec3 normal;
};

float sphere(vec3 p, vec3 c, float r) {
    return length(c - p) - r;
}

float sdRoundBox( vec3 p, vec3 b, float r ) {
  vec3 q = abs(p) - b + r;
  return length(max(q,0.0)) + min(max(q.x,max(q.y,q.z)),0.0) - r;
}

float mandel(vec3 p, float power, float phase, out float trap) {
    vec3 z = p;
    vec3 dz = vec3(0.0);
    float r, theta, phi;
    float dr = 1.0;
    float t0 = 1.0;
    trap = 1.0;
    for (int i = 0; i < 32; ++i) { // change i < # for iterations.
        r = length(z);
        if(r > 2.0) continue;
        trap = min(trap, dot(z, z));
        theta = atan(z.y / z.x);
        phi = asin(z.z / r) + phase;
        dr = pow(r, power - 1.0) * dr * power + 1.0;
        r = pow(r, power);
        theta = theta * power;
        phi = phi * power;
        z = r * vec3(cos(theta) * cos(phi), sin(theta) * cos(phi), sin(phi)) + p;
        t0 = min(t0, r);
    }
    return 0.25 * log(r) * r / dr;
}

// float mandelbox(p) {
//     vec3 z = p;

//     for (int i = 0; i < 8; ++i) {
//         z = max(min(2 - z, z) -2 - z)

//         if (dot(z, z) < 0.25) {
//             z += 4;
//         } else if (dot(z, z) < 1) {
//             z /= dot(z, z);
//         }
        
//         z = z + c;
//     }

//     return 
// }

float Scale = 3.0;
float MinRad2 = 0.5;
float Iterations = 16;

vec4 scale = vec4(Scale, Scale, Scale, abs(Scale)) / MinRad2;
float absScalem1 = abs(Scale - 1.0);
float AbsScaleRaisedTo1mIters = pow(abs(Scale), float(1-Iterations));

float mandelbox(vec3 pos) {
	vec4 p = vec4(pos,1), p0 = p;  // p.w is the distance estimate
	
	for (int i=0; i < Iterations; i++) {
		p.xyz = clamp(p.xyz, -1.0, 1.0) * 2.0 - p.xyz;  // min;max;mad
		float r2 = dot(p.xyz, p.xyz);
		p *= clamp(max(MinRad2/r2, MinRad2), 0.0, 1.0);  // dp3,div,max.sat,mul
		p = p*scale + p0;
             if ( r2>1000.0) break;
	}
	return ((length(p.xyz) - absScalem1) / p.w - AbsScaleRaisedTo1mIters);
}

float my_mandel(vec3 p, out float trap) {
    // return min(mandel(p, 7.0, 0.8, trap), length(p));

    return min(mandel(p, 4.0, 0.0, trap), length(p));

    // return sdRoundBox(p, vec3(1.0, 0.8, 0.6), 0.1);

    // return sphere(p, vec3(0.0, 0.0, 0.0), 1.0);

    // return mandelbox(p);
}

vec3 normal(vec3 p) {
    float trap = 1.0;
    float epsilon = stop_distance; // arbitrary — should be smaller than any surface detail in your distance function, but not so small as to get lost in float precision
    float centerDistance = my_mandel(p, trap);
    float xDistance = my_mandel(p + vec3(epsilon, 0, 0), trap);
    float yDistance = my_mandel(p + vec3(0, epsilon, 0), trap);
    float zDistance = my_mandel(p + vec3(0, 0, epsilon), trap);
    return (vec3(xDistance, yDistance, zDistance) - centerDistance) / epsilon;
}

HitInfo cast_ray() {
    vec3 p = origin;

    for (int j = 0; j < 128; j++) {
        float trap = 1.0;
        float d = my_mandel(p, trap);

        p += d * ray_direction;

        if (d < stop_distance) {
            return HitInfo(p, trap * vec3(1.0, 1.0, 1.0), normal(p));
        }
    }

    return HitInfo(vec3(0.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0), vec3(0.0, 0.0, 0.0));
}

void main() {
    HitInfo info = cast_ray();

    if (info.position == vec3(0.0, 0.0, 0.0)) {
        gl_FragColor = vec4(0.0, 0.0, 0.0, 0.0);
    } else {
        // float dot = dot(light_dir, -info.normal);
        float dot = 1;
        gl_FragColor = vec4(light_color * info.color * dot, 1.0);
    }
}
