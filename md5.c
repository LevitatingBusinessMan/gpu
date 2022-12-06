/* Basic MD5 functions */
#define F(x, y, z) ((x & y) | (~x & z))
#define G(x, y, z) ((x & z) | (y & ~z))
#define H(x, y, z) (x ^ y ^ z)
#define I(x, y, z) (y ^ (x | ~z))

/* ROTATE_LEFT rotates x left n bits */
#define ROTATE_LEFT(x, n) (((x) << (n)) | ((x) >> (32-(n))))

// message is 512 bits (should be 16 ints)
// padded like message + 1 + many zeros + 64bit-length
void md5 (char* s, int* k, char* message, int* result) {
	int A, a0;
	A = a0 = 0x67452301;
	int B, b0;
	B = b0 = 0xefcdab89;
	int C, c0;
	C = c0 = 0x98badcfe;
	int D, d0;
	D = d0 = 0x10325476;

	for (int i=0; i < 64; i++) {
		int f, g;
		if (0 <= i <= 15) {
			f = F(B, C, D);
			g = i; 
		}
		else if (16 <= i <= 31) {
			f = G(B, C, D);
			g = (5 * i + 1) % 16;
		}
		else if (32 <= i <= 47) {
			f = H(B, C, D);
			g = (3 * i + 5) % 16;
		}
		else if (48 <= i <= 63) {
			f = I(B, C, D);
			g = (7 * i) % 16;
		}
		f = f + A + k[i] + message[g];
		A = D;
		D = C;
		C = B;
		B = B + ROTATE_LEFT(f, s[i]);
	}


	result[0] = a0 + A;
	result[1] = b0 + B;
	result[2] = c0 + C;
	result[3] = d0 + D;
}

void main() {
	int s[] = { 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21};

	int k[] = {
			0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee,
			0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
			0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be,
			0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
			0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa,
			0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
			0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed,
			0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
			0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c,
			0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
			0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05,
			0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
			0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039,
			0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
			0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1,
			0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391,
	};

	int message[] = {
		0xb00bb00b, 0xb00bb00b, 0xb00bb00b, 0xb00bb00c,
		0x00000000, 0x00000000, 0x00000000, 0x00000000,
		0x00000000, 0x00000000, 0x00000000, 0x00000000,
		0x00000000, 0x00000000, 0x00000000, 0x00000020,
	};

	int result[4] = {0};

	md5(s,k,message,result);
}
