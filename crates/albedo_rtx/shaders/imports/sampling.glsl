#ifndef SAMPLING_H
#define SAMPLING_H

/**
 * Contains the result of a sampled BSDF. This struct can then be used to shade
 * the surface.
 *
 * **Note**: Contains pre-computed variables that are used at multiple places
 * throughout the shader lifecycle.
 */
struct BSDFSample
{
	vec3 dir;
  vec3 H;
  float NdotL;
  float NdotH;
  float LdotH;
  float NdotV;
  float pdf;
};

/**
 * Contains material data after processing.
 *
 * **Note**: This might be different from the user's material.
 */
struct MaterialState
{
  vec3 albedo;
  float metallic;
  float perceptualRoughness;
  float roughness;
  float roughness2;
};

/*
 * Implementation of Hammersley Points on the Hemisphere
 */
vec3
randomCosineWeightedVector(inout uint seed)
{
  // To avoid to use a second sine and a normalization, it's possible to
  // use directly the random number in [0.0; 1.0] and scale the generated
  // `x` and `z` coordinates by it to obtain a normalized vector.
  // The code below is equivalent to:
  //   x = cos(theta), y = sin(phi), z = sin(theta);
  //   normalize(x, y, z);

  float theta = rand(seed) * TWO_PI;
  float r = max(EPSILON, rand(seed));
  float rLen = sqrt(max(EPSILON, 1.0 - r));

  float z = sqrt(r); // weights the samples to tend the normal
  float x = cos(theta) * rLen; // weights to preserve normalization
  float y = sin(theta) * rLen; // weights to preserve normalization

  return vec3(x, y, z);
}

/**
 * Generalized Trowbridge Reitz, Burley.
 */
float GTR2(float NdotH, float a2)
{
  float t = 1.0 + (a2 - 1.0) * NdotH * NdotH;
  return a2 / max(EPSILON, PI_F * t * t);
}

/**
 * Geometry function: Smith
 */
float GeometrySmith_GGX(float NdotV, float a2)
{
	float b = NdotV * NdotV;
	return 1.0 / (NdotV + sqrt(a2 + b - a2 * b));
}

/**
 * Generate a random sample based on the Lambert diffuse BRDF.
 *
 * @param normal The normal to the evaluated surface
 * @param tangent The tangent to the evaluated surface
 * @param bitangent The bitangent to the evaluated surface
 *
 * @return A random direction generated based on the Lambert diffuse BRDF
 */
vec3 randomSampleDiffuse_Lambert(
  const vec3 normal,
  const vec3 tangent,
  const vec3 bitangent,
  inout uint seed
)
{
  vec3 localDir = randomCosineWeightedVector(seed);
  return normalize(project(localDir, normal, tangent, bitangent));
}

/**
 * @override
 *
 * **Note**: This override re-compute the tangent and bitangent.
 * If you plan to call that multiple times, please use the other overload.
 */
vec3 randomSampleDiffuse_Lambert(const vec3 normal, inout uint seed)
{
  vec3 worldUp = abs(normal.z) < 0.9999 ? vec3(0, 0, 1) : vec3(1, 0, 0);
  vec3 tangent = normalize(cross(worldUp, normal));
  vec3 bitangent = cross(normal, tangent);
  return randomSampleDiffuse_Lambert(normal, tangent, bitangent, seed);
}

/**
 * Generate a random sample based on the GGX specular BRDF.
 *
 * @param w0Surface to eye direction vector
 * @param normal The normal to the evaluated surface
 * @param tangent The tangent to the evaluated surface
 * @param bitangent The bitangent to the evaluated surface
 * @param roughness2 The roughness squared
 * @param seed The current value of a seed variable
 *
 * @return A random direction generated based on the GGX specular BRDF
 */
vec3 randomSampleSpecular_GGX(
  const vec3 w0,
  const vec3 normal,
  const vec3 tangent,
  const vec3 bitangent,
  const float roughness2,
  inout uint seed
)
{
  // @todo: refactor.
  float r1 = rand(seed);
	float r2 = rand(seed);

  float phi = r1 * 2.0 * PI_F;
  float cosTheta = sqrt((1.0 - r2) / (1.0 + (roughness2 - 1.0) * r2));
  float sinTheta = clamp(sqrt(1.0 - (cosTheta * cosTheta)), 0.0, 1.0);
  float sinPhi = sin(phi);
  float cosPhi = cos(phi);

  vec3 H = vec3(sinTheta * cosPhi, sinTheta * sinPhi, cosTheta);
  H = project(H, normal, tangent, bitangent);
  return 2.0 * dot(w0, H) * H - w0;
}

