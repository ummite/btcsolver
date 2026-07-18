/*
 * secp256k1 batch public key derivation — complete CUDA kernel.
 *
 * Pipeline: GPU derives compressed pubkey from privkey; CPU does hashing + FlatIndex lookup.
 *
 * Architecture:
 *   - Raw field arithmetic (8 × 32-bit limbs, no Montgomery form)
 *   - Projective point coordinates (x, y, z)
 *   - Precomputed affine table of odd multiples of G (on-device init)
 *   - Double-and-add scalar multiplication
 *   - Multi-GPU: one CUDA context per device
 *
 * Compile (Windows):
 *   nvcc -O3 -arch=sm_61 -Xcompiler /MD -shared -o libsecp_gpu.dll secp256k1_kernel.cu
 *
 * Expected: ~500k-1M keys/sec on GTX 1080 (vs ~120k/sec on CPU)
 */

#include <cuda_runtime.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>

/* ============================================================
 * Constants
 * ============================================================ */

#define FE_LIMBS 8
/* Max threads/block (hardware 1024). Plus de threads/block = mieux occupe les SM. */
#define BLOCK_SIZE 1024

/* secp256k1 prime: p = 2^256 - 2^32 - 977 */
__device__ __constant__ uint32_t CONST_P[FE_LIMBS] = {
    0xFFFFFC2F, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF,
    0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF
};

/* ============================================================
 * Type definitions
 * ============================================================ */

typedef struct __align__(8) { uint32_t v[FE_LIMBS]; } fe;

typedef struct __align__(16) {
    fe x; fe y; fe z;
} point_t;

/* ============================================================
 * Field arithmetic (raw, mod p)
 * ============================================================ */

__device__ __forceinline__ void fe_zero(fe *r) {
    r->v[0] = r->v[1] = r->v[2] = r->v[3] = 0;
    r->v[4] = r->v[5] = r->v[6] = r->v[7] = 0;
}

__device__ __forceinline__ void fe_copy(fe *r, const fe *a) {
    ((uint64_t*)&r->v[0])[0] = ((const uint64_t*)&a->v[0])[0];
    ((uint64_t*)&r->v[0])[1] = ((const uint64_t*)&a->v[0])[1];
    ((uint64_t*)&r->v[0])[2] = ((const uint64_t*)&a->v[0])[2];
    ((uint64_t*)&r->v[0])[3] = ((const uint64_t*)&a->v[0])[3];
}

__device__ __forceinline__ void fe_set1(fe *r) {
    r->v[0] = 1; fe_zero(r); r->v[0] = 1;
}

__device__ __forceinline__ int fe_is_zero(const fe *a) {
    uint64_t t = ((const uint64_t*)&a->v[0])[0] | ((const uint64_t*)&a->v[0])[1] |
                 ((const uint64_t*)&a->v[0])[2] | ((const uint64_t*)&a->v[0])[3];
    return (t == 0) ? -1 : 0;
}

/* Multiply two field elements: r = a * b mod p */
__device__ __forceinline__ void fe_mul(fe *r, const fe *a, const fe *b) {
    uint64_t t[FE_LIMBS * 2] = {0};

    /* Full 8x8 multiplication producing 16 limbs of 32 bits */
    for (int i = 0; i < FE_LIMBS; i++) {
        uint64_t c = 0;
        for (int j = 0; j < FE_LIMBS; j++) {
            c += t[i + j] + (uint64_t)a->v[i] * b->v[j];
            t[i + j] = c & 0xFFFFFFFF;
            c >>= 32;
        }
        t[i + FE_LIMBS] = c;
    }

    /* Reduce mod p using 2^256 ≡ 2^32 + 977 (mod p) */
    /* Fold high limbs into low limbs */
    for (int i = 0; i < FE_LIMBS; i++) {
        uint64_t c = t[i + FE_LIMBS];
        /* c * 2^32: add to t[i+1] and t[i] */
        uint64_t t1 = t[i + 1] + c;
        t[i + 1] = t1 & 0xFFFFFFFF;
        t[i] += (t1 >> 32) + c * 977;
    }

    /* Now t[0..8] may still be >= p, do conditional subtractions */
    /* Normalize and subtract p up to 3 times */
    uint64_t carry = 0;
    for (int i = 0; i < FE_LIMBS; i++) {
        uint64_t s = t[i] + carry;
        t[i] = s & 0xFFFFFFFF;
        carry = s >> 32;
    }
    /* t[8] = carry, need to fold again if nonzero */
    if (t[FE_LIMBS]) {
        uint64_t c = t[FE_LIMBS];
        t[1] += c;
        uint64_t t1 = t[1];
        t[1] = t1 & 0xFFFFFFFF;
        t[0] += (t1 >> 32) + c * 977;
        uint64_t t0 = t[0];
        t[0] = t0 & 0xFFFFFFFF;
        t[1] += t0 >> 32;
    }

    /* Conditional subtraction of p (up to 2 times should suffice) */
    for (int sub = 0; sub < 2; sub++) {
        int need_sub = 0;
        int all_eq = 1;
        for (int i = FE_LIMBS - 1; i >= 0; i--) {
            if (t[i] > CONST_P[i] + 1) { need_sub = 1; all_eq = 0; break; }
            if (t[i] > CONST_P[i]) { all_eq = 1; }
            else { all_eq = 0; break; }
        }
        if (!need_sub && !all_eq) break;

        int borrow = 0;
        for (int i = 0; i < FE_LIMBS; i++) {
            uint64_t s = (uint64_t)t[i] - CONST_P[i] - borrow;
            t[i] = s & 0xFFFFFFFF;
            borrow = (s >> 32) & 1;
        }
    }

    for (int i = 0; i < FE_LIMBS; i++) r->v[i] = (uint32_t)t[i];
}

/* Square: r = a * a mod p (optimized) */
__device__ __forceinline__ void fe_sq(fe *r, const fe *a) {
    uint64_t t[FE_LIMBS * 2] = {0};

    for (int i = 0; i < FE_LIMBS; i++) {
        uint64_t c = 0;
        for (int j = i; j < FE_LIMBS; j++) {
            uint64_t prod = (uint64_t)a->v[i] * a->v[j];
            if (i == j) {
                c += t[i + j] + prod;
                t[i + j] = c & 0xFFFFFFFF;
                c >>= 32;
            } else {
                c += t[i + j] + prod * 2;
                t[i + j] = c & 0xFFFFFFFF;
                c >>= 32;
            }
        }
        t[i + FE_LIMBS] = c;
    }

    /* Same reduction as fe_mul */
    for (int i = 0; i < FE_LIMBS; i++) {
        uint64_t c = t[i + FE_LIMBS];
        uint64_t t1 = t[i + 1] + c;
        t[i + 1] = t1 & 0xFFFFFFFF;
        t[i] += (t1 >> 32) + c * 977;
    }

    uint64_t carry = 0;
    for (int i = 0; i < FE_LIMBS; i++) {
        uint64_t s = t[i] + carry;
        t[i] = s & 0xFFFFFFFF;
        carry = s >> 32;
    }
    if (t[FE_LIMBS]) {
        uint64_t c = t[FE_LIMBS];
        uint64_t t1 = t[1] + c;
        t[1] = t1 & 0xFFFFFFFF;
        t[0] += (t1 >> 32) + c * 977;
        uint64_t t0 = t[0];
        t[0] = t0 & 0xFFFFFFFF;
        t[1] += t0 >> 32;
    }

    for (int sub = 0; sub < 2; sub++) {
        int need_sub = 0;
        for (int i = FE_LIMBS - 1; i >= 0; i--) {
            if (t[i] > CONST_P[i] + 1) { need_sub = 1; break; }
            if (t[i] < CONST_P[i]) { need_sub = 0; break; }
            if (i == 0) need_sub = 1;
        }
        if (!need_sub) break;

        int borrow = 0;
        for (int i = 0; i < FE_LIMBS; i++) {
            uint64_t s = (uint64_t)t[i] - CONST_P[i] - borrow;
            t[i] = s & 0xFFFFFFFF;
            borrow = (s >> 32) & 1;
        }
    }

    for (int i = 0; i < FE_LIMBS; i++) r->v[i] = (uint32_t)t[i];
}

