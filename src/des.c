#include <stdint.h>

/*
 * Bitsliced implementation of (fake) DES
 */

void DES (/*inputs*/  const uint64_t plain[static 64],
                      const uint64_t key[static 56],
          /*outputs*/ uint64_t cipher[static 64]){
  for (int i = 0; i < 64; i++){
    // Dummy implementation
    cipher[i] = plain[i] ^ key[i % 56];
  }
}
