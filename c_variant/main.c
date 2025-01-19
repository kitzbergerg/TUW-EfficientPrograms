#include <fcntl.h>
#include <immintrin.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <time.h>
#include <unistd.h>

// ---------------------- type defines and macros -----------------------

#define u0 void

#define s8 char
#define s16 short
#define s32 int
#define s64 long long

#define u8 unsigned char
#define u16 unsigned short
#define u32 unsigned int
#define u64 unsigned long long

#define f32 float
#define f64 double

#define INLINE static inline

#define MIN(X, Y) ((X) < (Y) ? (X) : (Y))
#define MAX(X, Y) ((X) > (Y) ? (X) : (Y))

#define WAIT() getchar();
#define ERR(X, ...)                               \
  fprintf(stderr, "ERR: " X "\n", ##__VA_ARGS__); \
  WAIT();

#define SWAP(T, a, b) \
  do {                \
    T tmp = a;        \
    a = b;            \
    b = tmp;          \
  } while (0)

// ---------------------- platform code --------------------------------

typedef int File;

File openRead(s8 *file_name) {
  mode_t mode = S_IRUSR | S_IWUSR | S_IRGRP | S_IROTH;
  File file = open(file_name, O_RDONLY, mode);

  if (file == -1) {
    ERR("Couldn't open file for reading");
    return 0;
  }

  return file;
}

s8 *readWholeFile(s8 *FileName, u32 *Size) {
  s8 *Buffer;
  File file;
  u32 FileSize, ReadBytes;

  file = openRead(FileName);

  if (file == 0) {
    return 0;
  }

  struct stat st;
  if (fstat(file, &st) == -1) {
    ERR("Error getting file size");
    close(file);
    return 0;
  }

  FileSize = st.st_size;
  Buffer = (s8 *)malloc(FileSize);

  if (Buffer == 0) {
    close(file);
    return 0;
  }

  ReadBytes = read(file, Buffer, FileSize);

  if (ReadBytes == -1) {
    ERR("Error reading file");
    close(file);
    return 0;
  }

  *Size = ReadBytes;

  close(file);

  return Buffer;
}

f64 platform_time() {
  struct timespec ts;
  clock_gettime(CLOCK_REALTIME, &ts);
  f64 seconds = (f64)ts.tv_sec + (f64)ts.tv_nsec / 1e9f;
  return seconds;
}

#define time() platform_time()

// ---------------------- primitives --------------------------------

typedef struct value {
  u32 start;
  u32 size;
} value;

u32 value_print(s8 *buffer, value v, s8 *result) {
  memcpy(result, buffer + v.start, v.size);
  return v.size;
}

INLINE u8 value_equal(s8 *buffer, value a, value b) {
  if (a.size != b.size) {
    return 0;
  } else {
    return memcmp(buffer + a.start, buffer + b.start, a.size) == 0;
  }
}

typedef struct record {
  value c1;
  value c2;
} record;

u32 record_print(s8 *buffer, record r, s8 *result) {
  u32 off = 0;
  off += value_print(buffer, r.c1, result);
  result[off++] = ',';
  off += value_print(buffer, r.c2, result + off);
  return off;
}

typedef struct merge {
  value c[5];
} merge;

u32 merge_print(s8 *buffer, merge r, s8 *result) {
  u32 off = 0;
  off += value_print(buffer, r.c[0], result);
  result[off++] = ',';
  off += value_print(buffer, r.c[1], result + off);
  result[off++] = ',';
  off += value_print(buffer, r.c[2], result + off);
  result[off++] = ',';
  off += value_print(buffer, r.c[3], result + off);
  result[off++] = ',';
  off += value_print(buffer, r.c[4], result + off);
  return off;
}

s32 merge_print_multiple(s8 *data_buffer, merge *data, u32 count) {
  // @TODO: just a guess technically this can lead to a buffer overrun
  s8 *buffer = (s8 *)malloc(sizeof(buffer[0]) * 1024 * 1024 * 1024);
  u32 lines = 20;
  u32 off = 0;

  for (u32 i = 0; i < count; i += lines) {
    u32 chunk = MIN(lines, count - i);

    for (u32 j = i; j < (i + chunk); ++j) {
      off += merge_print(data_buffer, data[j], buffer + off);
      buffer[off++] = '\n';
    }
  }

  if (fwrite(buffer, 1, off, stdout) != off) {
    ERR("Failed writing to stdout");
    return -1;
  }

  return 0;
}

// ---------------------- parsing and algorithm --------------------------------

typedef struct item {
  s8 *file;
  u32 file_size;
  record *records;
  u32 records_size;
} item;

u0 item_free(item *it) {
  free(it->file);
  free(it->records);
  memset(it, 0, sizeof(*it));
}

s32 item_parse(s8 *path, item *result) {
  u32 size;

  s8 *file = readWholeFile(path, &size);
  if (file == 0) {
    ERR("Couldn't open file");
    return -1;
  }

  s8 *str = file;
  u32 lines = 1024 * 1024 * 10;
  u32 sz = lines;

  record *records = (record *)malloc(sizeof(record) * sz);

  if (!records) {
    ERR("Failed allocating record buffer for item");
    return -1;
  }

  u32 cols = 2;

#define GIB 1024 * 1024 * 1024

  u32 stored = 0;
  s32 *indices = (s32 *)malloc(sizeof(indices[0]) * GIB);
  indices[stored++] = -1;

  __m512i comma = _mm512_set1_epi8(',');
  __m512i newline = _mm512_set1_epi8('\n');
  __m512i range = _mm512_set_epi8(
      63, 62, 61, 60, 59, 58, 57, 56, 55, 54, 53, 52, 51, 50, 49, 48, 47, 46,
      45, 44, 43, 42, 41, 40, 39, 38, 37, 36, 35, 34, 33, 32, 31, 30, 29, 28,
      27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17, 16, 15, 14, 13, 12, 11, 10, 9,
      8, 7, 6, 5, 4, 3, 2, 1, 0);

  u8 offsets[8];
  for (u32 i = 0; i < size; i += 64) {
    __m512i chunk = _mm512_loadu_epi8(file + i);

    u64 m = _mm512_cmpeq_epi8_mask(comma, chunk) |
            _mm512_cmpeq_epi8_mask(newline, chunk);

    __m512i hits = _mm512_maskz_mov_epi8(m, range);

    u32 count = _mm_popcnt_u64(m);
    _mm512_mask_compressstoreu_epi8(offsets, m, hits);

    for (u32 j = 0; j < count; j++) {
      indices[stored + j] = offsets[j] + i;
    }

    stored += count;
  }

  u32 count = 0;

  for (u32 i = 0; i < (stored - 1); i += cols) {
    if (sz <= count) {
      u32 tmp_sz = sz + lines;
      record *tmp = (record *)realloc(records, sizeof(tmp[0]) * tmp_sz);

      if (!tmp) {
        ERR("Failed reallocating record buffer for item");
        return -1;
      }

      sz = tmp_sz;
      records = tmp;
    }

    value *c = (value *)(records + count++);

    c[0].start = indices[i] + 1;
    c[0].size = indices[i + 1] - c[0].start;

    c[1].start = indices[i + 1] + 1;
    c[1].size = indices[i + 2] - c[1].start;
  }

  free(indices);

  result->records = records;
  result->records_size = count;
  result->file = file;
  result->file_size = size;

  return 0;
}

u32 npof2(u32 n) {
  if (n == 0) {
    return 1;
  }

  n--;
  n |= n >> 1;
  n |= n >> 2;
  n |= n >> 4;
  n |= n >> 8;
  n |= n >> 16;
  n++;

  return n;
}

u32 hash_table_likely_optimal(u32 sz) { return MAX(64, (sz * 162) / 100); }

INLINE u32 hash_table_hash1(s8 *buffer, value c1) {
  u32 hash = 2166136261u;
  s8 *data = buffer + c1.start;

  for (u32 i = 0; i < c1.size; ++i) {
    hash ^= (u8)data[i];
    hash *= 16777619u;
  }

  return hash;
}

u0 hash_sort(s8 *buffer, u8 *records0, u32 stride0, u32 size0, u32 lut_size,
             u32 **result_offsets, u8 **result) {
  u32 *hashes = (u32 *)malloc(sizeof(hashes[0]) * size0);
  u32 *lut = (u32 *)malloc(sizeof(lut[0]) * lut_size);
  u32 *offsets = (u32 *)malloc(sizeof(offsets[0]) * lut_size);
  u8 *tmp = (u8 *)malloc(sizeof(tmp[0]) * size0 * stride0);

  for (s32 i = 0; i < size0; ++i) {
    hashes[i] = hash_table_hash1(buffer, *(value *)(records0 + i * stride0)) &
                (lut_size - 1);
  }

  memset(lut, 0, lut_size * sizeof(lut[0]));
  memset(offsets, 0, lut_size * sizeof(lut[0]));

  for (s32 i = 0; i < size0; ++i) {
    u32 hash = hashes[i];
    lut[hash]++;
  }

  s32 sum = 0;
  for (s32 j = 0; j < lut_size; ++j) {
    s32 tmp = lut[j];
    lut[j] = sum;
    sum += tmp;
    offsets[j] = sum;
  }

  for (s32 i = 0; i < size0; ++i) {
    u32 hash = hashes[i];
    u32 p = lut[hash]++;

    memcpy(tmp + p * stride0, records0 + i * stride0, stride0);
  }

  free(hashes);
  free(lut);

  *result = tmp;
  *result_offsets = offsets;
}

s8 *reorder(s8 *buffer0, u8 *records0, u32 stride0, u32 size0, s8 *buffer1,
            u8 *records1, u32 stride1, u32 size1) {
  // @TODO: just guessing
  u32 sz = 1024 * 1024 * 1024;
  s8 *join_buffer = (s8 *)malloc(sizeof((join_buffer)[0]) * sz);
  s8 *pos = join_buffer;

  u32 values0 = stride0 / sizeof(value);

  for (u32 i = 0; i < size0 * values0; ++i) {
    value *v = (value *)(records0) + i;
    memcpy(pos, buffer0 + v->start, v->size);
    v->start = pos - join_buffer;
    pos += v->size;
  }

  u32 values1 = stride1 / sizeof(value);

  for (u32 i = 0; i < size1 * values1; ++i) {
    value *v = (value *)(records1) + i;
    memcpy(pos, buffer1 + v->start, v->size);
    v->start = pos - join_buffer;
    pos += v->size;
  }

  return join_buffer;
}

s32 hash_join(s8 *buffer0, u8 *records0, u32 stride0, u32 size0, s8 *buffer1,
              u8 *records1, u32 stride1, u32 size1, u8 **join, s8 **join_buffer,
              u32 *join_size) {
  // division by 8 turned out to be the sweet spot for the table size
  u32 lut_size = npof2(MAX(size0, size1)) / 8;

  u8 *sorted0, *sorted1;
  u32 *offsets0, *offsets1;

  hash_sort(buffer0, records0, stride0, size0, lut_size, &offsets0, &sorted0);
  hash_sort(buffer1, records1, stride1, size1, lut_size, &offsets1, &sorted1);

  *join_buffer = reorder(buffer0, sorted0, stride0, size0, buffer1, sorted1,
                         stride1, size1);

  u32 count = 0;
  u32 result_stride = stride0 + stride1 - sizeof(value);
  u32 result_size = lut_size;
  u8 *tmp = (u8 *)malloc(sizeof(tmp[0]) * result_stride * result_size);

  for (u32 i = 0; i < lut_size; ++i) {
    u32 lo0 = i == 0 ? 0 : offsets0[i - 1];
    u32 hi0 = offsets0[i];
    u32 lo1 = i == 0 ? 0 : offsets1[i - 1];
    u32 hi1 = offsets1[i];

    for (u32 j = lo0; j < hi0; ++j) {
      for (u32 k = lo1; k < hi1; ++k) {
        value *a = (value *)(sorted0 + j * stride0);
        value *b = (value *)(sorted1 + k * stride1);

        if (value_equal(*join_buffer, *a, *b)) {
          if (result_size <= count) {
            u32 new_size = result_size + lut_size;
            u8 *tmp_ = (u8 *)realloc(tmp, result_stride * new_size);

            if (!tmp_) {
              ERR("Failed reallocation in hash_table_join\n");
              return -1;
            }

            tmp = tmp_;
            result_size = new_size;
          }

          u8 *m = tmp + result_stride * count++;

          memcpy(m, a, stride0);
          memcpy(m + stride0, b + 1, stride1 - sizeof(value));
        }
      }
    }
  }

  free(offsets0);
  free(offsets1);
  free(sorted0);
  free(sorted1);

  *join = tmp;
  *join_size = count;

  return 0;
}

s32 main(s32 argc, s8 *argv[]) {
  if (argc != 5) {
    printf("usage: %s f1.csv f2.csv f3.csv f4.csv\n", argv[0]);
    return 1;
  }

  item items[4];
  s32 i;
  u32 failed = 0;

  for (i = 0; i < 4; ++i) {
    if (item_parse(argv[i + 1], &items[i]) != 0) failed = 1;
  }

  if (failed) return 1;

  s8 *join1_buffer;
  value *join1;
  u32 join_size1;

  if (hash_join(items[1].file, (u8 *)items[1].records, 2 * sizeof(value),
                items[1].records_size, items[2].file, (u8 *)items[2].records,
                2 * sizeof(value), items[2].records_size, (u8 **)&join1,
                &join1_buffer, &join_size1) != 0)
    return 1;

  s8 *join2_buffer;
  value *join2;
  u32 join_size2;

  if (hash_join(items[0].file, (u8 *)items[0].records, 2 * sizeof(value),
                items[0].records_size, join1_buffer, (u8 *)join1,
                3 * sizeof(value), join_size1, (u8 **)&join2, &join2_buffer,
                &join_size2) != 0)
    return 1;

  free(join1);
  free(join1_buffer);

  for (u32 i = 0; i < join_size2; ++i) {
    SWAP(value, join2[4 * i], join2[4 * i + 3]);
  }

  s8 *join3_buffer;
  value *join3;
  u32 join_size3;

  if (hash_join(items[3].file, (u8 *)items[3].records, 2 * sizeof(value),
                items[3].records_size, join2_buffer, (u8 *)join2,
                4 * sizeof(value), join_size2, (u8 **)&join3, &join3_buffer,
                &join_size3) != 0)
    return 1;

  free(join2);
  free(join2_buffer);

  for (u32 i = 0; i < join_size3; ++i) {
    SWAP(value, join3[5 * i + 1], join3[5 * i + 4]);
  }

  if (merge_print_multiple(join3_buffer, (merge *)join3, join_size3) != 0)
    return 1;

  free(join3);
  free(join3_buffer);

  item_free(&items[0]);
  item_free(&items[1]);
  item_free(&items[2]);
  item_free(&items[3]);

  return 0;
}