__device__ __forceinline__ void fe_add(fe *r, const fe *a, const fe *b) {
    uint64_t carry = 0;
    for (int i = 0; i < FE_LIMBS; i++) {
        uint64_t s = (uint64_t)a->v[i] + b->v[i] + carry;
        r->v[i] = s & 0xFFFFFFFF;
        carry = s >> 32;
    }
    /* Subtract p only if carry (result >= 2^256, so definitely >= p) */
    if (carry) {
        int borrow = 0;
        for (int i = 0; i < FE_LIMBS; i++) {
            uint64_t s = (uint64_t)r->v[i] - CONST_P[i] - borrow;
            r->v[i] = s & 0xFFFFFFFF;
            borrow = (s >> 32) & 1;
        }
    }
}

__device__ __forceinline__ void fe_sub(fe *r, const fe *a, const fe *b) {
    uint64_t borrow = 0;
    for (int i = 0; i < FE_LIMBS; i++) {
        uint64_t s = (uint64_t)a->v[i] - (uint64_t)b->v[i] - borrow;
        r->v[i] = s & 0xFFFFFFFF;
        borrow = (s >> 63) ? 1 : 0;
    }
    /* Add p if underflow */
    if (borrow) {
        uint64_t carry = 0;
        for (int i = 0; i < FE_LIMBS; i++) {
            uint64_t s = (uint64_t)r->v[i] + CONST_P[i] + carry;
            r->v[i] = s & 0xFFFFFFFF;
            carry = s >> 32;
        }
    }
}

/* ============================================================
 * Point arithmetic (projective coordinates)
 * ============================================================ */

/* Double: r = 2*p (for secp256k1 with a=0, b=7) */
__device__ __forceinline__ void pt_dbl(point_t *r, const point_t *p) {
    /* w = 3*x^2 + a*z^4 = 3*x^2 (since a=0) */
    /* h = 4*x*y^2 */
    /* r = 8*y^4 */
    /* x' = w^2 - 2*h */
    /* y' = w*(h - x') - r */
    /* z' = 2*y*z */

    fe x2, y2, w, h, zyz;

    fe_sq(&x2, &p->x);           // x^2
    fe_sq(&y2, &p->y);           // y^2

    /* w = 3*x^2 */
    fe_add(&w, &x2, &x2);       // 2*x^2
    fe_add(&w, &w, &x2);        // 3*x^2

    /* h = x^2 + 2*y^2 ... no, h = 4*x*y^2 in this formula */
    /* Actually let me use a cleaner formula. */
    /* Standard projective doubling for a=0: */
    /* T0 = y^2 */
    /* T1 = 2*T0 = 2*y^2 */
    /* T2 = T1 + T1 = 4*y^2 */
    /* T3 = x + T2 */
    /* T4 = x - T2 */
    /* T5 = T3 * T4 = x^2 - 16*y^4 ... this is getting messy */

    /* Use the formula from cudaBTC: */
    /* - T0 = x * y */
    /* - T1 = x * z */
    /* - T2 = T1 + T1 = 2*x*z */
    /* - T3 = T2 + T2 + T2 = 6*x*z ... no, T3 = 3*T1 = 3*x*z */
    /* Let me use the standard formula from the secp256k1 reference: */

    /* For a=0: */
    fe t0, t1, t2, t3;

    /* t0 = x^2 */
    fe_sq(&t0, &p->x);
    /* t1 = z^2 */
    fe_sq(&t1, &p->z);
    /* t2 = 2*t1 = 2*z^2 */
    fe_add(&t2, &t1, &t1);
    /* t3 = t2 + t2 = 4*z^2 */
    fe_add(&t3, &t2, &t2);
    /* t1 = t1 + t3 = 5*z^2 */
    fe_add(&t1, &t1, &t3);
    /* t2 = x + y */
    fe_add(&t2, &p->x, &p->y);
    /* t2 = t2^2 = x^2 + 2xy + y^2 */
    fe_sq(&t2, &t2);
    /* t3 = t0 + t1 = x^2 + 5*z^2 */
    fe_add(&t3, &t0, &t1);
    /* t2 = t2 - t3 = 2xy - 4*z^2 ... hmm this doesn't seem right either */

    /* OK, let me just use the simple and well-known formula: */
    /* S = 4*x*y^2 */
    /* M = 3*x^2 */
    /* X3 = M^2 - 2*S */
    /* Y3 = M*(S - X3) - 8*y^4 */
    /* Z3 = 2*y*z */

    fe_sq(&y2, &p->y);                        // y^2
    fe_mul(&h, &p->x, &y2);                    // x*y^2
    fe_add(&h, &h, &h);                        // 2*x*y^2
    fe_add(&h, &h, &h);                        // 4*x*y^2 (= S)

    /* M = 3*x^2 */
    fe_add(&w, &x2, &x2);                      // 2*x^2
    fe_add(&w, &w, &x2);                       // 3*x^2 (= M)

    /* X3 = M^2 - 2*S */
    fe_sq(&r->x, &w);                          // M^2
    fe_add(&t0, &h, &h);                       // 2*S
    fe_sub(&r->x, &r->x, &t0);                // M^2 - 2*S

    /* Y3 = M*(S - X3) - 8*y^4 */
    fe_sub(&t0, &h, &r->x);                   // S - X3
    fe_mul(&r->y, &w, &t0);                   // M*(S-X3)
    fe_sq(&t1, &y2);                           // y^4
    fe_add(&t1, &t1, &t1);                    // 2*y^4
    fe_add(&t1, &t1, &t1);                    // 4*y^4
    fe_add(&t1, &t1, &t1);                    // 8*y^4
    fe_sub(&r->y, &r->y, &t1);               // M*(S-X3) - 8*y^4

    /* Z3 = 2*y*z */
    fe_mul(&zyz, &p->y, &p->z);
    fe_add(&r->z, &zyz, &zyz);
}

/* Add: r = p + q (projective addition, p != q) */
__device__ __forceinline__ void pt_add(point_t *r, const point_t *p, const point_t *q) {
    /* Using the addition formula for projective coordinates */
    fe t0, t1, t2, t3, t4, t5;

    /* t0 = z1^2 */
    fe_sq(&t0, &p->z);
    /* t1 = z2^2 */
    fe_sq(&t1, &q->z);
    /* t2 = t1 * y1 = z2^2 * y1 */
    fe_mul(&t2, &t1, &p->y);
    /* t3 = t0 * x1 = z1^2 * x1 */
    fe_mul(&t3, &t0, &p->x);
    /* t4 = t1 * y2 = z2^2 * y2 */
    fe_mul(&t4, &t1, &q->y);
    /* t5 = t0 * x2 = z1^2 * x2 */
    fe_mul(&t5, &t0, &q->x);

    if (fe_is_zero(&t3) && fe_is_zero(&t5)) {
        /* Both points at infinity */
        fe_zero(&r->x); fe_zero(&r->y); fe_set1(&r->z);
        return;
    }

    /* t0 = t5 - t3 = z1^2*x2 - z2^2*x1 */
    fe_sub(&t0, &t5, &t3);
    /* t1 = t4 - t2 = z2^2*y2 - z1^2*y1 */
    fe_sub(&t1, &t4, &t2);

    /* Check if points are the same (need doubling instead) */
    /* If t0 == 0 and t1 == 0, points are the same */
    /* For simplicity, we'll handle this in the caller */

    /* t3 = t0^2 */
    fe_sq(&t3, &t0);
    /* t4 = t0 * t3 = t0^3 */
    fe_mul(&t4, &t0, &t3);
    /* t5 = t5 * t3 = z1^2*x2 * t0^2 */
    fe_mul(&t5, &t5, &t3);
    /* r->z = z1 * z2 * t0 */
    fe_mul(&r->z, &p->z, &q->z);
    fe_mul(&r->z, &r->z, &t0);

    /* r->x = t1^2 - t4 - 2*t5 */
    fe_sq(&r->x, &t1);
    fe_sub(&r->x, &r->x, &t4);
    fe_sub(&r->x, &r->x, &t5);
    fe_sub(&r->x, &r->x, &t5);

    /* r->y = t1*(t5 - r->x) - t2*t4 */
    fe_sub(&t0, &t5, &r->x);
    fe_mul(&r->y, &t1, &t0);
    fe_mul(&t3, &t2, &t4);
    fe_sub(&r->y, &r->y, &t3);
}

