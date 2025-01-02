#ifndef INTERSECTION_UTILS_H
#define INTERSECTION_UTILS_H

#define STACK_SIZE 32
// #define DEBUG_CWBVH_TRAVERSAL

struct Primitive
{
  Vertex v0;
  Vertex v1;
  Vertex v2;
};

Vertex
getVertex(uint vertexOffset, uint index)
{
  return vertices[vertexOffset + index];
}

/**
 * Retrieve the triangle from vertex offset and index
 *
 * @param instance Instance to extract the primitive from
 * @param intersection Intersection data
 */
Primitive extractPrimitive(Instance instance, Intersection intersection)
{
  Primitive p;
  p.v0 = getVertex(instance.vertexRootIndex, intersection.index);
  p.v1 = getVertex(instance.vertexRootIndex, intersection.index + 1);
  p.v2 = getVertex(instance.vertexRootIndex, intersection.index + 2);
  return p;
}

vec2 interpolateBarycentric(vec2 v0, vec2 v1, vec2 v2, vec3 barycentric)
{
  return (
    barycentric.x * v0 +
    barycentric.y * v1 +
    barycentric.z * v2
  );
}
vec3 interpolateBarycentric(vec3 v0, vec3 v1, vec3 v2, vec3 barycentric)
{
  return (
    barycentric.x * v0 +
    barycentric.y * v1 +
    barycentric.z * v2
  );
}

vec3 barycentricCoordinates(vec2 uv)
{
  return vec3(1.0 - uv.x - uv.y, uv);
}

Ray
transformRay(inout Ray ray, mat4 transform)
{
  Ray result;
  // @todo: radiance and throughput should go somewhere else.
  result.origin = transformPosition(ray.origin, transform);
  result.dir = transformDirection(ray.dir, transform);
  return result;
}

float
intersectPlane(Ray ray, vec3 normal, vec3 origin, vec3 edge01, vec3 edge02)
{
  float NdotD = dot(normal, ray.dir);
  if (NdotD < EPSILON) { return MAX_FLOAT; }

  float t = dot(normal, origin - ray.origin) / NdotD;
  if (t < EPSILON) { return MAX_FLOAT; }

  vec3 intersection = (ray.origin + ray.dir * t) - origin;

  // Check not before first edge.
  float interProj = dot(edge01, intersection);
  if (interProj < EPSILON || interProj > dot(edge01, edge01)) { return MAX_FLOAT; }

  interProj = dot(edge02, intersection);
  if (interProj < EPSILON || interProj > dot(edge02, edge02)) { return MAX_FLOAT; }

  return t;
}

float
intersectAABB(vec3 origin, vec3 inverseDir, vec3 aabbMin, vec3 aabbMax)
{
  // Ray is assumed to be in local coordinates, ie:
  // ray = inverse(objectMatrix * invCameraMatrix) * ray

  // Equation of ray: O + D * t

  vec3 tbottom = inverseDir * (aabbMin - origin);
  vec3 ttop = inverseDir * (aabbMax - origin);

  vec3 tmin = min(ttop, tbottom);
  vec3 tmax = max(ttop, tbottom);

  float smallestMax = min(min(tmax.x, tmax.y), min(tmax.x, tmax.z));
  float largestMin = max(max(tmin.x, tmin.y), max(tmin.x, tmin.z));

  if (smallestMax < largestMin || smallestMax < 0.0) { return MAX_FLOAT; }
  return (largestMin > 0.0) ? largestMin : smallestMax;
}

bool
isIntersectingAABB(vec3 origin, vec3 inverseDir, vec3 boxMin, vec3 boxMax)
{
  // Ray is assumed to be in local coordinates, ie:
  // ray = inverse(objectMatrix * invCameraMatrix) * ray
  // Equation of ray: O + D * t
  vec3 tMin = (boxMin - origin) * inverseDir;
  vec3 tMax = (boxMax - origin) * inverseDir;
  vec3 t1 = min(tMin, tMax);
  vec3 t2 = max(tMin, tMax);
  float tNear = max(max(t1.x, t1.y), t1.z);
  float tFar = min(min(t2.x, t2.y), t2.z);
  return tFar > tNear;
}

