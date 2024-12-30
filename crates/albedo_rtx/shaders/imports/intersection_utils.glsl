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
	return (x >> (8 * i)) & 0xff;
}

uvec4 as_uchar4(float val) {
	// @todo: Check.
	uint bits = floatBitsToUint(val);
    return uvec4(
        (bits >> 0u) & 0xFFu,
        (bits >> 8u) & 0xFFu,
        (bits >> 16u) & 0xFFu,
        (bits >> 24u) & 0xFFu
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
	return (d.x < 0.0f ? 0u : 0x04040404u) |
		   (d.y < 0.0f ? 0u : 0x02020202u) |
		   (d.z < 0.0f ? 0u : 0x01010101u);
}

struct BVH8node
{
	vec4 node0;
	vec4 node1;
	vec4 node2;
	vec4 node3;
	vec4 node4;
};

/*
uint bvh8_node_intersect(Ray r, vec3 inv_direction, uint oct_inv4, float max_t, BVH8node node)
{
	vec3 p = node.node0.xyz;

	uint e_imask = floatBitsToUint(node.node0.w);
	uint e_x = extract_byte(e_imask, 0u);
	uint e_y = extract_byte(e_imask, 1u);
	uint e_z = extract_byte(e_imask, 2u);

	vec3 adjust_ray_direction_inv = vec3(uintBitsToFloat(e_x << 23u) * inv_direction.x,
										 uintBitsToFloat(e_y << 23u) * inv_direction.y,
										 uintBitsToFloat(e_z << 23u) * inv_direction.z);

	vec3 adjust_ray_origin = (p - r.origin) * inv_direction;

	uint hit_mask = 0u;

	for(int i = 0; i < 2; ++i)
	{
		uint meta4 = floatBitsToUint(i == 0 ? node.node1.z : node.node1.w);

		uint is_inner4 = (meta4 & (meta4 << 1u)) & 0x10101010u;
		uint inner_mask4 = sign_extend_s8x4(is_inner4 << 3u);

		uint bit_index4 = (meta4 ^ (oct_inv4 & inner_mask4)) & 0x1F1F1F1Fu;

		uint child_bits4 = (meta4 >> 5u) & 0x07070707u;

		//near far
		uint q_lo_x = floatBitsToUint(i == 0 ? node.node2.x : node.node2.y);
		uint q_hi_x = floatBitsToUint(i == 0 ? node.node2.z : node.node2.w);

		uint q_lo_y = floatBitsToUint(i == 0 ? node.node3.x : node.node3.y);
		uint q_hi_y = floatBitsToUint(i == 0 ? node.node3.z : node.node3.w);

		uint q_lo_z = floatBitsToUint(i == 0 ? node.node4.x : node.node4.y);
		uint q_hi_z = floatBitsToUint(i == 0 ? node.node4.z : node.node4.w);

		uint x_min = r.dir.x < 0.0f ? q_hi_x : q_lo_x;
		uint x_max = r.dir.x < 0.0f ? q_lo_x : q_hi_x;

		uint y_min = r.dir.y < 0.0f ? q_hi_y : q_lo_y;
		uint y_max = r.dir.y < 0.0f ? q_lo_y : q_hi_y;

		uint z_min = r.dir.z < 0.0f ? q_hi_z : q_lo_z;
		uint z_max = r.dir.z < 0.0f ? q_lo_z : q_hi_z;

		//intersect 4 children
		//1 node have 2 children
		//each children have another 4 children or childs! (childs sound like child ass! that's why it's called children!)
		for(int j = 0; j < 4; ++j)
		{
			vec3 tmin3 = vec3(float(extract_byte(x_min, j)), float(extract_byte(y_min, j)),float(extract_byte(z_min, j)));
			vec3 tmax3 = vec3(float(extract_byte(x_max, j)), float(extract_byte(y_max, j)),float(extract_byte(z_max, j)));

			tmin3 = tmin3 * adjust_ray_direction_inv + adjust_ray_origin;
			tmax3 = tmax3 * adjust_ray_direction_inv + adjust_ray_origin;

			float tmin = max(max(tmin3.x, tmin3.y), tmin3.z);
			float tmax = max(max(tmax3.x, tmax3.y), tmax3.z);

			if(tmax >= 0.0f && tmax < max_t && tmin <= tmax)
			{
				uint child_bits = extract_byte(child_bits4, j);
				uint bit_index = extract_byte(bit_index4, j);
				hit_mask |= child_bits << bit_index;
			}
		}
	}
	return hit_mask;
}
*/

uint bvh8_node_intersect2(Ray r, vec3 inv_direction, uint oct_inv4, float max_t, BVH8node node)
{
	vec3 p = node.node0.xyz;

	uint e_imask = floatBitsToUint(node.node0.w);
	uint e_x = extract_byte(e_imask, 0u);
	uint e_y = extract_byte(e_imask, 1u);
	uint e_z = extract_byte(e_imask, 2u);

	vec3 adjust_ray_direction_inv = vec3(uintBitsToFloat(e_x << 23u) * inv_direction.x,
										 uintBitsToFloat(e_y << 23u) * inv_direction.y,
										 uintBitsToFloat(e_z << 23u) * inv_direction.z);

	vec3 adjust_ray_origin = (p - r.origin) * inv_direction;

	uint hit_mask = 0u;

	for(int i = 0; i < 2; ++i)
	{
		uint meta4 = floatBitsToUint(i == 0 ? node.node1.z : node.node1.w);

		uint is_inner4 = (meta4 & (meta4 << 1u)) & 0x10101010u;
		uint inner_mask4 = sign_extend_s8x4(is_inner4 << 3u);

		uint bit_index4 = (meta4 ^ (oct_inv4 & inner_mask4)) & 0x1F1F1F1Fu;

		uint child_bits4 = (meta4 >> 5u) & 0x07070707u;

		//near far
		uint q_lo_x = floatBitsToUint(i == 0 ? node.node2.x : node.node2.y);
		uint q_hi_x = floatBitsToUint(i == 0 ? node.node2.z : node.node2.w);

		uint q_lo_y = floatBitsToUint(i == 0 ? node.node3.x : node.node3.y);
		uint q_hi_y = floatBitsToUint(i == 0 ? node.node3.z : node.node3.w);

		uint q_lo_z = floatBitsToUint(i == 0 ? node.node4.x : node.node4.y);
		uint q_hi_z = floatBitsToUint(i == 0 ? node.node4.z : node.node4.w);

		uint x_min = r.dir.x < 0.0f ? q_hi_x : q_lo_x;
		uint x_max = r.dir.x < 0.0f ? q_lo_x : q_hi_x;

		uint y_min = r.dir.y < 0.0f ? q_hi_y : q_lo_y;
		uint y_max = r.dir.y < 0.0f ? q_lo_y : q_hi_y;

		uint z_min = r.dir.z < 0.0f ? q_hi_z : q_lo_z;
		uint z_max = r.dir.z < 0.0f ? q_lo_z : q_hi_z;

		//intersect 4 children
		//1 node have 2 children
		//each children have another 4 children or childs! (childs sound like child ass! that's why it's called children!)
		for(int j = 0; j < 4; ++j)
		{
			vec3 tmin3 = vec3(float(extract_byte(x_min, j)), float(extract_byte(y_min, j)),float(extract_byte(z_min, j)));
			vec3 tmax3 = vec3(float(extract_byte(x_max, j)), float(extract_byte(y_max, j)),float(extract_byte(z_max, j)));

			tmin3 = tmin3 * adjust_ray_direction_inv + adjust_ray_origin;
			tmax3 = tmax3 * adjust_ray_direction_inv + adjust_ray_origin;

			float tmin = max(max(tmin3.x, tmin3.y), tmin3.z);
			float tmax = max(max(tmax3.x, tmax3.y), tmax3.z);

			if(tmax >= 0.0f && tmax < max_t && tmin <= tmax)
			{
				uint child_bits = extract_byte(child_bits4, j);
				uint bit_index = extract_byte(bit_index4, j);
				hit_mask |= child_bits << bit_index;
			}
		}
	}
	return hit_mask;
}

#define TINYBVH
#ifdef TINYBVH
vec4
traverse_cwbvh(vec3 O, vec3 D, vec3 rD, float t, inout uint stepCount)
{
	// initialize ray
	const vec4 O4 = vec4( O, 1.0 ); // rayData[threadId].O;
	const vec4 D4 = vec4( D, 0.0 ); // rayData[threadId].D;
	const vec4 rD4 = vec4( rD, 1.0 ); // rayData[threadId].rD;

	vec4 hit;
	hit.x = t; // not fetching this from ray data to avoid one memory operation.
	// prepare traversal
	uvec2 stack[STACK_SIZE];
	uint hitAddr;
	uint stackPtr = 0;
	vec2 uv;
	float tmax = t;
	// const uint octinv4 = (7 - ((D.x < 0 ? 4 : 0) | (D.y < 0 ? 2 : 0) | (D.z < 0 ? 1 : 0))) * 0x1010101;
	uint octinv4 = get_oct_inv4(D);

	// uvec2 ngroup = uvec2(0u, 0b10000000000000000000000000000000);
	uvec2 ngroup = uvec2(0u, 1u << 31u); // @todo: Check.
	uvec2 tgroup = uvec2(0);

	do
	{
		#ifdef DEBUG_CWBVH_TRAVERSAL
		++stepCount;
		// if(stepCount > 2000) return hit;
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
			{
				const uint slot_index = (child_bit_index - 24) ^ (octinv4 & 255);
				const uint relative_index = bitCount( imask & ~(0xFFFFFFFF << slot_index) );
				const uint child_node_index = (child_node_base_index + relative_index) * 5;

				// @TODO: Offset node with BLAS offset.
				vec4 n0 = nodes[child_node_index + 0];
				vec4 n1 = nodes[child_node_index + 1];
				vec4 n2 = nodes[child_node_index + 2];
				vec4 n3 = nodes[child_node_index + 3];
				vec4 n4 = nodes[child_node_index + 4];

				uvec4 e = as_uchar4(n0.w);

				ngroup.x = as_uint( n1.x );
				tgroup = uvec2(as_uint( n1.y ), 0u);
				uint hitmask = 0;

				/*vec4 idir4 = vec4(
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
					vec4 lox4 = vec4( as_uchar4( rD.x < 0 ? n3.z : n2.x ) );
					vec4 hix4 = vec4( as_uchar4( rD.x < 0 ? n2.x : n3.z ) );
					vec4 loy4 = vec4( as_uchar4( rD.y < 0 ? n4.x : n2.z ) );
					vec4 hiy4 = vec4( as_uchar4( rD.y < 0 ? n2.z : n4.x ) );
					vec4 loz4 = vec4( as_uchar4( rD.z < 0 ? n4.z : n3.x ) );
					vec4 hiz4 = vec4( as_uchar4( rD.z < 0 ? n3.x : n4.z ) );
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
							const uint child_bits = extract_byte(child_bits4, 0);
							const uint bit_index = extract_byte(bit_index4, 0);
							// hitmask |= child_bits << bit_index;
							stepCount = 1;
						}
						if (cminb <= cmaxb) {
							hitmask |= ((child_bits4 >> 8) & 255) << ((bit_index4 >> 8) & 31);
							// const uint child_bits = extract_byte(child_bits4, 1);
							// const uint bit_index = extract_byte(bit_index4, 1);
							// hitmask |= child_bits << bit_index;
						}
						float cminc = max( max( max( tminx4.z, tminy4.z ), tminz4.z ), 0 );
						float cmaxc = min( min( min( tmaxx4.z, tmaxy4.z ), tmaxz4.z ), tmax );
						float cmind = max( max( max( tminx4.w, tminy4.w ), tminz4.w ), 0 );
						float cmaxd = min( min( min( tmaxx4.w, tmaxy4.w ), tmaxz4.w ), tmax );
						if (cminc <= cmaxc) {
							hitmask |= ((child_bits4 >> 16) & 255) << ((bit_index4 >> 16) & 31);
							// const uint child_bits = extract_byte(child_bits4, 2);
							// const uint bit_index = extract_byte(bit_index4, 2);
							// hitmask |= child_bits << bit_index;
						}
						if (cmind <= cmaxd) {
							hitmask |= (child_bits4 >> 24) << (bit_index4 >> 24);
							// const uint child_bits = extract_byte(child_bits4, 3);
							// const uint bit_index = extract_byte(bit_index4, 3);
							// hitmask |= child_bits << bit_index;
						}
					}
				}
				{	// second 4
					uint meta4 = as_uint( n1.w );
					uint is_inner4 = (meta4 & (meta4 << 1)) & 0x10101010;
					uint inner_mask4 = sign_extend_s8x4( is_inner4 << 3 );
					uint bit_index4 = (meta4 ^ (octinv4 & inner_mask4)) & 0x1F1F1F1F;
					uint child_bits4 = (meta4 >> 5) & 0x07070707;
					vec4 lox4 = vec4( as_uchar4( rD.x < 0 ? n3.w : n2.y ) );
					vec4 hix4 = vec4( as_uchar4( rD.x < 0 ? n2.y : n3.w ) );
					vec4 loy4 = vec4( as_uchar4( rD.y < 0 ? n4.y : n2.w ) );
					vec4 hiy4 = vec4( as_uchar4( rD.y < 0 ? n2.w : n4.y ) );
					vec4 loz4 = vec4( as_uchar4( rD.z < 0 ? n4.w : n3.y ) );
					vec4 hiz4 = vec4( as_uchar4( rD.z < 0 ? n3.y : n4.w ) );
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
							// const uint child_bits = extract_byte(child_bits4, 0);
							// const uint bit_index = extract_byte(bit_index4, 0);
							// hitmask |= child_bits << bit_index;
						}
						if (cminb <= cmaxb) {
							hitmask |= ((child_bits4 >> 8) & 255) << ((bit_index4 >> 8) & 31);
							// const uint child_bits = extract_byte(child_bits4, 1);
							// const uint bit_index = extract_byte(bit_index4, 1);
							// hitmask |= child_bits << bit_index;
						}
						float cminc = max( max( max( tminx4.z, tminy4.z ), tminz4.z ), 0 );
						float cmaxc = min( min( min( tmaxx4.z, tmaxy4.z ), tmaxz4.z ), tmax );
						float cmind = max( max( max( tminx4.w, tminy4.w ), tminz4.w ), 0 );
						float cmaxd = min( min( min( tmaxx4.w, tmaxy4.w ), tmaxz4.w ), tmax );
						if (cminc <= cmaxc) {
							hitmask |= ((child_bits4 >> 16) & 255) << ((bit_index4 >> 16) & 31);
							// const uint child_bits = extract_byte(child_bits4, 2);
							// const uint bit_index = extract_byte(bit_index4, 2);
							// hitmask |= child_bits << bit_index;
						}
						if (cmind <= cmaxd) {
							hitmask |= (child_bits4 >> 24) << (bit_index4 >> 24);
							// const uint child_bits = extract_byte(child_bits4, 3);
							// const uint bit_index = extract_byte(bit_index4, 3);
							// hitmask |= child_bits << bit_index;
						}
					}
				}*/

				// DEBUG
				BVH8node node;
				node.node0 = n0;
				node.node1 = n1;
				node.node2 = n2;
				node.node3 = n3;
				node.node4 = n4;
				Ray r;
				r.origin = O;
				r.dir = D;
				hitmask = bvh8_node_intersect(r, rD.xyz, octinv4, tmax, node);
				// END DEBUG

				uint mask = extract_byte(as_uint(n0.w), 3);
				ngroup.y = (hitmask & 0xFF000000) | mask;
				tgroup.y = hitmask & 0x00FFFFFF;
			}
		}
		else {
			tgroup = ngroup;
			ngroup = uvec2(0u);
		}

		while (tgroup.y != 0)
		{
			// M�ller-Trumbore intersection; triangles are stored as 3x16 bytes,
			// with the original primitive index in the (otherwise unused) w
			// component of vertex 0.
			int triangleIndex = findMSB( tgroup.y );
			uint triAddr = tgroup.x + triangleIndex * 3u;
			// TODO: Offset with base
			vec3 v0 = trianglesCWBVH[triAddr].xyz;
			vec3 e1 = trianglesCWBVH[triAddr + 1].xyz;
			vec3 e2 = trianglesCWBVH[triAddr + 2].xyz;
			vec3 r = cross( D4.xyz, e2 );

			float a = dot( e1, r );
			if (abs( a ) > 0.0000001)
			{
				float f = 1 / a;
				vec3 s = O4.xyz - v0;
				float u = f * dot( s, r );
				if (u >= 0.0 && u <= 1.0)
				{
					vec3 q = cross( s, e1 );
					float v = f * dot( D4.xyz, q );
					if (v >= 0.0 && u + v <= 1.0)
					{
						const float d = f * dot( e2, q );
						if (d > 0.0 && d < tmax)
						{
							uv = vec2(u, v);
							tmax = d;
							hitAddr = as_uint( trianglesCWBVH[triAddr].w );
						}
					}
				}
			}
			tgroup.y -= 1 << triangleIndex;
		}
		if (ngroup.y <= 0x00FFFFFF)
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

#else

vec4 hit_triangle(Ray r, int i, float t)
{
	// vec3 v0 = texelFetch(vertices_tex, triIndex.x).xyz; // int(triIndex.x)
	// vec3 v1 = texelFetch(vertices_tex, triIndex.y).xyz;
	// vec3 v2 = texelFetch(vertices_tex, triIndex.z).xyz;
	vec3 v0 = trianglesCWBVH[i].xyz;
	vec3 v1 = trianglesCWBVH[i + 1].xyz;
	vec3 v2 = trianglesCWBVH[i + 2].xyz;

	// v1 -= v0;
	// v2 -= v0;

	vec3 pv = cross(r.dir, v2);


	vec3 tv = r.origin - v0;
	vec3 qv = cross(tv, v1);

	vec4 uvt;
	uvt.x = dot(tv, pv);//u
	//if(uvt.x < 0.0f)
	//	return false;

	uvt.y = dot(r.dir, qv);//v
	//if(uvt.y < 0.0f)
	//	return false;

	uvt.z = dot(v2, qv);//t
	//float det = dot(e0, pv);
	float inv_det = 1.0f / dot(v1, pv);//det;

	uvt.xyz = uvt.xyz * inv_det;
	uvt.w = 1.0 - uvt.x - uvt.y;//u + v >= det


	if (all(greaterThanEqual(uvt, vec4(0.0f))) && uvt.z < t)
	{
		// t = uvt.z;
		// //rec.t = t;
		// rec.u = uvt.x;
		// rec.v = uvt.y;
		// rec.triangle_ind = i;
		// rec.mtl_ind = (triIndex.w);
		return vec4(t, uvt.x, uvt.y, trianglesCWBVH[i].w);
	}
	return vec4(MAX_FLOAT, vec3(0.0));
}

vec4 traverse_cwbvh(Ray r, inout uint stepCount)
{
	uvec2 local_stack[STACK_SIZE];

	int stack_size = 0;

	uint oct_inv4 = get_oct_inv4(r.dir);

	uvec2 current_group = uvec2(0u, 0x80000000u);

	vec3 inv_direction = vec3(1.0) / r.dir;

	vec4 hit = vec4(MAX_FLOAT, vec3(0.0));

	int test = 0;

	while(stack_size > 0 || current_group.y != 0u)
	{
		uvec2 triangle_group = uvec2(0u);

		if(stepCount > 10000) return hit;
		if(test > 10000) return hit;

		if((current_group.y & 0xff000000u) != 0u)
		{
			uint hits_imask = current_group.y;
			int child_index_offset = findMSB(hits_imask);
			uint child_index_base = current_group.x;

			current_group.y &= ~(1u << child_index_offset);

			if((current_group.y & 0xff000000u) != 0u)
			{
				local_stack[stack_size++] = current_group;
				++stepCount;
			}

			uint slot_index = (child_index_offset - 24u) ^ (oct_inv4 & 0xffu);
			uint relative_index = bitCount(hits_imask & ~(0xffffffffu << slot_index));

			uint child_node_index = child_index_base + relative_index;

			BVH8node node;
			node.node0 = nodes[child_node_index * 5 + 0];
			node.node1 = nodes[child_node_index * 5 + 1];
			node.node2 = nodes[child_node_index * 5 + 2];
			node.node3 = nodes[child_node_index * 5 + 3];
			node.node4 = nodes[child_node_index * 5 + 4];

			uint hitmask = bvh8_node_intersect(r, inv_direction, oct_inv4, hit.x, node);

			uint imask = extract_byte(floatBitsToUint(node.node0.w), 3u);

			current_group.x = floatBitsToUint(node.node1.x);
			triangle_group.x = floatBitsToUint(node.node1.y);

			current_group.y = (hitmask & 0xff000000u) | imask;
			triangle_group.y = (hitmask & 0x00ffffffu);
		}
		else
		{
			triangle_group = current_group;
			current_group = uvec2(0u);
		}

		while(triangle_group.y != 0u)
		{
			int triangle_ind = findMSB(triangle_group.y);
			triangle_group.y &= ~(1u << uint(triangle_ind));

			// int tri_idx = int(triangle_group.x + triangle_ind);
			int tri_idx = int(triangle_group.x) + triangle_ind * 3;

			//bool hit_triangle(Ray r, int i, inout HitRecord rec, inout float t)

			vec4 h = hit_triangle(r, tri_idx, hit.x);
			if(h.x < hit.x) {
				hit = h;
			}
		}

		if((current_group.y & 0xff000000u) == 0u)
		{
			if(stack_size == 0)
				break;
			current_group = local_stack[stack_size--];
		}

		test++;
	}
	return hit;
}
#endif

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
    vec3 rayInverseDir = 1.0 / rayModel.dir;

	#ifdef TINYBVH
	vec4 hit = traverse_cwbvh(rayModel.origin, rayModel.dir, rayInverseDir, dist, stepCount);
	#else
	vec4 hit = traverse_cwbvh(rayModel, stepCount);
	#endif
	if (hit.x > 0.0 && hit.x < dist)
    {
		intersection.uv = hit.yz;
		intersection.index = as_uint(hit.w);
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

		#ifdef TINYBVH
    	vec3 rayInverseDir = 1.0 / rayModel.dir;
		traverse_cwbvh(rayModel.origin, rayModel.dir, rayInverseDir, MAX_FLOAT, stepCount);
		#else
		traverse_cwbvh(rayModel, stepCount);
		#endif
	}
	return stepCount;
}

#endif // INTERSECTION_UTILS_H