/* ============================================================
 * Modular inverse: a^(p-2) mod p (Fermat's little theorem)
 * ============================================================ */

__device__ __forceinline__ void fe_inv(fe *r, const fe *a) {
    /* p-2 in binary:
     * p = 2^256 - 2^32 - 977
     * p-2 = 2^256 - 2^32 - 979
     *
     * In hex: FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFE FFFFFC2D
     *
     * Bits 255-32: all 1 (224 bits)
     * Bits 31-0: 0xFFFFFC2D = 1111111111111111111111000000101101
     *
     * Zero bits in p-2: bits 9,8,7,6,4,1
     * Total: 255 bits (254 down to 0), 249 ones, 6 zeros
     */

    fe b;
    fe_copy(&b, a);
    fe sq;

    /* Bits 254 down to 10: all 1 (245 bits) */
    for (int i = 0; i < 245; i++) {
        fe_sq(&sq, &b);
        fe_mul(&b, &sq, &b);
    }

    /* Bit 9 = 0 */ fe_sq(&sq, &b);
    /* Bit 8 = 0 */ fe_sq(&sq, &b);
    /* Bit 7 = 0 */ fe_sq(&sq, &b);
    /* Bit 6 = 0 */ fe_sq(&sq, &b);
    /* Bit 5 = 1 */ fe_sq(&sq, &b); fe_mul(&b, &sq, &b);
    /* Bit 4 = 0 */ fe_sq(&sq, &b);
    /* Bit 3 = 1 */ fe_mul(&b, &sq, &b);
    /* Bit 2 = 1 */ fe_sq(&sq, &b); fe_mul(&b, &sq, &b);
    /* Bit 1 = 0 */ fe_sq(&sq, &b);
    /* Bit 0 = 1 */ fe_mul(&b, &sq, &b);

    fe_copy(r, &b);
}

/* ============================================================
 * Projective to affine conversion
 * ============================================================ */

__device__ __forceinline__ void pt_to_affine(fe *ox, fe *oy, const point_t *p) {
    fe zinv, zsq;
    fe_sq(&zsq, &p->z);
    fe_inv(&zinv, &zsq);
    fe_mul(ox, &p->x, &zinv);
    fe_mul(&zinv, &zinv, &p->z);
    fe_mul(oy, &p->y, &zinv);
}

/* ============================================================
 * Precomputation: compute odd multiples of G on device
 * ============================================================ */

struct dev_prec {
    /* table[i] = (i+1)*G affine → 1G..16G for 4-bit window scalar mult */
    point_t table[16];
    int done;
};

/* Generator G of secp256k1 (affine, raw form, little-endian) */
__device__ __constant__ uint32_t CONST_Gx[FE_LIMBS] = {
    0x16F81798, 0x59F2815B, 0x2DCE28D9, 0x029BFCDB,
    0xCE870B07, 0x55A06295, 0xF9DCBBAC, 0x79BE667E
};
__device__ __constant__ uint32_t CONST_Gy[FE_LIMBS] = {
    0xFB10D4B8, 0x9C47D08F, 0xA6855419, 0xFD17B448,
    0x0E1108A8, 0x5DA4FBFC, 0x26A3C465, 0x483ADA77
};

__global__ void precompute_kernel(struct dev_prec *prec) {
    if (threadIdx.x != 0 || blockIdx.x != 0) return;

    point_t g;
    for (int i = 0; i < FE_LIMBS; i++) {
        g.x.v[i] = CONST_Gx[i];
        g.y.v[i] = CONST_Gy[i];
    }
    fe_set1(&g.z);

    fe_copy(&prec->table[0].x, &g.x);
    fe_copy(&prec->table[0].y, &g.y);
    fe_set1(&prec->table[0].z);

    for (int k = 1; k < 16; k++) {
        point_t a, b, r;
        fe_copy(&a.x, &prec->table[k - 1].x);
        fe_copy(&a.y, &prec->table[k - 1].y);
        fe_set1(&a.z);
        fe_copy(&b.x, &g.x);
        fe_copy(&b.y, &g.y);
        fe_set1(&b.z);
        pt_add(&r, &a, &b);
        fe xr, yr;
        pt_to_affine(&xr, &yr, &r);
        fe_copy(&prec->table[k].x, &xr);
        fe_copy(&prec->table[k].y, &yr);
        fe_set1(&prec->table[k].z);
    }
    prec->done = 1;
}

__device__ __forceinline__ int sk_bit(const uint32_t sk[FE_LIMBS], int b) {
    return (int)((sk[b >> 5] >> (b & 31)) & 1u);
}

/* P = sk * G, 4-bit window, table[0..15] = 1G..16G */
__device__ __forceinline__ void scalar_mul_g(
    point_t *acc,
    const uint32_t sk[FE_LIMBS],
    const point_t *table)
{
    int msb = -1;
    for (int i = FE_LIMBS - 1; i >= 0; i--) {
        if (sk[i] != 0) {
            for (int b = 31; b >= 0; b--) {
                if ((sk[i] >> b) & 1u) { msb = i * 32 + b; goto found_msb_sm; }
            }
        }
    }
found_msb_sm:
    if (msb < 0) {
        fe_zero(&acc->x); fe_zero(&acc->y); fe_set1(&acc->z);
        return;
    }
    int first_len = (msb % 4) + 1;
    int digit = 0;
    for (int i = 0; i < first_len; i++)
        digit = (digit << 1) | sk_bit(sk, msb - i);
    if (digit <= 0) {
        fe_zero(&acc->x); fe_zero(&acc->y); fe_set1(&acc->z);
    } else {
        fe_copy(&acc->x, &table[digit - 1].x);
        fe_copy(&acc->y, &table[digit - 1].y);
        fe_set1(&acc->z);
    }
    int bit = msb - first_len;
    while (bit >= 0) {
        pt_dbl(acc, acc); pt_dbl(acc, acc); pt_dbl(acc, acc); pt_dbl(acc, acc);
        digit = 0;
        for (int i = 0; i < 4; i++)
            digit = (digit << 1) | sk_bit(sk, bit - i);
        if (digit > 0) {
            point_t tp;
            fe_copy(&tp.x, &table[digit - 1].x);
            fe_copy(&tp.y, &table[digit - 1].y);
            fe_set1(&tp.z);
            pt_add(acc, acc, &tp);
        }
        bit -= 4;
    }
}

/* ============================================================
 * SHA256 implementation (device) — variable length
 * ============================================================ */
__device__ __constant__ uint32_t sha256_k[64] = {
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5,
    0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
    0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc,
    0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
    0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
    0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3,
    0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5,
    0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
    0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
};

__device__ __forceinline__ uint32_t sha_rotr(uint32_t x, int n) {
    return (x >> n) | (x << (32 - n));
}

/* SHA256 of N bytes (N < 55, fits in 1 block). Output: 32 bytes big-endian. */
__device__ void sha256_n(const uint8_t *input, int n, uint8_t *output) {
    uint32_t h[8] = {
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19
    };

    uint32_t w[64] = {0};
    for (int i = 0; i < n; i++) {
        int wi = i / 4;
        int shift = (3 - (i % 4)) * 8;
        w[wi] |= (uint32_t)input[i] << shift;
    }
    // Padding: 0x80 byte, then zeros, then 64-bit length in bits (BE)
    int last_byte = n % 4;
    int pad_word = n / 4;
    uint32_t pad = 0x80000000U >> (last_byte * 8);
    uint64_t bitlen = n * 8;
    if (last_byte == 3) { pad_word++; pad = 0x80; }
    w[pad_word] |= pad;
    // Length in last word
    w[15] |= (uint32_t)bitlen;

    // Schedule
    for (int i = 16; i < 64; i++) {
        uint32_t s0 = sha_rotr(w[i-15], 7) ^ sha_rotr(w[i-15], 18) ^ (w[i-15] >> 3);
        uint32_t s1 = sha_rotr(w[i-2], 17) ^ sha_rotr(w[i-2], 19) ^ (w[i-2] >> 10);
        w[i] = w[i-16] + s0 + w[i-7] + s1;
    }

    // 64 rounds
    uint32_t a=h[0],b=h[1],c=h[2],d=h[3],e=h[4],f=h[5],g=h[6],hh=h[7];
    for (int i = 0; i < 64; i++) {
        uint32_t S1 = sha_rotr(e,6)^sha_rotr(e,11)^sha_rotr(e,25);
        uint32_t ch_ = (e&f)^(~e&g);
        uint32_t t1 = hh+S1+ch_+sha256_k[i]+w[i];
        uint32_t S0 = sha_rotr(a,2)^sha_rotr(a,13)^sha_rotr(a,22);
        uint32_t maj_ = (a&b)^(a&c)^(b&c);
        uint32_t t2 = S0+maj_;
        hh=g; g=f; f=e; e=d+t1; d=c; c=b; b=a; a=t1+t2;
    }
    h[0]+=a; h[1]+=b; h[2]+=c; h[3]+=d; h[4]+=e; h[5]+=f; h[6]+=g; h[7]+=hh;

    for (int i = 0; i < 8; i++) {
        output[i*4+0] = (h[i]>>24)&0xFF; output[i*4+1] = (h[i]>>16)&0xFF;
        output[i*4+2] = (h[i]>>8)&0xFF;  output[i*4+3]  = h[i]&0xFF;
    }
}