float
intersectAABB(Ray ray, vec3 aabbMin, vec3 aabbMax)
{
  return intersectAABB(ray.origin, 1.0 / ray.dir, aabbMin, aabbMax);
}

// TODO: implement watertight version of Ray-Triangle intersection, available
// behind a flag

// Implementation of:
// Möller, Tomas; Trumbore, Ben (1997). "Fast, Minimum Storage Ray-Triangle Intersection"
float
intersectTriangle(Ray ray, uint startIndex, uint vertexOffset, inout vec2 uv)
{
  // TODO: pre-process edge?
  // Maybe not really useful if decide to add skinning in shader.
  vec3 v0 = getVertex(vertexOffset, startIndex).position.xyz;
  vec3 v1 = getVertex(vertexOffset, startIndex + 1).position.xyz;
  vec3 v2 = getVertex(vertexOffset, startIndex + 2).position.xyz;

  vec3 e1 = v1 - v0;
  vec3 e2 = v2 - v0;

  vec3 p = cross(ray.dir, e2);
  float det = dot(e1, p);

  // Ray is parralel to edge.
  if (det <= NEG_EPSILON) { return MAX_FLOAT; }
  if (det > NEG_EPSILON && det < EPSILON) { return MAX_FLOAT; }

  float invDet = 1.0 / det;

  // Computes Barycentric coordinates.
  vec3 centered = ray.origin - v0;

  float u = dot(centered, p) * invDet;
  if (u < EPSILON || u > EPSILON1) { return MAX_FLOAT; }

  vec3 q = cross(centered, e1);
  float v = dot(ray.dir, q) * invDet;
  if (v < EPSILON || u + v > EPSILON1) { return MAX_FLOAT; }

  uv = vec2(u, v);
  return dot(e2, q) * invDet;
}

#define SIMD_AABBTEST

uint sign_extend_s8x4( uint i )
{
	// docs: "with the given parameters, prmt will extend the sign to all bits in a byte."
	// const uint b0 = (i & (1u << 31u)) != 0u ? 0xff000000 : 0u;
	// const uint b1 = (i & (1u << 23u)) != 0u ? 0x00ff0000 : 0u;
	// const uint b2 = (i & (1u << 15u)) != 0u ? 0x0000ff00 : 0u;
	// const uint b3 = (i & (1u << 7u)) != 0u ? 0x000000ff : 0u;

	// return b0 + b1 + b2 + b3; // probably can do better than this.

	return ((i >> 7) & 0x01010101) * 0xff;
}

uint
extract_byte(uint x, uint i)
{
	return (x >> (8u * i)) & 0xFFu;
}

uvec4 as_uchar4(float val) {
	uint bits = floatBitsToUint(val);
	return uvec4(
		bitfieldExtract(bits, 0, 8),
		bitfieldExtract(bits, 8, 8),
		bitfieldExtract(bits, 16, 8),
		bitfieldExtract(bits, 24, 8)
	);
}

uint as_uint(float val) {
	return floatBitsToUint(val);
}

float as_float(uint val) {
	return uintBitsToFloat(val);
}

uint get_oct_inv4(vec3 d)
{
	return (d.x < 0.0 ? 0u : 0x04040404u) |
		   (d.y < 0.0 ? 0u : 0x02020202u) |
		   (d.z < 0.0 ? 0u : 0x01010101u);
}

