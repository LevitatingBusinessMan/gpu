#define F(b, c, d) ((b & c) | (~b & d))
#define G(b, c, d) ((b & d) | (c & ~d))
#define H(b, c, d) (b ^ c ^ d)
#define I(b, c, d) (c ^ (b | ~d))

#define ROTATE_LEFT(x, n) (x << n | x >> (32-n)) 

// message is 512 bits (should be 16 ints)
// padded like message + 1 + many zeros + 64bit-length
kernel void md5(global uchar* s, global uint* k, global uint* message, global uint* digest) {
	unsigned int A, a0;
	A = a0 = 0x67452301;
	unsigned int B, b0;
	B = b0 = 0xefcdab89;
	unsigned int C, c0;
	C = c0 = 0x98badcfe;
	unsigned int D, d0;
	D = d0 = 0x10325476;

	for (int i=0; i < 64; i++) {
		unsigned int f, g;
		if (i < 16) {
			f = F(B, C, D);
			g = i; 
		}
		else if (i < 32) {
			f = G(B, C, D);
			g = (5 * i + 1) % 16;
		}
		else if (i < 48) {
			f = H(B, C, D);
			g = (3 * i + 5) % 16;
		}
		else if (i < 64) {
			f = I(B, C, D);
			g = (7 * i) % 16;
		}
		f = f + A + k[i] + message[g];
		A = D;
		D = C;
		C = B;
		B = B + ROTATE_LEFT(f, s[i]);
	}

	digest[0] = a0 + A;
	digest[1] = b0 + B;
	digest[2] = c0 + C;
	digest[3] = d0 + D;
}