/* ============================================================
 * RIPEMD160 implementation (device) — variable length
 * ============================================================ */
__device__ __forceinline__ uint32_t f_rip(int j, uint32_t x, uint32_t y, uint32_t z) {
    int r = j / 16;
    if (r == 0) return x^y^z;
    if (r == 1) return (x&y)|(~x&z);
    if (r == 2) return (x|~y)^z;
    return x^(y|~z);
}

/* RIPEMD160 of N bytes (N < 55, fits in 1 block). Output: 20 bytes big-endian. */
__device__ void ripemd160_n(const uint8_t *input, int n, uint8_t *output) {
    uint32_t h[5] = {0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476, 0xc3d2e1f0};

    // Build padded block (16 x 32-bit words, big-endian)
    uint32_t X[16] = {0};
    for (int i = 0; i < n; i++) {
        int wi = i / 4;
        int shift = (3 - (i % 4)) * 8;
        X[wi] |= (uint32_t)input[i] << shift;
    }
    int last_byte = n % 4;
    int pad_word = n / 4;
    X[pad_word] |= 0x80000000U >> (last_byte * 8);
    if (last_byte == 3) X[0] = (uint32_t)(n * 8); // length in first word of 2nd block -> wraps
    else X[15] |= (uint32_t)(n * 8);

    // Round constants
    static const uint32_t K[5]  = {0x00000000,0x5a827999,0x6ed9eba1,0x8f1bbcdc,0xa953fd4e};
    static const uint32_t K1[5] = {0x50a28be6,0x5c4dd124,0x6d703ef3,0x7a6d76e9,0x00000000};
    static const int  R[5]  = {11,14,15,12,5};
    static const int  R1[5] = {8,9,9,11,13};

    // Permutations
    static const int P[80]  = {0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,
        5,4,3,2,1,0,7,6,5,4,3,2,1,0,7,6,
        1,6,11,0,5,10,15,4,9,14,3,8,13,2,7,12,
        5,8,11,14,1,4,7,10,13,0,3,6,9,12,15,2,
        0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15};
    static const int P1[80] = {5,14,7,0,9,2,11,4,13,6,15,8,1,10,3,12,
        6,11,3,7,0,13,5,10,14,15,8,12,4,9,1,2,
        15,5,1,3,7,14,6,9,11,8,12,2,10,0,4,13,
        8,6,4,1,3,11,15,0,5,12,2,13,9,7,10,14,
        12,15,10,4,1,5,8,7,6,2,13,14,0,3,9,11};

    // Compression
    uint32_t a=h[0],b=h[1],c=h[2],d=h[3],e=h[4];
    uint32_t a1=a,b1=b,c1=c,d1=d,e1=e;

    for (int j = 0; j < 80; j++) {
        int r = j / 16;
        uint32_t t = sha_rotr(a + f_rip(j,b,c,d) + X[P[j]] + K[r], R[r]) + e;
        a=e; e=sha_rotr(d,10); d=c; c=sha_rotr(b,10); b=t;

        t = sha_rotr(a1 + f_rip(79-j,b1,c1,d1) + X[P1[j]] + K1[r], R1[r]) + e1;
        a1=e1; e1=sha_rotr(d1,10); d1=c1; c1=sha_rotr(b1,10); b1=t;
    }

    uint32_t t1 = h[2]+d+e1;
    uint32_t t2 = h[3]+a+b1;
    uint32_t t3 = h[4]+b+c1;
    uint32_t t4 = h[0]+e+a1;
    h[0] = h[1]+t1; h[1] = t1; h[2] = t2; h[3] = t3; h[4] = t4;

    for (int i = 0; i < 5; i++) {
        output[i*4+0] = (h[i]>>24)&0xFF; output[i*4+1] = (h[i]>>16)&0xFF;
        output[i*4+2] = (h[i]>>8)&0xFF;  output[i*4+3]  = h[i]&0xFF;
    }
}

/* ============================================================
 * FlatIndex binary search on GPU
 * ============================================================ */

/* FlatIndex entry on GPU — matches Rust ScriptEntry layout (12 bytes, packed) */
typedef struct {
    uint32_t script_offset;
    uint16_t script_len;
    uint32_t utxo_offset;
    uint32_t utxo_count;
} gpu_script_entry_t;

/* Compare a generated script (gen_script, gen_len) with the stored script at index i.
 * Returns: -1 if stored < gen, 0 if equal, +1 if stored > gen
 * Optimized: 4-byte (uint32_t) comparison for fewer memory accesses.
 * Uses __ldg() for read-only cache. Unaligned loads are efficient on sm_86+. */
__device__ int cmp_script_with_index(
    const gpu_script_entry_t *script_entries,
    const uint8_t *all_data,
    int i,
    const uint8_t *gen_script,
    int gen_len)
{
    gpu_script_entry_t entry = script_entries[i];
    const uint8_t *stored = all_data + entry.script_offset;
    int stored_len = entry.script_len;

    int minlen = (stored_len < gen_len) ? stored_len : gen_len;

    /* 4-byte comparison (uint32_t) — reduces memory accesses by 4x */
    int k = 0;
    const int full_words = minlen / 4;
    for (int w = 0; w < full_words; w++) {
        uint32_t sw = __ldg((const uint32_t *)(stored + k));
        uint32_t gw = *((const uint32_t *)(gen_script + k));
        if (sw != gw) {
            /* Byte-level comparison within the differing word (LE) */
            for (int b = 0; b < 4; b++) {
                uint8_t sb = (sw >> (8 * b)) & 0xFF;
                uint8_t gb = (gw >> (8 * b)) & 0xFF;
                if (sb != gb) return sb < gb ? -1 : 1;
            }
        }
        k += 4;
    }

    /* Remaining bytes (< 4) */
    for (int r = k; r < minlen; r++) {
        uint8_t sb = __ldg(stored + r);
        uint8_t gb = gen_script[r];
        if (sb != gb) return sb < gb ? -1 : 1;
    }

    /* One is a prefix of the other */
    if (stored_len < gen_len) return -1;
    if (stored_len > gen_len) return 1;
    return 0;
}

/* Sum UTXO values for a matched script entry.
 * UTXO entries are 44 bytes: txid(32) + vout(4) + value(8) */
__device__ uint64_t sum_utxo_values(
    const uint8_t *utxo_data,
    uint32_t utxo_offset,
    uint32_t utxo_count)
{
    uint64_t total = 0;
    for (uint32_t i = 0; i < utxo_count; i++) {
        uint32_t pos = utxo_offset + (i * 44);
        /* value is at offset 32 within each 44-byte entry, little-endian */
        uint64_t val = *(const uint64_t *)(utxo_data + pos + 32);
        total += val;
    }
    return total;
}

/* Binary search for a generated script in the FlatIndex.
 * Returns total UTXO value if found, 0 if not found. */