vec4
traverse_cwbvh(vec3 O, vec3 D, float t, inout uint stepCount)
{
	// initialize ray
	const vec4 O4 = vec4( O, 1.0 );
	const vec4 D4 = vec4( D, 0.0 );
	const vec4 rD4 = vec4( 1.0 / D, 1.0 );

	vec4 hit;
	hit.x = t; // not fetching this from ray data to avoid one memory operation.
	// prepare traversal
	uvec2 stack[STACK_SIZE];
	uint hitAddr;
	uint stackPtr = 0;
	vec2 uv;
	float tmax = t;
	const uint octinv4 = (7 - ((D.x < 0 ? 4 : 0) | (D.y < 0 ? 2 : 0) | (D.z < 0 ? 1 : 0))) * 0x1010101;

	uvec2 ngroup = uvec2(0u, 1u << 31u);
	uvec2 tgroup = uvec2(0);

	do
	{
		#ifdef DEBUG_CWBVH_TRAVERSAL
		++stepCount;
		#endif

		if (ngroup.y > 0x00FFFFFF)
		{
			uint hits = ngroup.y;
			uint imask = ngroup.y;
			int child_bit_index = findMSB( hits );
			uint child_node_base_index = ngroup.x;
			ngroup.y &= ~(1 << child_bit_index);
			if (ngroup.y > 0x00FFFFFF)
			{
				stack[stackPtr++] = ngroup;
			}

			const uint slot_index = (child_bit_index - 24) ^ (octinv4 & 255);
			const uint relative_index = bitCount( imask & ~(0xFFFFFFFF << slot_index) );
			const uint child_node_index = (child_node_base_index + relative_index) * 5;

			// @TODO: Offset node with BLAS offset.
			vec4 n0 = nodes[child_node_index + 0];
			vec4 n1 = nodes[child_node_index + 1];
			vec4 n2 = nodes[child_node_index + 2];
			vec4 n3 = nodes[child_node_index + 3];
			vec4 n4 = nodes[child_node_index + 4];

			ngroup.x = as_uint( n1.x );
			tgroup = uvec2(as_uint( n1.y ), 0u);
			uint hitmask = 0;

			uvec3 e = uvec3 (bitfieldExtract(as_uint(n0.w), 0, 8), bitfieldExtract(as_uint(n0.w), 8, 8), bitfieldExtract(as_uint(n0.w), 16, 8));
			e = uvec3 ((e.x + 127) & 0xFF, (e.y + 127) & 0xFF, (e.z + 127) & 0xFF);
			vec4 idir4 = vec4(
				as_float( e.x << 23u ) * rD4.x,
				as_float( e.y << 23u ) * rD4.y,
				as_float( e.z << 23u ) * rD4.z,
				1.0
			);
			const vec4 orig4 = (n0 - O4) * rD4;

			{	// first 4
				uint meta4 = as_uint( n1.z );
				uint is_inner4 = (meta4 & (meta4 << 1)) & 0x10101010;
				uint inner_mask4 = sign_extend_s8x4( is_inner4 << 3 );
				uint bit_index4 = (meta4 ^ (octinv4 & inner_mask4)) & 0x1F1F1F1F;
				uint child_bits4 = (meta4 >> 5) & 0x07070707;
				vec4 lox4 = vec4( as_uchar4( rD4.x < 0 ? n3.z : n2.x ) );
				vec4 hix4 = vec4( as_uchar4( rD4.x < 0 ? n2.x : n3.z ) );
				vec4 loy4 = vec4( as_uchar4( rD4.y < 0 ? n4.x : n2.z ) );
				vec4 hiy4 = vec4( as_uchar4( rD4.y < 0 ? n2.z : n4.x ) );
				vec4 loz4 = vec4( as_uchar4( rD4.z < 0 ? n4.z : n3.x ) );
				vec4 hiz4 = vec4( as_uchar4( rD4.z < 0 ? n3.x : n4.z ) );
				{
					vec4 tminx4 = lox4 * idir4.xxxx + orig4.xxxx, tmaxx4 = hix4 * idir4.xxxx + orig4.xxxx;
					vec4 tminy4 = loy4 * idir4.yyyy + orig4.yyyy, tmaxy4 = hiy4 * idir4.yyyy + orig4.yyyy;
					vec4 tminz4 = loz4 * idir4.zzzz + orig4.zzzz, tmaxz4 = hiz4 * idir4.zzzz + orig4.zzzz;
					float cmina = max( max( max( tminx4.x, tminy4.x ), tminz4.x ), 0 );
					float cmaxa = min( min( min( tmaxx4.x, tmaxy4.x ), tmaxz4.x ), tmax );
					float cminb = max( max( max( tminx4.y, tminy4.y ), tminz4.y ), 0 );
					float cmaxb = min( min( min( tmaxx4.y, tmaxy4.y ), tmaxz4.y ), tmax );
					if (cmina <= cmaxa) {
						hitmask = (child_bits4 & 255) << (bit_index4 & 31);
					}
					if (cminb <= cmaxb) {
						hitmask |= ((child_bits4 >> 8) & 255) << ((bit_index4 >> 8) & 31);
					}
					float cminc = max( max( max( tminx4.z, tminy4.z ), tminz4.z ), 0 );
					float cmaxc = min( min( min( tmaxx4.z, tmaxy4.z ), tmaxz4.z ), tmax );
					float cmind = max( max( max( tminx4.w, tminy4.w ), tminz4.w ), 0 );
					float cmaxd = min( min( min( tmaxx4.w, tmaxy4.w ), tmaxz4.w ), tmax );
					if (cminc <= cmaxc) {
						hitmask |= ((child_bits4 >> 16) & 255) << ((bit_index4 >> 16) & 31);
					}
					if (cmind <= cmaxd) {
						hitmask |= (child_bits4 >> 24) << (bit_index4 >> 24);
					}
				}
			}
			{	// second 4
				uint meta4 = as_uint( n1.w );
				uint is_inner4 = (meta4 & (meta4 << 1)) & 0x10101010;
				uint inner_mask4 = sign_extend_s8x4( is_inner4 << 3 );
				uint bit_index4 = (meta4 ^ (octinv4 & inner_mask4)) & 0x1F1F1F1F;
				uint child_bits4 = (meta4 >> 5) & 0x07070707;
				vec4 lox4 = vec4( as_uchar4( rD4.x < 0 ? n3.w : n2.y ) );
				vec4 hix4 = vec4( as_uchar4( rD4.x < 0 ? n2.y : n3.w ) );
				vec4 loy4 = vec4( as_uchar4( rD4.y < 0 ? n4.y : n2.w ) );
				vec4 hiy4 = vec4( as_uchar4( rD4.y < 0 ? n2.w : n4.y ) );
				vec4 loz4 = vec4( as_uchar4( rD4.z < 0 ? n4.w : n3.y ) );
				vec4 hiz4 = vec4( as_uchar4( rD4.z < 0 ? n3.y : n4.w ) );
				{
					const vec4 tminx4 = lox4 * idir4.xxxx + orig4.xxxx, tmaxx4 = hix4 * idir4.xxxx + orig4.xxxx;
					const vec4 tminy4 = loy4 * idir4.yyyy + orig4.yyyy, tmaxy4 = hiy4 * idir4.yyyy + orig4.yyyy;
					const vec4 tminz4 = loz4 * idir4.zzzz + orig4.zzzz, tmaxz4 = hiz4 * idir4.zzzz + orig4.zzzz;
					const float cmina = max( max( max( tminx4.x, tminy4.x ), tminz4.x ), 0 );
					const float cmaxa = min( min( min( tmaxx4.x, tmaxy4.x ), tmaxz4.x ), tmax );
					const float cminb = max( max( max( tminx4.y, tminy4.y ), tminz4.y ), 0 );
					const float cmaxb = min( min( min( tmaxx4.y, tmaxy4.y ), tmaxz4.y ), tmax );
					if (cmina <= cmaxa) {
						hitmask |= (child_bits4 & 255) << (bit_index4 & 31);
					}
					if (cminb <= cmaxb) {
						hitmask |= ((child_bits4 >> 8) & 255) << ((bit_index4 >> 8) & 31);
					}
					float cminc = max( max( max( tminx4.z, tminy4.z ), tminz4.z ), 0 );
					float cmaxc = min( min( min( tmaxx4.z, tmaxy4.z ), tmaxz4.z ), tmax );
					float cmind = max( max( max( tminx4.w, tminy4.w ), tminz4.w ), 0 );
					float cmaxd = min( min( min( tmaxx4.w, tmaxy4.w ), tmaxz4.w ), tmax );
					if (cminc <= cmaxc) {
						hitmask |= ((child_bits4 >> 16) & 255) << ((bit_index4 >> 16) & 31);
					}
					if (cmind <= cmaxd) {
						hitmask |= (child_bits4 >> 24) << (bit_index4 >> 24);
					}
				}
			}

			uint mask = extract_byte(as_uint(n0.w), 3);
			ngroup.y = (hitmask & 0xFF000000) | mask;
			tgroup.y = hitmask & 0x00FFFFFF;
		}
		else
		{
			tgroup = ngroup;
			ngroup = uvec2(0u);
		}

		while (tgroup.y != 0u)
		{
			// M�ller-Trumbore intersection; triangles are stored as 3x16 bytes,
			// with the original primitive index in the (otherwise unused) w
			// component of vertex 0.
			int triangleIndex = findMSB( tgroup.y );
			uint triAddr = tgroup.x + triangleIndex * 3;
			vec3 e1 = trianglesCWBVH[triAddr].xyz;
			vec3 e2 = trianglesCWBVH[triAddr + 1].xyz;
			vec4 v0 = trianglesCWBVH[triAddr + 2];
			tgroup.y -= 1 << triangleIndex;
			vec3 r = cross( D.xyz, e1 );
			float a = dot( e2, r );
			if (abs( a ) < EPSILON) continue;
			float f = 1.0 / a;
			vec3 s = O.xyz - v0.xyz;
			float u = f * dot( s, r );
			if (u < EPSILON || u > EPSILON1) continue;
			vec3 q = cross( s, e2 );
			float v = f * dot( D.xyz, q );
			if (v < EPSILON || u + v > EPSILON1) continue;
			float d = f * dot( e1, q );
			if (d <= EPSILON || d >= tmax) continue;
			uv = vec2(u, v);
			tmax = d;
			hitAddr = as_uint( v0.w );
		}

		if (ngroup.y <= 0x00FFFFFFu)
		{
			if (stackPtr > 0)
			{
				ngroup = stack[--stackPtr];
			}
			else
			{
				hit = vec4(tmax, uv.x, uv.y, as_float( hitAddr ));
				break;
			}
		}
	} while (true);

	return hit;
}

float
sceneHit(Ray ray, inout Intersection intersection)
{
  float dist = MAX_FLOAT;
  uint stepCount = 0;
  for (uint i = 0; i < instances.length(); ++i)
  {
    Instance instance = instances[i];

    // Performs intersection in model space.
    Ray rayModel = transformRay(ray, instance.worldToModel);
	vec4 hit = traverse_cwbvh(rayModel.origin, rayModel.dir, dist, stepCount);
	if (hit.x > 0.0 && hit.x < dist)
    {
		intersection.uv = hit.yz;
		intersection.index = as_uint(hit.w) * 3;
		intersection.instance = i;
		intersection.emitter = INVALID_UINT;
		intersection.materialIndex = instance.materialIndex;
      	dist = hit.x;
    }
  }
  return dist;
}

uint sceneTraversal(Ray ray)
{
	uint stepCount = 0;
	for (uint i = 0; i < instances.length(); ++i)
  	{
		Instance instance = instances[i];
		Ray rayModel = transformRay(ray, instance.worldToModel);
		traverse_cwbvh(rayModel.origin, rayModel.dir, MAX_FLOAT, stepCount);
	}
	return stepCount;
}

#endif // INTERSECTION_UTILS_H
