from pathlib import Path

p = Path(__file__).with_name("secp256k1_kernel.cu")
text = p.read_text(encoding="utf-8")
assert len(text) > 10000, len(text)

start = text.find("__global__ void precompute_kernel")
end = text.find("/* ============================================================\n * SHA256", start)
assert start > 0 and end > start, (start, end)

new_pre = r"""__global__ void precompute_kernel(struct dev_prec *prec) {
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

"""
text = text[:start] + new_pre + text[end:]

old1 = """    /* Scalar multiplication: P = sk * G */
    point_t acc;
    fe_zero(&acc.x); fe_zero(&acc.y); fe_set1(&acc.z);

    int msb = -1;
    for (int i = FE_LIMBS - 1; i >= 0; i--) {
        if (sk[i] != 0) {
            for (int b = 31; b >= 0; b--) {
                if ((sk[i] >> b) & 1) { msb = i*32+b; goto found_msb; }
            }
        }
    }
found_msb:

    uint8_t *out = output + idx * OUT_STRIDE;
    if (msb < 0) { for (int i = 0; i < OUT_STRIDE; i++) out[i] = 0; return; }

    /* Double-and-add */
    fe_copy(&acc.x, &base_table[0].x);
    fe_copy(&acc.y, &base_table[0].y);
    fe_set1(&acc.z);

    point_t tp;
    for (int bit = msb - 1; bit >= 0; bit--) {
        pt_dbl(&acc, &acc);
        int limb = bit / 32;
        int pos  = bit % 32;
        if ((sk[limb] >> pos) & 1) {
            fe_copy(&tp.x, &base_table[0].x);
            fe_copy(&tp.y, &base_table[0].y);
            fe_set1(&tp.z);
            pt_add(&acc, &acc, &tp);
        }
    }

    /* Convert to affine */
    fe ox, oy;
    pt_to_affine(&ox, &oy, &acc);"""

new1 = """    uint8_t *out = output + idx * OUT_STRIDE;
    {
        int sk0 = 1;
        for (int i = 0; i < FE_LIMBS; i++) if (sk[i]) { sk0 = 0; break; }
        if (sk0) { for (int i = 0; i < OUT_STRIDE; i++) out[i] = 0; return; }
    }
    point_t acc;
    scalar_mul_g(&acc, sk, base_table);

    /* Convert to affine */
    fe ox, oy;
    pt_to_affine(&ox, &oy, &acc);"""

if old1 not in text:
    raise SystemExit("old1 missing")
text = text.replace(old1, new1, 1)

old2 = """    /* Scalar multiplication: P = sk * G (same as derive_pubkey_kernel) */
    point_t acc;
    fe_zero(&acc.x); fe_zero(&acc.y); fe_set1(&acc.z);

    int msb = -1;
    for (int i = FE_LIMBS - 1; i >= 0; i--) {
        if (sk[i] != 0) {
            for (int b = 31; b >= 0; b--) {
                if ((sk[i] >> b) & 1) { msb = i*32+b; goto found_msb2; }
            }
        }
    }
found_msb2:

    if (msb < 0) { total_values[idx] = 0; return; }

    /* Double-and-add */
    fe_copy(&acc.x, &base_table[0].x);
    fe_copy(&acc.y, &base_table[0].y);
    fe_set1(&acc.z);

    point_t tp;
    for (int bit = msb - 1; bit >= 0; bit--) {
        pt_dbl(&acc, &acc);
        int limb = bit / 32;
        int pos  = bit % 32;
        if ((sk[limb] >> pos) & 1) {
            fe_copy(&tp.x, &base_table[0].x);
            fe_copy(&tp.y, &base_table[0].y);
            fe_set1(&tp.z);
            pt_add(&acc, &acc, &tp);
        }
    }

    /* Convert to affine */
    fe ox, oy;
    pt_to_affine(&ox, &oy, &acc);"""

new2 = """    {
        int sk0 = 1;
        for (int i = 0; i < FE_LIMBS; i++) if (sk[i]) { sk0 = 0; break; }
        if (sk0) { total_values[idx] = 0; return; }
    }
    point_t acc;
    scalar_mul_g(&acc, sk, base_table);

    /* Convert to affine */
    fe ox, oy;
    pt_to_affine(&ox, &oy, &acc);"""

if old2 not in text:
    raise SystemExit("old2 missing")
text = text.replace(old2, new2, 1)

text = text.replace("point_t         base_table[8];", "point_t         base_table[16];", 1)

p.write_text(text, encoding="utf-8", newline="\n")
print("OK", p.stat().st_size)
print("scalar_mul_g", text.count("scalar_mul_g"))
print("Double-and-add left", text.count("Double-and-add"))