__device__ uint64_t flat_index_lookup(
    const gpu_script_entry_t *script_entries,
    const uint8_t *all_data,
    const uint8_t *utxo_data,
    uint32_t num_entries,
    const uint8_t *gen_script,
    int gen_len)
{
    if (num_entries == 0) return 0;

    uint32_t lo = 0;
    uint32_t hi = num_entries;

    while (lo < hi) {
        uint32_t mid = lo + ((hi - lo) >> 1);
        int cmp = cmp_script_with_index(script_entries, all_data, (int)mid,
                                        gen_script, gen_len);
        if (cmp < 0) lo = mid + 1;
        else if (cmp > 0) hi = mid;
        else {
            /* Found — sum UTXO values */
            gpu_script_entry_t entry = script_entries[mid];
            return sum_utxo_values(utxo_data, entry.utxo_offset, entry.utxo_count);
        }
    }

    return 0;
}

/* ============================================================
 * Main kernel: derive pubkey + hash160 + SHA256(pubkey)
 *
 * Output per key (85 bytes):
 *   [0..32]   = compressed pubkey (33 bytes)
 *   [33..52]  = hash160 = RIPEMD160(SHA256(pubkey)) (20 bytes)
 *   [53..84]  = SHA256(pubkey) (32 bytes)
 *
 * CPU consumer builds scripts by simple byte concatenation:
 *   Legacy:  76a914 + hash160 + 88ac  (25 bytes)
 *   Segwit:  0014 + sha256[0..19]     (22 bytes)
 *   Wrapped: 76a914 + RIPEMD160(0014+sha256[0..19]) + 88ac  (CPU does 1 RIPEMD160)
 *   Taproot: x-only pubkey from pubkey[1..33]
 *
 * GPU saves: SHA256(pubkey) + RIPEMD160(SHA256(pubkey)) per key
 * CPU saves: zero hashing for legacy, just byte concat
 * ============================================================ */
#define OUT_PUBKEY  0   // 33 bytes
#define OUT_HASH160 33  // 20 bytes
#define OUT_SHA256  53  // 32 bytes
#define OUT_STRIDE  85  // total per key

__global__ void derive_pubkey_kernel(
    const uint8_t *privkeys,
    uint8_t *output,  /* per key: OUT_STRIDE bytes */
    const point_t *base_table,
    int count)
{
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx >= count) return;

    /* Load private key (LE) */
    const uint32_t *sk32 = (const uint32_t *)(privkeys + idx * 32);
    uint32_t sk[FE_LIMBS];
    for (int i = 0; i < FE_LIMBS; i++) sk[i] = sk32[i];

    uint8_t *out = output + idx * OUT_STRIDE;
    {
        int sk0 = 1;
        for (int i = 0; i < FE_LIMBS; i++) if (sk[i]) { sk0 = 0; break; }
        if (sk0) { for (int i = 0; i < OUT_STRIDE; i++) out[i] = 0; return; }
    }
    point_t acc;
    scalar_mul_g(&acc, sk, base_table);

    /* Convert to affine */
    fe ox, oy;
    pt_to_affine(&ox, &oy, &acc);

    /* Compressed pubkey: 0x02/0x03 + x (BE) */
    uint8_t pubkey[33];
    pubkey[0] = (oy.v[0] & 1) ? 0x03 : 0x02;
    for (int i = 0; i < 32; i++) {
        int byte_idx = 31 - i;
        pubkey[1+i] = (ox.v[byte_idx/4] >> ((byte_idx%4)*8)) & 0xFF;
    }
    // Write pubkey to output
    for (int i = 0; i < 33; i++) out[OUT_PUBKEY+i] = pubkey[i];

    /* Compute SHA256(pubkey) — needed for both hash160 and segwit */
    uint8_t sha[32];
    sha256_n(pubkey, 33, sha);

    /* Write SHA256(pubkey) to output */
    for (int i = 0; i < 32; i++) out[OUT_SHA256+i] = sha[i];

    /* Compute hash160 = RIPEMD160(SHA256(pubkey)) */
    ripemd160_n(sha, 32, out + OUT_HASH160);
}

/* ============================================================
 * Combined kernel: derive pubkey + FlatIndex lookup
 *
 * Each thread processes one key:
 *   1. Derive compressed pubkey from privkey
 *   2. Compute SHA256(pubkey) and hash160
 *   3. Build 4 script types and binary search in FlatIndex
 *   4. Output: 8 bytes per key (total value across all address types)
 *
 * Address type bitmask:
 *   bit 0 = legacy (P2PKH)
 *   bit 1 = segwit  (P2WPKH)
 *   bit 2 = wrapped (P2SH-P2WPKH)
 *   bit 3 = taproot (P2TR)
 * ============================================================ */
__global__ void derive_lookup_kernel(
    const uint8_t *privkeys,
    uint64_t *total_values,  /* 8 bytes per key: sum of all matched UTXO values */
    const point_t *base_table,
    const gpu_script_entry_t *script_entries,
    const uint8_t *all_data,
    const uint8_t *utxo_data,
    uint32_t num_script_entries,
    int count,
    uint32_t addr_types)
{
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx >= count) return;

    /* Load private key (LE) */
    const uint32_t *sk32 = (const uint32_t *)(privkeys + idx * 32);
    uint32_t sk[FE_LIMBS];
    for (int i = 0; i < FE_LIMBS; i++) sk[i] = sk32[i];

    {
        int sk0 = 1;
        for (int i = 0; i < FE_LIMBS; i++) if (sk[i]) { sk0 = 0; break; }
        if (sk0) { total_values[idx] = 0; return; }
    }
    point_t acc;
    scalar_mul_g(&acc, sk, base_table);

    /* Convert to affine */
    fe ox, oy;
    pt_to_affine(&ox, &oy, &acc);

    /* Compressed pubkey: 0x02/0x03 + x (BE) */
    uint8_t pubkey[33];
    pubkey[0] = (oy.v[0] & 1) ? 0x03 : 0x02;
    for (int i = 0; i < 32; i++) {
        int byte_idx = 31 - i;
        pubkey[1+i] = (ox.v[byte_idx/4] >> ((byte_idx%4)*8)) & 0xFF;
    }

    /* Compute SHA256(pubkey) — needed for hash160, segwit, wrapped */
    uint8_t sha[32];
    sha256_n(pubkey, 33, sha);

    /* Compute hash160 = RIPEMD160(SHA256(pubkey)) — needed for legacy */
    uint8_t hash160[20];
    ripemd160_n(sha, 32, hash160);

    uint64_t total = 0;

    /* Legacy (P2PKH): 76a914 + hash160(20) + 88ac = 25 bytes */
    if (addr_types & 0x01) {
        uint8_t script[25];
        script[0] = 0x76; script[1] = 0xa9; script[2] = 0x14;
        for (int i = 0; i < 20; i++) script[3+i] = hash160[i];
        script[23] = 0x88; script[24] = 0xac;
        total += flat_index_lookup(script_entries, all_data, utxo_data,
                                   num_script_entries, script, 25);
    }

    /* Segwit (P2WPKH): 0014 + sha256[0..19] = 22 bytes */
    if (addr_types & 0x02) {
        uint8_t script[22];
        script[0] = 0x00; script[1] = 0x14;
        for (int i = 0; i < 20; i++) script[2+i] = sha[i];
        total += flat_index_lookup(script_entries, all_data, utxo_data,
                                   num_script_entries, script, 22);
    }

    /* Wrapped (P2SH-P2WPKH): 76a914 + RIPEMD160(0014+sha[0..19]) + 88ac = 25 bytes */
    if (addr_types & 0x04) {
        uint8_t witness[22];
        witness[0] = 0x00; witness[1] = 0x14;
        for (int i = 0; i < 20; i++) witness[2+i] = sha[i];
        uint8_t w_hash160[20];
        ripemd160_n(witness, 22, w_hash160);
        uint8_t script[25];
        script[0] = 0x76; script[1] = 0xa9; script[2] = 0x14;
        for (int i = 0; i < 20; i++) script[3+i] = w_hash160[i];
        script[23] = 0x88; script[24] = 0xac;
        total += flat_index_lookup(script_entries, all_data, utxo_data,
                                   num_script_entries, script, 25);
    }

    /* Taproot (P2TR): 52ab + xonly_pk(32) = 34 bytes */
    if (addr_types & 0x08) {
        uint8_t script[34];
        script[0] = 0x52; script[1] = 0xab;
        for (int i = 0; i < 32; i++) script[2+i] = pubkey[1+i];
        total += flat_index_lookup(script_entries, all_data, utxo_data,
                                   num_script_entries, script, 34);
    }

    total_values[idx] = total;
}