/**
 * Approximated fresnel effect.
 */
float
SchlickFresnel(float u)
{
	float m = clamp(1.0 - u, 0.0, 1.0);
	float m2 = m * m;
	return m2 * m2 * m;
}

/**
 * Samples the BSDF function based on geometry and material data.
 *
 * @param w0 Surface to eye direction vector
 * @param normal The normal to the evaluated surface
 * @param mat The material data
 * @param seed The current value of a seed variable
 */
BSDFSample
sampleBSDF_UE4(
  const vec3 w0,
  const vec3 normal,
  const MaterialState mat,
  inout uint seed
)
{
  BSDFSample bsdf;

  vec3 worldUp = abs(normal.z) < 0.9999 ? vec3(0, 0, 1) : vec3(1, 0, 0);
  vec3 tangent = normalize(cross(worldUp, normal));
  vec3 bitangent = cross(normal, tangent);

  float diffuseRatio = 0.5 * (1.0 - mat.metallic);
  float specularRatio = 1.0 - diffuseRatio;

  /*
   * 1. Sample BSDF direction
   */

  float probability = rand(seed);
  if (probability < diffuseRatio)
  {
    bsdf.dir = randomSampleDiffuse_Lambert(normal, tangent, bitangent, seed);
  }
  else
  {
    bsdf.dir = randomSampleSpecular_GGX(w0, normal, tangent, bitangent, mat.roughness2, seed);
  }

  /*
   * 1. Sample PDF
   */

  vec3 L = bsdf.dir;
  bsdf.H = normalize(L + w0);
  bsdf.NdotL = dot(normal, L);
  bsdf.NdotH = dot(normal, bsdf.H);
	bsdf.LdotH = dot(L, bsdf.H);
	bsdf.NdotV = dot(normal, w0);

  float cosTheta = abs(bsdf.NdotH);
  float pdfGTR2 = GTR2(cosTheta, mat.roughness2) * cosTheta;

  // Calculate diffuse and specular pdfs and mix ratio
  float pdfSpec = pdfGTR2 / (4.0 * abs(bsdf.LdotH) + EPSILON);
  float pdfDiff = abs(bsdf.NdotL) * (1.0 / PI_F);

  // Weight pdfs according to ratios
  bsdf.pdf = diffuseRatio * pdfDiff + specularRatio * pdfSpec;
  return bsdf;
}

/**
 * Evaluates a sample with the given BSDF and geometric data.
 * This method is based on a general Cook-Torrance model.
 *
 * @param bsdf The BSDF sample to evaluate
 * @param normal The normal to the evaluated surface
 * @param mat The material data
 *
 * This method accepts only PBR materials based on the metal-roughness
 * workflow.
 *
 * This method is inspired and modified from:
 *  - OpenGLPathtracer: https://github.com/RobertBeckebans/OpenGL-PathTracer/blob/master/PathTracer/src/shaders/Progressive/PathTraceFrag.glsl
 *  - Real shading in Unreal Engine 4: https://blog.selfshadow.com/publications/s2013-shading-course/karis/s2013_pbs_epic_notes_v2.pdf
 */
vec3 evalSample_UE4(const BSDFSample bsdf, const vec3 normal, const MaterialState mat)
{
	if (bsdf.NdotL <= EPSILON || bsdf.NdotV <= EPSILON) { return vec3(0.0); }

	float Ds = GTR2(bsdf.NdotH, mat.roughness2);
	float FH = SchlickFresnel(bsdf.LdotH);
	float Gs = GeometrySmith_GGX(bsdf.NdotL, mat.roughness2) * GeometrySmith_GGX(bsdf.NdotV, mat.roughness2);
	vec3 Fs = mix(mix(vec3(0.0), mat.albedo, mat.metallic), vec3(1.0), FH);
  vec3 diffuse = (mat.albedo / PI_F) * (1.0 - mat.metallic);
	return diffuse + Gs * Fs * Ds;
}

#endif // SAMPLING_H
