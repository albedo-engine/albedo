#define PI_F 3.14159265359
#define TWO_PI 6.28318530718
#define TO_RAD_F (PI_F / 180.0)
#define MAX_FLOAT 3.402823466e+38

#define VEC3_UP vec3(0.0, 0.999999995, 0.0001)

#define MAX_UINT 0xFFFFFFFF
#define INVALID_UINT MAX_UINT

uint
WangHash(inout uint seed)
{
  seed = uint(seed ^ uint(61)) ^ uint(seed >> uint(16));
  seed *= uint(9);
  seed = seed ^ (seed >> 4);
  seed *= uint(0x27d4eb2d);
  seed = seed ^ (seed >> 15);
  return seed;
}

float
rand(inout uint seed)
{
  return float(WangHash(seed)) / 4294967296.0;
}