/* ============================================================
 * Device state and management
 * ============================================================ */

#define MAX_DEVICES 8

typedef struct {
    int             id;
    cudaStream_t    stream;
    cudaStream_t    stream2;            /* second stream for double-buffering */
    uint8_t        *d_privkeys;
    uint8_t        *d_pubkeys;
    point_t         base_table[16];
    size_t          alloc_size;
    int             initialized;
    char            name[128];
    size_t          mem_total;

    /* FlatIndex data (loaded once, reused across batches) */
    gpu_script_entry_t *d_script_entries;
    uint8_t          *d_all_data;       /* script bytes */
    uint8_t          *d_utxo_data;      /* full UTXO entries (44 bytes each) */
    uint32_t          num_script_entries;
    size_t            all_data_size;
    size_t            utxo_data_size;
    int               index_loaded;

    /* Double-buffering: persistent output buffers (no malloc/free per batch) */
    uint64_t         *d_out;            /* slot 0 output */
    uint64_t         *d_out2;           /* slot 1 output */
    size_t            out_alloc_size;   /* max output size allocated */
    int               current_slot;     /* 0 or 1 for ping-pong */

    /* Pinned host memory for truly async transfers (cudaMemcpyAsync with pinned = non-blocking) */
    uint8_t          *h_privkeys_pinned;  /* pinned input buffer */
    uint64_t         *h_out_pinned;       /* pinned output buffer */
    size_t            pinned_alloc_size;  /* max pinned input size */
    size_t            pinned_out_size;    /* max pinned output size */
    uint64_t         *h_total_values;     /* host output pointer for async sync */
    int               pinned_offset;      /* offset in total_values for last async call */
    int               pinned_count;       /* count for last async call */
} gpu_device_t;

static gpu_device_t gpu_devs[MAX_DEVICES];
static int num_gpu_devs = 0;

static cudaError_t init_one_device(int dev_id) {
    cudaDeviceProp prop;
    cudaGetDeviceProperties(&prop, dev_id);

    if (prop.totalGlobalMem < 2ULL * 1024 * 1024 * 1024)
        return cudaErrorInsufficientDriver;

    cudaSetDevice(dev_id);

    /* Precompute base table on device */
    struct dev_prec *d_prec = NULL;
    cudaMalloc(&d_prec, sizeof(struct dev_prec));

    precompute_kernel<<<1, 1>>>(d_prec);
    cudaDeviceSynchronize();

    struct dev_prec h_prec;
    cudaMemcpy(&h_prec, d_prec, sizeof(struct dev_prec), cudaMemcpyDeviceToHost);
    cudaFree(d_prec);

    /* Store table */
    memcpy(gpu_devs[num_gpu_devs].base_table, h_prec.table, sizeof(h_prec.table));
    gpu_devs[num_gpu_devs].id = dev_id;
    gpu_devs[num_gpu_devs].alloc_size = 0;
    gpu_devs[num_gpu_devs].initialized = 1;
    gpu_devs[num_gpu_devs].d_out = NULL;
    gpu_devs[num_gpu_devs].d_out2 = NULL;
    gpu_devs[num_gpu_devs].out_alloc_size = 0;
    gpu_devs[num_gpu_devs].current_slot = 0;
    cudaStreamCreate(&gpu_devs[num_gpu_devs].stream2);
    gpu_devs[num_gpu_devs].d_script_entries = NULL;
    gpu_devs[num_gpu_devs].d_all_data = NULL;
    gpu_devs[num_gpu_devs].d_utxo_data = NULL;
    gpu_devs[num_gpu_devs].num_script_entries = 0;
    gpu_devs[num_gpu_devs].all_data_size = 0;
    gpu_devs[num_gpu_devs].utxo_data_size = 0;
    gpu_devs[num_gpu_devs].index_loaded = 0;
    // Pinned memory fields
    gpu_devs[num_gpu_devs].h_privkeys_pinned = NULL;
    gpu_devs[num_gpu_devs].h_out_pinned = NULL;
    gpu_devs[num_gpu_devs].pinned_alloc_size = 0;
    gpu_devs[num_gpu_devs].pinned_out_size = 0;
    gpu_devs[num_gpu_devs].h_total_values = NULL;
    gpu_devs[num_gpu_devs].pinned_offset = 0;
    gpu_devs[num_gpu_devs].pinned_count = 0;
    gpu_devs[num_gpu_devs].mem_total = prop.totalGlobalMem;
    strncpy(gpu_devs[num_gpu_devs].name, prop.name, 127);
    gpu_devs[num_gpu_devs].name[127] = 0;

    cudaStreamCreate(&gpu_devs[num_gpu_devs].stream);
    num_gpu_devs++;

    cudaSetDevice(0);
    return cudaSuccess;
}

/* ============================================================
 * Public API (C linkage, exported from DLL)
 * ============================================================ */

#ifdef _WIN32
#define GPU_API extern "C" __declspec(dllexport)
#else
#define GPU_API extern "C"
#endif

GPU_API int secp_gpu_init(void) {
    num_gpu_devs = 0;

    int count = 0;
    cudaGetDeviceCount(&count);

    for (int i = 0; i < count && num_gpu_devs < MAX_DEVICES; i++) {
        cudaError_t err = init_one_device(i);
        if (err == cudaSuccess) {
            printf("[GPU] Device %d: %s (%.0f GB VRAM)\n",
                i, gpu_devs[num_gpu_devs-1].name,
                gpu_devs[num_gpu_devs-1].mem_total / (1024.0*1024*1024));
        } else {
            printf("[GPU] Skipping device %d (err=%d)\n", i, err);
        }
    }

    if (num_gpu_devs == 0) {
        fprintf(stderr, "[GPU] No suitable NVIDIA GPU found\n");
        return -1;
    }

    printf("[GPU] Initialized %d GPU(s)\n", num_gpu_devs);
    return num_gpu_devs;
}

GPU_API int secp_gpu_derive_multi(
    const uint8_t *privkeys,
    uint8_t *pubkeys,
    int count,
    const int *device_ids,
    int num_devs)
{
    if (num_devs <= 0) return -1;

    int keys_per_dev = count / num_devs;
    int remainder = count % num_devs;
    int offset = 0;

    for (int d = 0; d < num_devs; d++) {
        int n = keys_per_dev + (d < remainder ? 1 : 0);
        if (n <= 0) continue;

        int did = device_ids ? device_ids[d % num_gpu_devs] : d;
        if (did >= num_gpu_devs) continue;
        gpu_device_t *dev = &gpu_devs[did];

        cudaSetDevice(dev->id);

        size_t out_stride = 85; // pubkey(33) + hash160(20) + sha256(32)
        size_t needed = n * 32;
        if (needed > dev->alloc_size) {
            if (dev->d_privkeys) cudaFree(dev->d_privkeys);
            if (dev->d_pubkeys) cudaFree(dev->d_pubkeys);
            cudaMalloc(&dev->d_privkeys, needed);
            cudaMalloc(&dev->d_pubkeys, n * out_stride);
            dev->alloc_size = needed;
        }

        cudaMemcpyAsync(dev->d_privkeys, privkeys + offset * 32, n * 32,
                        cudaMemcpyHostToDevice, dev->stream);

        int blocks = (n + BLOCK_SIZE - 1) / BLOCK_SIZE;
        derive_pubkey_kernel<<<blocks, BLOCK_SIZE, 0, dev->stream>>>(
            dev->d_privkeys, dev->d_pubkeys,
            dev->base_table, n);

        cudaMemcpyAsync(pubkeys + offset * out_stride, dev->d_pubkeys, n * out_stride,
                        cudaMemcpyDeviceToHost, dev->stream);

        cudaStreamSynchronize(dev->stream);
        offset += n;
    }

    cudaSetDevice(0);
    return 0;
}

