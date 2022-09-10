#ifndef COLORSPACE_H
#define COLORSPACE_H

// From three.js: https://github.com/mrdoob/three.js/blob/dev/src/renderers/shaders/ShaderChunk/encodings_pars_fragment.glsl.js
vec3 linearTosRGB( in vec3 value ) {
  return vec3( mix( pow( value.rgb, vec3( 0.41666 ) ) * 1.055 - vec3( 0.055 ), value.rgb * 12.92, vec3( lessThanEqual( value.rgb, vec3( 0.0031308 ) ) ) ));
}

// Approximation http://chilliant.blogspot.com/2012/08/srgb-approximations-for-hlsl.html
vec3 sRGBToLinear(vec3 color) {
  vec3 sRGB = color.rgb;
  color.rgb = sRGB * (sRGB * (sRGB * 0.305306011 + 0.682171111) + 0.012522878);
  return color;
}

#endif // COLORSPACE_H
