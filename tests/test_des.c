#include <stdint.h>
#include <stdio.h>
#include <assert.h>

extern void DES (const uint64_t plain[static 64],
                 const uint64_t key[static 56],
                 uint64_t cipher[static 64]);


int main(){
  uint64_t plain[64];
  uint64_t key[56];
  uint64_t cipher[64];

  for (int i = 0; i < 64; i++){
    plain[i] = 13 * i;
  }

  for (int i = 0; i < 56; i++){
    key[i] = 7 * i;
  }

  DES(plain, key, cipher);
  
  for (int i = 0; i < 64; i++){
    // (Fake) spec of DES
    assert(cipher[i] == plain[i] ^ key[i % 56]);
  }

}