GPU_API int secp_gpu_derive(const uint8_t *privkeys, uint8_t *pubkeys, int count) {
    if (num_gpu_devs == 0) {
        fprintf(stderr, "[GPU] Not initialized. Call secp_gpu_init() first.\n");
        return -1;
    }
    return secp_gpu_derive_multi(privkeys, pubkeys, count, NULL, num_gpu_devs);
}

GPU_API int secp_gpu_device_count(void) {
    int count = 0;
    cudaGetDeviceCount(&count);
    return count;
}

GPU_API void secp_gpu_device_name(int idx, char *buf, int bufsize) {
    cudaDeviceProp prop;
    if (cudaGetDeviceProperties(&prop, idx) == cudaSuccess) {
        snprintf(buf, bufsize, "%s", prop.name);
    } else {
        snprintf(buf, bufsize, "Unknown device %d", idx);
    }
}

/* Load FlatIndex data onto GPU devices.
 * script_entries: array of gpu_script_entry_t (12 bytes each, packed)
 * all_data: raw script bytes
 * utxo_data: raw UTXO entry bytes (44 bytes each: txid[32] + vout[4] + value[8])
 * num_entries: number of script entries
 * Returns: 0 on success, -1 on error */
GPU_API int secp_gpu_load_index(
    const void *script_entries,
    const uint8_t *all_data,
    const uint8_t *utxo_data,
    uint32_t num_entries,
    size_t all_data_size,
    size_t utxo_data_size)
{
    if (num_gpu_devs == 0) {
        fprintf(stderr, "[GPU] Not initialized.\n");
        return -1;
    }

    for (int d = 0; d < num_gpu_devs; d++) {
        cudaSetDevice(gpu_devs[d].id);

        /* Free previous index if any */
        if (gpu_devs[d].d_script_entries) cudaFree(gpu_devs[d].d_script_entries);
        if (gpu_devs[d].d_all_data) cudaFree(gpu_devs[d].d_all_data);
        if (gpu_devs[d].d_utxo_data) cudaFree(gpu_devs[d].d_utxo_data);

        size_t entries_size = num_entries * sizeof(gpu_script_entry_t);

        cudaMalloc(&gpu_devs[d].d_script_entries, entries_size);
        cudaMalloc(&gpu_devs[d].d_all_data, all_data_size);
        cudaMalloc(&gpu_devs[d].d_utxo_data, utxo_data_size);

        cudaMemcpy(gpu_devs[d].d_script_entries, script_entries, entries_size,
                   cudaMemcpyHostToDevice);
        cudaMemcpy(gpu_devs[d].d_all_data, all_data, all_data_size,
                   cudaMemcpyHostToDevice);
        cudaMemcpy(gpu_devs[d].d_utxo_data, utxo_data, utxo_data_size,
                   cudaMemcpyHostToDevice);

        gpu_devs[d].num_script_entries = num_entries;
        gpu_devs[d].all_data_size = all_data_size;
        gpu_devs[d].utxo_data_size = utxo_data_size;
        gpu_devs[d].index_loaded = 1;

        size_t total_mb = (entries_size + all_data_size + utxo_data_size) / (1024.0 * 1024.0);
        printf("[GPU] Device %d: index loaded (%.0f MB — entries=%.0fMB data=%.0fMB utxo=%.0fMB)\n",
               d, total_mb,
               entries_size / (1024.0*1024.0),
               all_data_size / (1024.0*1024.0),
               utxo_data_size / (1024.0*1024.0));
    }

    cudaSetDevice(0);
    return 0;
}

/* Unload FlatIndex data from GPU devices */
GPU_API void secp_gpu_unload_index(void) {
    for (int d = 0; d < num_gpu_devs; d++) {
        if (!gpu_devs[d].index_loaded) continue;
        cudaSetDevice(gpu_devs[d].id);
        if (gpu_devs[d].d_script_entries) { cudaFree(gpu_devs[d].d_script_entries); gpu_devs[d].d_script_entries = NULL; }
        if (gpu_devs[d].d_all_data) { cudaFree(gpu_devs[d].d_all_data); gpu_devs[d].d_all_data = NULL; }
        if (gpu_devs[d].d_utxo_data) { cudaFree(gpu_devs[d].d_utxo_data); gpu_devs[d].d_utxo_data = NULL; }
        gpu_devs[d].index_loaded = 0;
        printf("[GPU] Device %d: index unloaded\n", d);
    }
    cudaSetDevice(0);
}

/* Derive pubkey + FlatIndex lookup in one kernel launch.
 * privkeys: input private keys (32 bytes each, LE)
 * total_values: output total UTXO value per key (8 bytes each, LE)
 * count: number of keys
 * addr_types: bitmask (1=legacy, 2=segwit, 4=wrapped, 8=taproot)
 * Returns: 0 on success, -1 on error */
GPU_API int secp_gpu_derive_lookup(
    const uint8_t *privkeys,
    uint64_t *total_values,
    int count,
    uint32_t addr_types,
    const int *device_ids,
    int num_devs)
{
    if (num_gpu_devs == 0) {
        fprintf(stderr, "[GPU] Not initialized.\n");
        return -1;
    }
    if (num_devs <= 0) num_devs = num_gpu_devs;

    /* Check index is loaded */
    for (int d = 0; d < num_devs; d++) {
        int did = device_ids ? device_ids[d % num_gpu_devs] : d;
        if (did >= num_gpu_devs) continue;
        if (!gpu_devs[did].index_loaded) {
            fprintf(stderr, "[GPU] Device %d: index not loaded. Call secp_gpu_load_index() first.\n", did);
            return -1;
        }
    }

    int keys_per_dev = count / num_devs;
    int remainder = count % num_devs;
    int offset = 0;

    for (int d = 0; d < num_devs; d++) {
        int n = keys_per_dev + (d < remainder ? 1 : 0);
        if (n <= 0) continue;

        int did = device_ids ? device_ids[d % num_gpu_devs] : d;
        if (did >= num_gpu_devs) continue;
        gpu_device_t *dev = &gpu_devs[did];

        cudaSetDevice(dev->id);

        /* Allocate input buffer (grow if needed) */
        size_t needed = n * 32;
        if (needed > dev->alloc_size) {
            if (dev->d_privkeys) cudaFree(dev->d_privkeys);
            cudaMalloc(&dev->d_privkeys, needed);
            dev->alloc_size = needed;
        }

        /* Persistent output buffer (ping-pong double-buffering) */
        size_t out_size = n * sizeof(uint64_t);
        int slot = dev->current_slot;
        uint64_t *d_out = (slot == 0) ? dev->d_out : dev->d_out2;
        cudaStream_t stream = (slot == 0) ? dev->stream : dev->stream2;
        if (out_size > dev->out_alloc_size) {
            if (d_out) cudaFree(d_out);
            cudaMalloc(&d_out, out_size);
            if (slot == 0) dev->d_out = d_out; else dev->d_out2 = d_out;
            dev->out_alloc_size = out_size;
        }
        if (!d_out) {
            cudaMalloc(&d_out, out_size);
            if (slot == 0) dev->d_out = d_out; else dev->d_out2 = d_out;
            dev->out_alloc_size = out_size;
        }

        /* Transfer privkeys to device */
        cudaMemcpyAsync(dev->d_privkeys, privkeys + offset * 32, n * 32,
                        cudaMemcpyHostToDevice, stream);

        /* Launch kernel */
        int blocks = (n + BLOCK_SIZE - 1) / BLOCK_SIZE;
        derive_lookup_kernel<<<blocks, BLOCK_SIZE, 0, stream>>>(
            dev->d_privkeys, d_out,
            dev->base_table,
            dev->d_script_entries, dev->d_all_data, dev->d_utxo_data,
            dev->num_script_entries,
            n, addr_types);

        /* Transfer results back */
        cudaMemcpyAsync(total_values + offset, d_out, out_size,
                        cudaMemcpyDeviceToHost, stream);

        cudaStreamSynchronize(stream);
        dev->current_slot = 1 - slot; // toggle for next call
        offset += n;
    }

    cudaSetDevice(0);
    return 0;
}

/* Async version: launch kernel without waiting. Call secp_gpu_sync_all() before reading results.
 * Uses pinned host memory for truly async transfers (non-blocking cudaMemcpyAsync).
 * Pipeline: host→pinned→device (async) → kernel → device→pinned (async) → sync→host */
GPU_API int secp_gpu_derive_lookup_async(
    const uint8_t *privkeys,
    uint64_t *total_values,
    int count,
    uint32_t addr_types,
    const int *device_ids,
    int num_devs)
{
    if (num_gpu_devs == 0) return -1;
    if (num_devs <= 0) num_devs = num_gpu_devs;

    for (int d = 0; d < num_devs; d++) {
        int did = device_ids ? device_ids[d % num_gpu_devs] : d;
        if (did >= num_gpu_devs) continue;
        if (!gpu_devs[did].index_loaded) return -1;
    }

    int keys_per_dev = count / num_devs;
    int remainder = count % num_devs;
    int offset = 0;

    for (int d = 0; d < num_devs; d++) {
        int n = keys_per_dev + (d < remainder ? 1 : 0);
        if (n <= 0) continue;

        int did = device_ids ? device_ids[d % num_gpu_devs] : d;
        if (did >= num_gpu_devs) continue;
        gpu_device_t *dev = &gpu_devs[did];

        cudaSetDevice(dev->id);

        size_t needed = n * 32;
        size_t out_size = n * sizeof(uint64_t);

        /* Allocate device buffers (grow if needed) */
        if (needed > dev->alloc_size) {
            if (dev->d_privkeys) cudaFree(dev->d_privkeys);
            cudaMalloc(&dev->d_privkeys, needed);
            dev->alloc_size = needed;
        }

        int slot = dev->current_slot;
        uint64_t *d_out = (slot == 0) ? dev->d_out : dev->d_out2;
        cudaStream_t stream = (slot == 0) ? dev->stream : dev->stream2;
        if (out_size > dev->out_alloc_size) {
            if (d_out) cudaFree(d_out);
            cudaMalloc(&d_out, out_size);
            if (slot == 0) dev->d_out = d_out; else dev->d_out2 = d_out;
            dev->out_alloc_size = out_size;
        }
        if (!d_out) {
            cudaMalloc(&d_out, out_size);
            if (slot == 0) dev->d_out = d_out; else dev->d_out2 = d_out;
            dev->out_alloc_size = out_size;
        }

        /* Allocate pinned host memory (grow if needed) — enables truly async transfers */
        if (needed > dev->pinned_alloc_size) {
            if (dev->h_privkeys_pinned) cudaFreeHost(dev->h_privkeys_pinned);
            cudaMallocHost(&dev->h_privkeys_pinned, needed);
            dev->pinned_alloc_size = needed;
        }
        if (!dev->h_privkeys_pinned) {
            cudaMallocHost(&dev->h_privkeys_pinned, needed);
            dev->pinned_alloc_size = needed;
        }

        if (out_size > dev->pinned_out_size) {
            if (dev->h_out_pinned) cudaFreeHost(dev->h_out_pinned);
            cudaMallocHost(&dev->h_out_pinned, out_size);
            dev->pinned_out_size = out_size;
        }
        if (!dev->h_out_pinned) {
            cudaMallocHost(&dev->h_out_pinned, out_size);
            dev->pinned_out_size = out_size;
        }

        /* Copy host → pinned (fast, both host memory) */
        memcpy(dev->h_privkeys_pinned, privkeys + offset * 32, needed);

        /* Pinned → device (TRULY async — non-blocking!) */
        cudaMemcpyAsync(dev->d_privkeys, dev->h_privkeys_pinned, needed,
                        cudaMemcpyHostToDevice, stream);

        /* Launch kernel */
        int blocks = (n + BLOCK_SIZE - 1) / BLOCK_SIZE;
        derive_lookup_kernel<<<blocks, BLOCK_SIZE, 0, stream>>>(
            dev->d_privkeys, d_out,
            dev->base_table,
            dev->d_script_entries, dev->d_all_data, dev->d_utxo_data,
            dev->num_script_entries,
            n, addr_types);

        /* Device → pinned output (TRULY async — non-blocking!) */
        cudaMemcpyAsync(dev->h_out_pinned, d_out, out_size,
                        cudaMemcpyDeviceToHost, stream);

        /* Store offset/size/pointer for sync_all to copy pinned→host */
        dev->h_total_values = total_values + offset;
        dev->pinned_count = n;

        // NO synchronize — caller must call secp_gpu_sync_all() before reading total_values
        dev->current_slot = 1 - slot;
        offset += n;
    }

    cudaSetDevice(0);
    return 0;
}

/* Synchronize all GPU devices (wait for pending async operations) + copy pinned→host */
GPU_API void secp_gpu_sync_all(void) {
    for (int i = 0; i < num_gpu_devs; i++) {
        cudaSetDevice(gpu_devs[i].id);
        cudaStreamSynchronize(gpu_devs[i].stream);
        cudaStreamSynchronize(gpu_devs[i].stream2);

        /* Copy pinned output → caller's host buffer */
        gpu_device_t *dev = &gpu_devs[i];
        if (dev->h_total_values && dev->h_out_pinned && dev->pinned_count > 0) {
            size_t out_size = dev->pinned_count * sizeof(uint64_t);
            memcpy(dev->h_total_values, dev->h_out_pinned, out_size);
        }
    }
    cudaSetDevice(0);
}

/* Simple wrapper for single-GPU derive+lookup */
GPU_API int secp_gpu_derive_lookup_single(
    const uint8_t *privkeys,
    uint64_t *total_values,
    int count,
    uint32_t addr_types)
{
    return secp_gpu_derive_lookup(privkeys, total_values, count, addr_types, NULL, num_gpu_devs);
}

GPU_API void secp_gpu_cleanup(void) {
    for (int i = 0; i < num_gpu_devs; i++) {
        cudaSetDevice(gpu_devs[i].id);
        if (gpu_devs[i].d_privkeys) cudaFree(gpu_devs[i].d_privkeys);
        if (gpu_devs[i].d_pubkeys) cudaFree(gpu_devs[i].d_pubkeys);
        if (gpu_devs[i].d_script_entries) cudaFree(gpu_devs[i].d_script_entries);
        if (gpu_devs[i].d_all_data) cudaFree(gpu_devs[i].d_all_data);
        if (gpu_devs[i].d_utxo_data) cudaFree(gpu_devs[i].d_utxo_data);
        if (gpu_devs[i].d_out) cudaFree(gpu_devs[i].d_out);
        if (gpu_devs[i].d_out2) cudaFree(gpu_devs[i].d_out2);
        // Free pinned host memory
        if (gpu_devs[i].h_privkeys_pinned) cudaFreeHost(gpu_devs[i].h_privkeys_pinned);
        if (gpu_devs[i].h_out_pinned) cudaFreeHost(gpu_devs[i].h_out_pinned);
        if (gpu_devs[i].stream) cudaStreamDestroy(gpu_devs[i].stream);
        if (gpu_devs[i].stream2) cudaStreamDestroy(gpu_devs[i].stream2);
        gpu_devs[i].initialized = 0;
    }
    num_gpu_devs = 0;
    cudaSetDevice(0);
}

/* ============================================================
 * Standalone test
 * ============================================================ */

#ifdef STANDALONE_TEST
int main() {
    printf("secp256k1 CUDA GPU test\n");
    printf("Available devices: %d\n", secp_gpu_device_count());

    int n = secp_gpu_init();
    if (n < 0) return 1;

    /* Test with key = 1 */
    uint8_t privkey[32] = {0};
    privkey[31] = 1;

    uint8_t output[85];
    if (secp_gpu_derive(privkey, output, 1) == 0) {
        printf("PubKey(1) = %02x", output[0]);
        for (int i = 1; i < 33; i++) printf("%02x", output[i]);
        printf("\n");
        printf("Expected    = 0279BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798\n");
        // Print hash160
        printf("hash160   = ");
        for (int i = 33; i < 53; i++) printf("%02x", output[i]);
        printf("\n");
        printf("Expected  = 1698f40d626808d2bcf0e6cca7c0d5d4c835e0eb\n");
        // Print SHA256(pubkey)
        printf("SHA256(pk)= ");
        for (int i = 53; i < 85; i++) printf("%02x", output[i]);
        printf("\n");
    }

    secp_gpu_cleanup();
    return 0;
}
#endif
